use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Board {
    pub id: i64,
    pub title: String,
    pub owner_id: i64,
    pub is_shared: bool,
}

#[derive(Serialize)]
pub struct BoardWithMembers {
    pub id: i64,
    pub title: String,
    pub owner_id: i64,
    pub is_shared: bool,
    pub members: Vec<User>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct List {
    pub id: i64,
    pub board_id: i64,
    pub title: String,
    pub position: f64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CardRow {
    pub id: i64,
    pub list_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub position: f64,
    pub done: bool,
}

#[derive(Serialize)]
pub struct Card {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct CreateBoard {
    pub title: String,
    #[serde(default)]
    pub is_shared: bool,
}

#[derive(Deserialize)]
pub struct UpdateBoard {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
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

#[derive(Deserialize)]
pub struct CreateList { pub title: String }
#[derive(Deserialize)]
pub struct UpdateList { pub title: String }

#[derive(Deserialize)]
pub struct CreateCard { pub title: String, pub content: Option<String> }

#[derive(Deserialize)]
pub struct UpdateCard {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub list_id: Option<i64>,
    #[serde(default)]
    pub position: Option<f64>,
    #[serde(default)]
    pub done: Option<bool>,  // ← теперь Option<bool>
}