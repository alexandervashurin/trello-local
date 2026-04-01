// backend/src/controllers/oauth.rs
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    Extension, Json,
};
use oauth2::{
    basic::BasicClient,
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
    TokenResponse,
};
use sqlx::SqlitePool;
use crate::models::{OAuthCallback, OAuthUrl};
use crate::views::{AuthToken, ClaimsWith2FA};
use crate::controllers::auth::generate_token;
use crate::controllers::sessions;
use uuid::Uuid;

/// Получить URL для авторизации GitHub
pub async fn github_auth_url(
    State(_pool): State<SqlitePool>,
) -> Result<Json<OAuthUrl>, (StatusCode, String)> {
    let client_id = std::env::var("GITHUB_CLIENT_ID")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GITHUB_CLIENT_ID not configured".to_string()))?;
    
    let state = Uuid::new_v4().to_string();
    
    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        None,
    )
    .set_redirect_uri(
        RedirectUrl::new(get_redirect_url("/api/oauth/github/callback"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    );

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Ok(Json(OAuthUrl {
        url: auth_url.to_string(),
        state,
    }))
}

/// Callback для GitHub OAuth
pub async fn github_callback(
    State(pool): State<SqlitePool>,
    Query(params): Query<OAuthCallback>,
) -> Result<Redirect, (StatusCode, String)> {
    let client_id = std::env::var("GITHUB_CLIENT_ID")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GITHUB_CLIENT_ID not configured".to_string()))?;
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GITHUB_CLIENT_SECRET not configured".to_string()))?;

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        Some(
            TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(get_redirect_url("/api/oauth/github/callback"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    );

    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    let token_response = match token_result {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("GitHub token exchange failed: {:?}", e);
            return Ok(Redirect::to("/login.html?error=oauth_failed"));
        }
    };

    // Получаем информацию о пользователе из GitHub API
    let github_user = get_github_user(token_response.access_token().secret()).await;
    
    match github_user {
        Ok(user_info) => {
            // Проверяем, есть ли уже такой OAuth аккаунт
            let existing_oauth: Option<(i64,)> = sqlx::query_as(
                "SELECT user_id FROM oauth_accounts WHERE provider = 'github' AND provider_user_id = ?",
            )
            .bind(&user_info.provider_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            let user_id = if let Some((uid,)) = existing_oauth {
                uid
            } else {
                // Создаём нового пользователя или связываем с существующим по email
                let existing_user: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM users WHERE email = ?",
                )
                .bind(&user_info.email.clone().unwrap_or_default())
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten();

                if let Some((uid,)) = existing_user {
                    // Привязываем OAuth к существующему пользователю
                    let _ = sqlx::query(
                        "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token) VALUES (?, ?, ?, ?)",
                    )
                    .bind(uid)
                    .bind("github")
                    .bind(&user_info.provider_user_id)
                    .bind(token_response.access_token().secret())
                    .execute(&pool)
                    .await;
                    uid
                } else {
                    // Создаём нового пользователя
                    let username = user_info.name.clone().unwrap_or_else(|| format!("user_{}", user_info.provider_user_id));
                    let result = sqlx::query_as::<_, (i64,)>(
                        "INSERT INTO users (username, email, oauth_enabled) VALUES (?, ?, 1) RETURNING id",
                    )
                    .bind(&username)
                    .bind(&user_info.email.clone().unwrap_or_default())
                    .fetch_one(&pool)
                    .await;

                    match result {
                        Ok((uid,)) => {
                            let _ = sqlx::query(
                                "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token) VALUES (?, ?, ?, ?)",
                            )
                            .bind(uid)
                            .bind("github")
                            .bind(&user_info.provider_user_id)
                            .bind(token_response.access_token().secret())
                            .execute(&pool)
                            .await;
                            uid
                        }
                        Err(e) => {
                            tracing::error!("Failed to create user: {:?}", e);
                            return Ok(Redirect::to("/login.html?error=registration_failed"));
                        }
                    }
                }
            };

            // Получаем информацию о пользователе для токена
            let user_info_result: Option<(String,)> = sqlx::query_as(
                "SELECT username FROM users WHERE id = ?",
            )
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            let username = user_info_result.map(|(u,)| u).unwrap_or_else(|| "user".to_string());

            // Генерируем токен
            let token = match generate_token(user_id, &username, true) {
                Ok(t) => t,
                Err(_) => return Ok(Redirect::to("/login.html?error=token_error")),
            };

            // Сохраняем сессию
            let _ = sessions::save_session(&pool, user_id, &token, None, None).await;

            // Редирект на главную с токеном
            Ok(Redirect::to(&format!("/?token={}&username={}", token, username)))
        }
        Err(e) => {
            tracing::error!("Failed to get GitHub user info: {:?}", e);
            Ok(Redirect::to("/login.html?error=oauth_failed"))
        }
    }
}

/// Получить URL для авторизации Google
pub async fn google_auth_url(
    State(_pool): State<SqlitePool>,
) -> Result<Json<OAuthUrl>, (StatusCode, String)> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GOOGLE_CLIENT_ID not configured".to_string()))?;
    
    let state = Uuid::new_v4().to_string();
    
    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        None,
    )
    .set_redirect_uri(
        RedirectUrl::new(get_redirect_url("/api/oauth/google/callback"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    );

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Json(OAuthUrl {
        url: auth_url.to_string(),
        state,
    }))
}

/// Callback для Google OAuth
pub async fn google_callback(
    State(pool): State<SqlitePool>,
    Query(params): Query<OAuthCallback>,
) -> Result<Redirect, (StatusCode, String)> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GOOGLE_CLIENT_ID not configured".to_string()))?;
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "GOOGLE_CLIENT_SECRET not configured".to_string()))?;

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        Some(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(get_redirect_url("/api/oauth/google/callback"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    );

    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    let token_response = match token_result {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Google token exchange failed: {:?}", e);
            return Ok(Redirect::to("/login.html?error=oauth_failed"));
        }
    };

    // Получаем информацию о пользователе из Google API
    let google_user = get_google_user(token_response.access_token().secret()).await;
    
    match google_user {
        Ok(user_info) => {
            // Проверяем, есть ли уже такой OAuth аккаунт
            let existing_oauth: Option<(i64,)> = sqlx::query_as(
                "SELECT user_id FROM oauth_accounts WHERE provider = 'google' AND provider_user_id = ?",
            )
            .bind(&user_info.provider_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            let user_id = if let Some((uid,)) = existing_oauth {
                uid
            } else {
                // Создаём нового пользователя или связываем с существующим по email
                let existing_user: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM users WHERE email = ?",
                )
                .bind(&user_info.email.clone().unwrap_or_default())
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten();

                if let Some((uid,)) = existing_user {
                    let _ = sqlx::query(
                        "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token) VALUES (?, ?, ?, ?)",
                    )
                    .bind(uid)
                    .bind("google")
                    .bind(&user_info.provider_user_id)
                    .bind(token_response.access_token().secret())
                    .execute(&pool)
                    .await;
                    uid
                } else {
                    let username = user_info.name.unwrap_or_else(|| format!("user_{}", user_info.provider_user_id));
                    let result = sqlx::query_as::<_, (i64,)>(
                        "INSERT INTO users (username, email, oauth_enabled) VALUES (?, ?, 1) RETURNING id",
                    )
                    .bind(&username)
                    .bind(&user_info.email.clone().unwrap_or_default())
                    .fetch_one(&pool)
                    .await;

                    match result {
                        Ok((uid,)) => {
                            let _ = sqlx::query(
                                "INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token) VALUES (?, ?, ?, ?)",
                            )
                            .bind(uid)
                            .bind("google")
                            .bind(&user_info.provider_user_id)
                            .bind(token_response.access_token().secret())
                            .execute(&pool)
                            .await;
                            uid
                        }
                        Err(e) => {
                            tracing::error!("Failed to create user: {:?}", e);
                            return Ok(Redirect::to("/login.html?error=registration_failed"));
                        }
                    }
                }
            };

            let user_info_result: Option<(String,)> = sqlx::query_as(
                "SELECT username FROM users WHERE id = ?",
            )
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            let username = user_info_result.map(|(u,)| u).unwrap_or_else(|| "user".to_string());

            let token = match generate_token(user_id, &username, true) {
                Ok(t) => t,
                Err(_) => return Ok(Redirect::to("/login.html?error=token_error")),
            };

            let _ = sessions::save_session(&pool, user_id, &token, None, None).await;

            Ok(Redirect::to(&format!("/?token={}&username={}", token, username)))
        }
        Err(e) => {
            tracing::error!("Failed to get Google user info: {:?}", e);
            Ok(Redirect::to("/login.html?error=oauth_failed"))
        }
    }
}

/// Получить информацию о пользователе GitHub
async fn get_github_user(access_token: &str) -> Result<crate::models::OAuthUserInfo, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("User-Agent", "Trello-Local")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("Failed to get GitHub user info".to_string());
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    Ok(crate::models::OAuthUserInfo {
        provider_user_id: json["id"].as_u64().unwrap_or(0).to_string(),
        email: json["email"].as_str().map(|s| s.to_string()),
        name: json["login"].as_str().map(|s| s.to_string()),
        avatar: json["avatar_url"].as_str().map(|s| s.to_string()),
    })
}

/// Получить информацию о пользователе Google
async fn get_google_user(access_token: &str) -> Result<crate::models::OAuthUserInfo, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("Failed to get Google user info".to_string());
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    Ok(crate::models::OAuthUserInfo {
        provider_user_id: json["id"].as_str().unwrap_or("").to_string(),
        email: json["email"].as_str().map(|s| s.to_string()),
        name: json["name"].as_str().map(|s| s.to_string()),
        avatar: json["picture"].as_str().map(|s| s.to_string()),
    })
}

fn get_redirect_url(path: &str) -> String {
    let base_url = std::env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    format!("{}{}", base_url, path)
}
