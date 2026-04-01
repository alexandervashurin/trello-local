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

/// Получение JWT secret из переменной окружения
/// В production среде JWT_SECRET должен быть установлен обязательно
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .inspect_err(|_| {
            tracing::warn!("JWT_SECRET не установлен! Используйте уникальное значение в production");
        })
        .unwrap_or_else(|_| {
            // Генерируем случайный секрет только для разработки
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Время не может идти вспять")
                .as_nanos();
            format!("dev-secret-{}", seed)
        })
        .into_bytes()
}

/// Регистрация нового пользователя
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    // Валидация имени пользователя
    if payload.username.trim().len() < 3 {
        return Err((StatusCode::BAD_REQUEST, "Имя пользователя должно быть не менее 3 символов".to_string()));
    }
    if payload.username.len() > 50 {
        return Err((StatusCode::BAD_REQUEST, "Имя пользователя слишком длинное".to_string()));
    }

    // Валидация пароля
    if payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Пароль должен быть не менее 8 символов".to_string()));
    }
    
    // Проверка сложности пароля
    let has_upper = payload.password.chars().any(|c| c.is_uppercase());
    let has_lower = payload.password.chars().any(|c| c.is_lowercase());
    let has_digit = payload.password.chars().any(|c| c.is_numeric());
    
    if !has_upper || !has_lower || !has_digit {
        return Err((StatusCode::BAD_REQUEST, 
            "Пароль должен содержать заглавные и строчные буквы, а также цифры".to_string()));
    }

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, strftime('%s', 'now')) RETURNING id, username, email, avatar_color, bio, last_login, created_at",
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

    // Логирование успешной регистрации
    tracing::info!(
        target: "security",
        username = %payload.username,
        user_id = user.id,
        event = "register_success",
        "Успешная регистрация нового пользователя"
    );

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
        "SELECT id, username, password_hash, email, avatar_color, bio, last_login, created_at FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        // Логирование неудачной попытки входа (пользователь не найден)
        tracing::warn!(
            target: "security",
            username = %payload.username,
            event = "login_failed",
            reason = "user_not_found",
            "Неудачная попытка входа: пользователь не найден"
        );
        (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string())
    })?;

    let valid = bcrypt::verify(&payload.password, &user_with_password.password_hash)
        .map_err(|e| {
            // Логирование ошибки проверки пароля
            tracing::error!(
                target: "security",
                username = %payload.username,
                event = "login_error",
                reason = "bcrypt_error",
                error = %e,
                "Ошибка при проверке пароля"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if !valid {
        // Логирование неудачной попытки входа (неверный пароль)
        tracing::warn!(
            target: "security",
            username = %payload.username,
            user_id = user_with_password.id,
            event = "login_failed",
            reason = "invalid_password",
            "Неудачная попытка входа: неверный пароль"
        );
        return Err((StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()));
    }

    // Успешный вход
    tracing::info!(
        target: "security",
        username = %payload.username,
        user_id = user_with_password.id,
        event = "login_success",
        "Успешный вход пользователя"
    );

    // Обновляем last_login
    let _ = sqlx::query("UPDATE users SET last_login = strftime('%s', 'now') WHERE id = ?")
        .bind(user_with_password.id)
        .execute(&pool)
        .await;

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
        .expect("Время не может идти вспять")
        .as_secs() + 60 * 60 * 24 * 7; // 7 дней

    let claims = Claims {
        user_id,
        username: username.to_string(),
        exp: expiration as usize,
        user_agent: None,
        ip_address: None,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(&get_jwt_secret()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Валидация токена
pub async fn validate_token(
    State(_pool): State<SqlitePool>,
    token: String,
) -> Result<Json<Claims>, (StatusCode, String)> {
    use jsonwebtoken::{decode, Validation, Algorithm, DecodingKey};

    decode::<Claims>(&token, &DecodingKey::from_secret(&get_jwt_secret()), &Validation::new(Algorithm::HS256))
        .map(|data| Json(data.claims))
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Неверный токен".to_string()))
}
