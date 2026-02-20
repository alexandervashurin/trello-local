use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Card {
    pub id: i64,
    pub list_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub done: bool,
    pub due_date: Option<i64>,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Label {
    pub id: i64,
    pub card_id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Attachment {
    pub id: i64,
    pub card_id: i64,
    pub user_id: i64,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct ActivityLog {
    pub id: i64,
    pub board_id: i64,
    pub user_id: Option<i64>,
    pub action_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub description: String,
    pub metadata: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Checklist {
    pub id: i64,
    pub card_id: i64,
    pub title: String,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct ChecklistItem {
    pub id: i64,
    pub checklist_id: i64,
    pub title: String,
    pub done: bool,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct CardAssignee {
    pub card_id: i64,
    pub user_id: i64,
    pub assigned_at: i64,
    pub assigned_by: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CardAssigneeWithUser {
    pub card_id: i64,
    pub user_id: i64,
    pub username: String,
    pub assigned_at: i64,
    pub assigned_by: i64,
}

#[derive(Deserialize)]
pub struct CreateCard {
    pub title: String,
    pub content: Option<String>,
    pub due_date: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct UpdateCard {
    pub title: Option<String>,
    pub content: Option<String>,
    pub list_id: Option<i64>,
    pub position: Option<f64>,
    pub done: Option<bool>,
    pub due_date: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateLabel {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateLabel {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateChecklist {
    pub title: String,
}

#[derive(Deserialize)]
pub struct CreateChecklistItem {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateChecklistItem {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(Deserialize)]
pub struct AddCardAssignee {
    pub user_id: i64,
}
