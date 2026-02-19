use axum::{extract::State, http::StatusCode, Json};
use crate::models::{RegisterUser, LoginUser, AuthToken, Claims, User};
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm, EncodingKey, DecodingKey};
use std::time::SystemTime;

// Секретный ключ для JWT (в продакшене использовать переменную окружения!)
const JWT_SECRET: &[u8] = b"trello-local-secret-key-change-in-production-2024";

pub async fn register(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    // Хэшируем пароль
    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Создаём пользователя
    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, strftime('%s', 'now')) RETURNING id, username, created_at",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Генерируем JWT токен
    let token = generate_token(user.id, &user.username)?;

    Ok(Json(AuthToken {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

pub async fn login(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    // Получаем пользователя с паролем
    let user_with_password: crate::models::UserWithPassword = sqlx::query_as(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()))?;

    // Проверяем пароль
    let valid = bcrypt::verify(&payload.password, &user_with_password.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()));
    }

    // Генерируем JWT токен
    let token = generate_token(user_with_password.id, &user_with_password.username)?;

    Ok(Json(AuthToken {
        token,
        user_id: user_with_password.id,
        username: user_with_password.username,
    }))
}

fn generate_token(user_id: i64, username: &str) -> Result<String, (StatusCode, String)> {
    let expiration = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 60 * 60 * 24 * 7; // 7 дней

    let claims = Claims {
        user_id,
        username: username.to_string(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn validate_token(
    State(_pool): State<sqlx::SqlitePool>,
    token: String,
) -> Result<Json<Claims>, (StatusCode, String)> {
    decode::<Claims>(&token, &DecodingKey::from_secret(JWT_SECRET), &Validation::new(Algorithm::HS256))
        .map(|data| Json(data.claims))
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Неверный токен".to_string()))
}
