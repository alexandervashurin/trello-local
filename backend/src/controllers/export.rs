use axum::{
    extract::{Path, State, Extension},
    http::{StatusCode, HeaderMap},
    Json,
    response::Response,
    body::Body,
};
use sqlx::SqlitePool;
use crate::views::Claims;
use crate::models::{Board, List, Card, Label};
use serde::Serialize;
use chrono::Utc;

/// Данные для экспорта доски
#[derive(Serialize)]
pub struct BoardExport {
    pub id: i64,
    pub title: String,
    pub visibility: String,
    pub is_shared: bool,
    pub exported_at: String,
    pub lists: Vec<ListExport>,
}

#[derive(Serialize)]
pub struct ListExport {
    pub id: i64,
    pub title: String,
    pub position: f64,
    pub cards: Vec<CardExport>,
}

#[derive(Serialize)]
pub struct CardExport {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub done: bool,
    pub due_date: Option<i64>,
    pub labels: Vec<LabelExport>,
}

#[derive(Serialize)]
pub struct LabelExport {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Экспорт доски в JSON
pub async fn export_board_json(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, (StatusCode, String)> {
    let board_data = get_board_export_data(&pool, board_id, claims.user_id).await?;
    
    let json = serde_json::to_string_pretty(&board_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()
        .expect("Content-Type должен быть валидным заголовком"));
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"board_{}_export.json\"", board_id).parse()
            .expect("Content-Disposition должен быть валидным заголовком"),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?)
}

/// Экспорт доски в CSV
pub async fn export_board_csv(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, (StatusCode, String)> {
    let board_data = get_board_export_data(&pool, board_id, claims.user_id).await?;
    
    let mut csv = String::new();
    
    // Заголовок CSV
    csv.push_str("List,Card,Description,Status,Due Date,Labels\n");
    
    // Данные
    for list in &board_data.lists {
        for card in &list.cards {
            let labels = card.labels.iter().map(|l| l.name.clone()).collect::<Vec<_>>().join("; ");
            let due_date = card.due_date
                .map(|ts| chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default())
                .unwrap_or_default();
            let status = if card.done { "Done" } else { "Todo" };
            
            // Экранирование кавычек в CSV
            let title = card.title.replace('"', "\"\"");
            let content = card.content.as_ref().map(|s| s.replace('"', "\"\"")).unwrap_or_default();
            
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                list.title.replace('"', "\"\""),
                title,
                content,
                status,
                due_date,
                labels
            ));
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/csv".parse()
        .expect("Content-Type должен быть валидным заголовком"));
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"board_{}_export.csv\"", board_id).parse()
            .expect("Content-Disposition должен быть валидным заголовком"),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(csv))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?)
}

/// Получить данные для экспорта доски
async fn get_board_export_data(
    pool: &SqlitePool,
    board_id: i64,
    user_id: i64,
) -> Result<BoardExport, (StatusCode, String)> {
    // Проверка прав доступа
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ? AND (owner_id = ? OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = boards.id AND user_id = ?))",
    )
    .bind(board_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Доска не найдена или нет доступа".to_string()))?;

    // Загружаем списки
    let lists: Vec<List> = sqlx::query_as(
        "SELECT id, board_id, title, position FROM lists WHERE board_id = ? ORDER BY position, id",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Загружаем карточки и метки для каждого списка
    let mut list_exports = Vec::new();
    for list in lists {
        let cards: Vec<Card> = sqlx::query_as(
            "SELECT id, list_id, title, content, done, due_date, position FROM cards WHERE list_id = ? ORDER BY position, id",
        )
        .bind(list.id)
        .fetch_all(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut card_exports = Vec::new();
        for card in cards {
            let labels: Vec<Label> = sqlx::query_as(
                "SELECT id, card_id, name, color FROM labels WHERE card_id = ?",
            )
            .bind(card.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            card_exports.push(CardExport {
                id: card.id,
                title: card.title,
                content: card.content,
                done: card.done,
                due_date: card.due_date,
                labels: labels.into_iter().map(|l| LabelExport {
                    id: l.id,
                    name: l.name,
                    color: l.color,
                }).collect(),
            });
        }

        list_exports.push(ListExport {
            id: list.id,
            title: list.title,
            position: list.position,
            cards: card_exports,
        });
    }

    Ok(BoardExport {
        id: board.id,
        title: board.title,
        visibility: board.visibility,
        is_shared: board.is_shared,
        exported_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        lists: list_exports,
    })
}

/// Статистика по доске
#[derive(Serialize)]
pub struct BoardStats {
    pub board_id: i64,
    pub board_title: String,
    pub total_lists: i64,
    pub total_cards: i64,
    pub completed_cards: i64,
    pub pending_cards: i64,
    pub completion_percentage: f64,
    pub total_labels: i64,
    pub cards_with_due_date: i64,
    pub overdue_cards: i64,
}

/// Получить статистику по доске
pub async fn get_board_stats(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<BoardStats>, (StatusCode, String)> {
    // Проверка прав доступа
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ? AND (owner_id = ? OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = boards.id AND user_id = ?))",
    )
    .bind(board_id)
    .bind(claims.user_id)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Доска не найдена или нет доступа".to_string()))?;

    let now = Utc::now().timestamp();

    // Общая статистика
    let stats: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM lists WHERE board_id = ?) as total_lists,
            (SELECT COUNT(*) FROM cards c INNER JOIN lists l ON c.list_id = l.id WHERE l.board_id = ?) as total_cards,
            (SELECT COUNT(*) FROM cards c INNER JOIN lists l ON c.list_id = l.id WHERE l.board_id = ? AND c.done = 1) as completed_cards,
            (SELECT COUNT(*) FROM cards c INNER JOIN lists l ON c.list_id = l.id WHERE l.board_id = ? AND c.done = 0) as pending_cards,
            (SELECT COUNT(*) FROM labels l INNER JOIN cards c ON l.card_id = c.id INNER JOIN lists li ON c.list_id = li.id WHERE li.board_id = ?) as total_labels,
            (SELECT COUNT(*) FROM cards c INNER JOIN lists l ON c.list_id = l.id WHERE l.board_id = ? AND c.due_date IS NOT NULL) as cards_with_due_date
        "#,
    )
    .bind(board_id)
    .bind(board_id)
    .bind(board_id)
    .bind(board_id)
    .bind(board_id)
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Просроченные карточки
    let overdue_cards: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM cards c 
        INNER JOIN lists l ON c.list_id = l.id 
        WHERE l.board_id = ? AND c.due_date IS NOT NULL AND c.due_date < ? AND c.done = 0
        "#,
    )
    .bind(board_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_cards = stats.1;
    let completed_cards = stats.2;
    let completion_percentage = if total_cards > 0 {
        (completed_cards as f64 / total_cards as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(BoardStats {
        board_id: board.id,
        board_title: board.title,
        total_lists: stats.0,
        total_cards: stats.1,
        completed_cards: stats.2,
        pending_cards: stats.3,
        completion_percentage,
        total_labels: stats.4,
        cards_with_due_date: stats.5,
        overdue_cards,
    }))
}
