use crate::models::{CreateNotification, Notification};
use crate::views::Claims;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;

/// Параметры запроса уведомлений
#[derive(Deserialize)]
pub struct NotificationQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i64>,
}

/// Получить уведомления текущего пользователя
pub async fn get_notifications(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<Vec<Notification>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50);

    let notifications: Vec<Notification> = if query.unread_only.unwrap_or(false) {
        sqlx::query_as(
            "SELECT id, user_id, title, message, notification_type, is_read, created_at, link FROM notifications WHERE user_id = $1 AND is_read = FALSE ORDER BY created_at DESC LIMIT $2",
        )
        .bind(claims.user_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, title, message, notification_type, is_read, created_at, link FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(claims.user_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(notifications))
}

/// Создать уведомление (для внутренних нужд системы)
pub async fn create_notification(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateNotification>,
) -> Result<Json<Notification>, (StatusCode, String)> {
    let notification_type = payload
        .notification_type
        .unwrap_or_else(|| "info".to_string());

    let notification: Notification = sqlx::query_as(
        "INSERT INTO notifications (user_id, title, message, notification_type, link) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(claims.user_id)
    .bind(&payload.title)
    .bind(&payload.message)
    .bind(&notification_type)
    .bind(&payload.link)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(notification))
}

/// Отметить уведомление как прочитанное
pub async fn mark_notification_read(
    Path(notification_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Уведомление не найдено".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Отметить все уведомления как прочитанные
pub async fn mark_all_read(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    sqlx::query("UPDATE notifications SET is_read = TRUE WHERE user_id = $1")
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(()))
}

/// Удалить уведомление
pub async fn delete_notification(
    Path(notification_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Уведомление не найдено".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Получить количество непрочитанных уведомлений
pub async fn get_unread_count(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<i64>, (StatusCode, String)> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE")
            .bind(claims.user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(count.0))
}

/// Создать системное уведомление (внутренняя функция)
pub async fn create_system_notification(
    pool: &PgPool,
    user_id: i64,
    title: &str,
    message: &str,
    notification_type: &str,
    link: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO notifications (user_id, title, message, notification_type, link) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(title)
    .bind(message)
    .bind(notification_type)
    .bind(link)
    .execute(pool)
    .await?;

    Ok(())
}
