use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{CardRow, Card, CreateCard, UpdateCard};

pub async fn create_card(
    Path(list_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    let card_row = sqlx::query_as::<_, CardRow>(
        "INSERT INTO cards (list_id, title, content) VALUES (?, ?, ?) RETURNING id, list_id, title, content, position",
    )
    .bind(list_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let card = Card {
        id: card_row.id,
        title: card_row.title,
        content: card_row.content,
    };

    Ok(Json(card))
}

pub async fn update_card(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<UpdateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    let current: CardRow = sqlx::query_as::<_, CardRow>("SELECT * FROM cards WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("no rows returned") {
                (StatusCode::NOT_FOUND, "Карточка не найдена".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    let new_title = if payload.title.is_empty() { current.title } else { payload.title };
    let new_content = payload.content.or(current.content);
    let new_list_id = payload.list_id.unwrap_or(current.list_id);
    let new_position = payload.position.unwrap_or(current.position);
    let new_done = payload.done.unwrap_or(current.done);  // ← новое

    let updated: CardRow = sqlx::query_as(
        "UPDATE cards SET title = ?, content = ?, list_id = ?, position = ?, done = ? WHERE id = ? RETURNING *"
    )
    .bind(new_title)
    .bind(new_content)
    .bind(new_list_id)
    .bind(new_position)
    .bind(new_done)  // ← новое
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let card = Card {
        id: updated.id,
        title: updated.title,
        content: updated.content,
        done: updated.done,  // ← новое
    };

    Ok(Json(card))
}

pub async fn delete_card(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM cards WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Карточка не найдена".to_string()))
    } else {
        Ok(Json(()))
    }
}