use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;
use jsonwebtoken::{decode, Validation, Algorithm, DecodingKey};

use crate::views::Claims;

// Секретный ключ для JWT (в продакшене использовать переменную окружения!)
const JWT_SECRET: &[u8] = b"trello-local-secret-key-change-in-production-2024";

/// Извлечение Claims из запроса
pub async fn extract_claims(
    State(_pool): State<SqlitePool>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let claims = match auth_header {
        Some(header) => {
            // Ожидаем формат "Bearer <token>"
            if !header.starts_with("Bearer ") {
                return Err((StatusCode::UNAUTHORIZED, "Неверный формат Authorization заголовка".to_string()));
            }

            let token = header.trim_start_matches("Bearer ").trim();

            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(Algorithm::HS256),
            ) {
                Ok(token_data) => Some(token_data.claims),
                Err(_) => return Err((StatusCode::UNAUTHORIZED, "Неверный или истёкший токен".to_string())),
            }
        }
        None => None, // Токен не обязателен для некоторых эндпоинтов
    };

    // Добавляем claims в Extension запроса
    if let Some(claims) = claims {
        request.extensions_mut().insert(claims);
    }

    Ok(next.run(request).await)
}

/// Middleware для обязательной аутентификации
pub async fn require_auth(
    State(_pool): State<SqlitePool>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or((StatusCode::UNAUTHORIZED, "Требуется аутентификация".to_string()))?;

    // Проверяем, что user_id валиден
    if claims.user_id <= 0 {
        return Err((StatusCode::UNAUTHORIZED, "Неверный user_id в токене".to_string()));
    }

    Ok(next.run(request).await)
}

/// Извлечение Claims из запроса (удобная функция для контроллеров)
pub fn get_claims_from_request(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

/// Извлечение Claims из запроса с ошибкой если нет
pub fn get_claims_or_unauthorized(request: &Request) -> Result<&Claims, (StatusCode, String)> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or((StatusCode::UNAUTHORIZED, "Требуется аутентификация".to_string()))
}
