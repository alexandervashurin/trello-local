use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{RegisterUser, LoginUser, User};
use crate::views::AuthToken;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::SystemTime;

use crate::views::Claims;
use crate::controllers::sessions;

// Секретный ключ для JWT (в продакшене использовать переменную окружения!)
const JWT_SECRET: &[u8] = b"trello-local-secret-key-change-in-production-2024";

/// Регистрация нового пользователя
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, strftime('%s', 'now')) RETURNING id, username, created_at",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let token = generate_token(user.id, &user.username)?;

    // Сохраняем сессию (без user_agent и ip для тестов)
    let _ = sessions::save_session(&pool, user.id, &token, None, None).await;

    Ok(Json(AuthToken {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// Вход пользователя
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    let user_with_password: crate::models::UserWithPassword = sqlx::query_as(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()))?;

    let valid = bcrypt::verify(&payload.password, &user_with_password.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()));
    }

    let token = generate_token(user_with_password.id, &user_with_password.username)?;

    // Сохраняем сессию (без user_agent и ip для тестов)
    let _ = sessions::save_session(&pool, user_with_password.id, &token, None, None).await;

    Ok(Json(AuthToken {
        token,
        user_id: user_with_password.id,
        username: user_with_password.username,
    }))
}

/// Генерация JWT токена
fn generate_token(user_id: i64, username: &str) -> Result<String, (StatusCode, String)> {
    let expiration = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 60 * 60 * 24 * 7; // 7 дней

    let claims = Claims {
        user_id,
        username: username.to_string(),
        exp: expiration as usize,
        user_agent: None,
        ip_address: None,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Валидация токена
pub async fn validate_token(
    State(_pool): State<SqlitePool>,
    token: String,
) -> Result<Json<Claims>, (StatusCode, String)> {
    use jsonwebtoken::{decode, Validation, Algorithm, DecodingKey};
    
    decode::<Claims>(&token, &DecodingKey::from_secret(JWT_SECRET), &Validation::new(Algorithm::HS256))
        .map(|data| Json(data.claims))
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Неверный токен".to_string()))
}
