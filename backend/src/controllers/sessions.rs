use crate::models::{Session, SessionInfo};
use crate::views::Claims;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// Получить все сессии текущего пользователя
pub async fn get_sessions(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SessionInfo>>, (StatusCode, String)> {
    let sessions: Vec<Session> = sqlx::query_as(
        "SELECT id, user_id, token_hash, user_agent, ip_address, created_at, expires_at, last_activity FROM sessions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session_infos: Vec<SessionInfo> = sessions
        .into_iter()
        .map(|s| {
            SessionInfo {
                id: s.id,
                user_id: s.user_id,
                user_agent: s.user_agent,
                ip_address: s.ip_address,
                created_at: s.created_at,
                expires_at: s.expires_at,
                last_activity: s.last_activity,
                is_current: false, // Будет установлено ниже
            }
        })
        .collect();

    Ok(Json(session_infos))
}

/// Завершить конкретную сессию
pub async fn delete_session(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<i64>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
        .bind(session_id)
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Сессия не найдена".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Завершить все сессии текущего пользователя (logout везде)
pub async fn delete_all_sessions(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let _result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(()))
}

/// Хэширование токена для безопасного хранения
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Сохранение сессии в БД
pub async fn save_session(
    pool: &PgPool,
    user_id: i64,
    token: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(token);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() as i64;
    let expires_at = now + 60 * 60 * 24 * 7; // 7 дней

    sqlx::query(
        "INSERT INTO sessions (user_id, token_hash, user_agent, ip_address, created_at, expires_at, last_activity) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(user_agent)
    .bind(ip_address)
    .bind(now)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Обновление времени последней активности сессии
pub async fn update_session_activity(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(token);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() as i64;

    sqlx::query("UPDATE sessions SET last_activity = $1 WHERE token_hash = $2")
        .bind(now)
        .bind(&token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

/// Проверка токена по сессии
pub async fn is_session_valid(pool: &PgPool, token: &str) -> Result<bool, sqlx::Error> {
    let token_hash = hash_token(token);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() as i64;

    let result: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM sessions WHERE token_hash = $1 AND expires_at > $2")
            .bind(&token_hash)
            .bind(now)
            .fetch_optional(pool)
            .await?;

    Ok(result.is_some())
}
