use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::User;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

pub async fn get_users(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY id")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

pub async fn create_user(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, created_at) VALUES (?, strftime('%s', 'now')) RETURNING id, username, created_at",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}

pub async fn get_user(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Пользователь не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}
