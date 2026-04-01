use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Уведомление пользователя
#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: String, // info, success, warning, error
    pub is_read: bool,
    pub created_at: i64,
    pub link: Option<String>, // Ссылка на связанный ресурс (опционально)
}

/// Уведомление с информацией о создателе
#[derive(Serialize, Clone, Debug)]
pub struct NotificationWithCreator {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub is_read: bool,
    pub created_at: i64,
    pub link: Option<String>,
    pub creator_username: Option<String>,
}

/// Создание уведомления
#[derive(Deserialize)]
pub struct CreateNotification {
    pub title: String,
    pub message: String,
    pub notification_type: Option<String>,
    pub link: Option<String>,
}

/// Обновление статуса прочтения
#[derive(Deserialize)]
pub struct UpdateNotificationRead {
    pub is_read: bool,
}
