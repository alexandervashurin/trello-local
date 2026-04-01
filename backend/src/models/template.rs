use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Шаблон доски
#[derive(Serialize, FromRow, Clone, Debug)]
pub struct BoardTemplate {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: i64,
}

/// Список в шаблоне
#[derive(Serialize, FromRow, Clone, Debug)]
pub struct TemplateList {
    pub id: i64,
    pub template_id: i64,
    pub title: String,
    pub position: i64,
}

/// Карточка в шаблоне
#[derive(Serialize, FromRow, Clone, Debug)]
pub struct TemplateCard {
    pub id: i64,
    pub list_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub position: i64,
}

/// Данные для создания шаблона
#[derive(Deserialize)]
pub struct CreateBoardTemplate {
    pub title: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

/// Результат применения шаблона
#[derive(Serialize)]
pub struct TemplateApplyResult {
    pub board_id: i64,
    pub board_title: String,
    pub lists_created: i64,
    pub cards_created: i64,
}
