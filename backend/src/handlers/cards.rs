use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{Card, CreateCard, UpdateCard};

pub async fn create_card(
    Path(list_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    let card = sqlx::query_as::<_, Card>(
        "INSERT INTO cards (list_id, title, content, done) VALUES (?, ?, ?, 0) RETURNING id, list_id, title, content, done",
    )
    .bind(list_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(card))
}

pub async fn update_card(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<UpdateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    // 1. Получаем текущую карточку из БД
    let current: Card = sqlx::query_as::<_, Card>("SELECT id, list_id, title, content, done FROM cards WHERE id = ?")
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

    // 2. Формируем новые значения: используем payload, если указано, иначе — старые
    let new_title = payload.title.unwrap_or(current.title);
    let new_content = payload.content.or(current.content);
    let new_list_id = payload.list_id.unwrap_or(current.list_id);
    let new_done = payload.done.unwrap_or(current.done);

    // 3. Обновляем запись в БД и возвращаем изменённую карточку
    let updated: Card = sqlx::query_as(
        "UPDATE cards SET title = ?, content = ?, list_id = ?, done = ? WHERE id = ? RETURNING id, list_id, title, content, done"
    )
    .bind(new_title)
    .bind(new_content)
    .bind(new_list_id)
    .bind(new_done)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
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