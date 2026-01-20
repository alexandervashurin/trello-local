use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct Board {
    pub id: i64,
    pub title: String,
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
}

#[derive(Serialize)]
pub struct Card {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct BoardWithLists {
    pub id: i64,
    pub title: String,
    pub lists: Vec<ListWithCards>,
}

#[derive(Serialize)]
pub struct ListWithCards {
    pub id: i64,
    pub title: String,
    pub cards: Vec<Card>,
}

#[derive(Deserialize)]
pub struct CreateBoard { pub title: String }
#[derive(Deserialize)]
pub struct UpdateBoard { pub title: String }

#[derive(Deserialize)]
pub struct CreateList { pub title: String }
#[derive(Deserialize)]
pub struct UpdateList { pub title: String }

#[derive(Deserialize)]
pub struct CreateCard { pub title: String, pub content: Option<String> }
#[derive(Deserialize)]
pub struct UpdateCard {
    pub title: String,
    pub content: Option<String>,
    #[serde(default)]
    pub list_id: Option<i64>,
    #[serde(default)]
    pub position: Option<f64>,
}