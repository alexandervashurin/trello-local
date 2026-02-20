use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{List, CreateList, UpdateList};
use crate::controllers::cards::log_activity;

/// Создать список в доске
pub async fn create_list(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateList>,
) -> Result<Json<List>, (StatusCode, String)> {
    let list: List = sqlx::query_as::<_, List>(
        "INSERT INTO lists (board_id, title) VALUES (?, ?) RETURNING id, board_id, title, position",
    )
    .bind(board_id)
    .bind(&payload.title)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    let _ = log_activity(
        &pool,
        board_id,
        None,
        "create",
        Some("list"),
        Some(list.id),
        &format!("Создан список \"{}\"", &payload.title),
        None,
    ).await;

    Ok(Json(list))
}

/// Обновить список
pub async fn update_list(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateList>,
) -> Result<Json<List>, (StatusCode, String)> {
    // Получаем текущий список и board_id
    let current: List = sqlx::query_as::<_, List>(
        "SELECT id, board_id, title, position FROM lists WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Список не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let board_id = current.board_id;

    let list: List = sqlx::query_as::<_, List>(
        "UPDATE lists SET title = ? WHERE id = ? RETURNING id, board_id, title, position",
    )
    .bind(&payload.title)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Список не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Логирование
    if payload.title != current.title {
        let _ = log_activity(
            &pool,
            board_id,
            None,
            "update",
            Some("list"),
            Some(id),
            &format!("Список \"{}\": название → \"{}\"", current.title, payload.title),
            None,
        ).await;
    }

    Ok(Json(list))
}

/// Удалить список
pub async fn delete_list(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Получаем информацию о списке перед удалением
    let list: Option<(String, i64)> = sqlx::query_as(
        "SELECT title, board_id FROM lists WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (title, board_id) = list.ok_or((StatusCode::NOT_FOUND, "Список не найден".to_string()))?;

    let result = sqlx::query("DELETE FROM lists WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Список не найден".to_string()))
    } else {
        // Логирование
        let _ = log_activity(
            &pool,
            board_id,
            None,
            "delete",
            Some("list"),
            Some(id),
            &format!("Удалён список \"{}\"", title),
            None,
        ).await;
        Ok(Json(()))
    }
}
