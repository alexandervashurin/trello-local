use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct Card {
    pub id: i64,
    pub list_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct CreateCard {
    pub title: String,
    pub content: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateCard {
    pub title: Option<String>,
    pub content: Option<String>,
    pub list_id: Option<i64>,
    pub position: Option<f64>,
    pub done: Option<bool>,
}
