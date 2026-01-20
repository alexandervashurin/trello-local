use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{List, CreateList, UpdateList};

pub async fn create_list(
    Path(board_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateList>,
) -> Result<Json<List>, (StatusCode, String)> {
    let list = sqlx::query_as::<_, List>(
        "INSERT INTO lists (board_id, title) VALUES (?, ?) RETURNING id, board_id, title, position",
    )
    .bind(board_id)
    .bind(&payload.title)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(list))
}

pub async fn update_list(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<UpdateList>,
) -> Result<Json<List>, (StatusCode, String)> {
    let list = sqlx::query_as::<_, List>(
        "UPDATE lists SET title = ? WHERE id = ? RETURNING id, board_id, title, position",
    )
    .bind(&payload.title)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Список не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(list))
}

pub async fn delete_list(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM lists WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Список не найден".to_string()))
    } else {
        Ok(Json(()))
    }
}