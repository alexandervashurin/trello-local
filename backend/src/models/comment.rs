use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct Comment {
    pub id: i64,
    pub card_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone)]
pub struct CommentWithUser {
    pub id: i64,
    pub card_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct CreateComment {
    pub content: String,
}

#[derive(Deserialize, Default)]
pub struct UpdateComment {
    pub content: Option<String>,
}
