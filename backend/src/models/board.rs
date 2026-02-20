use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct Board {
    pub id: i64,
    pub title: String,
    pub owner_id: i64,
    pub is_shared: bool,
}

#[derive(Deserialize)]
pub struct CreateBoard {
    pub title: String,
    #[serde(default)]
    pub is_shared: bool,
}

#[derive(Deserialize, Default)]
pub struct UpdateBoard {
    pub title: Option<String>,
    pub is_shared: Option<bool>,
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
