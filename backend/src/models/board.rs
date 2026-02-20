use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct Board {
    pub id: i64,
    pub title: String,
    pub owner_id: i64,
    pub is_shared: bool,
    pub visibility: String,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct BoardMember {
    pub board_id: i64,
    pub user_id: i64,
    pub role: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct CreateBoard {
    pub title: String,
    #[serde(default)]
    pub is_shared: bool,
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "private".to_string()
}

#[derive(Deserialize, Default)]
pub struct UpdateBoard {
    pub title: Option<String>,
    pub is_shared: Option<bool>,
    pub visibility: Option<String>,
}

#[derive(Deserialize)]
pub struct AddBoardMember {
    pub user_id: i64,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "member".to_string()
}

#[derive(Deserialize)]
pub struct CreateInvitation {
    pub role: String,
    pub expires_in_hours: Option<i64>,
}

#[derive(Serialize, FromRow, Clone)]
pub struct BoardInvitation {
    pub id: i64,
    pub board_id: i64,
    pub token: String,
    pub role: String,
    pub created_by: i64,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub used: bool,
}
