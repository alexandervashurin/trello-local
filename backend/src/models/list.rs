use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct List {
    pub id: i64,
    pub board_id: i64,
    pub title: String,
    pub position: f64,
}

#[derive(Deserialize)]
pub struct CreateList {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateList {
    pub title: String,
}
