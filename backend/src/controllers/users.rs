use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{User, CreateUser};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetUserQuery {
    username: Option<String>,
}

/// Получить всех пользователей (с опциональным поиском по имени)
pub async fn get_users(
    State(pool): State<SqlitePool>,
    query: Query<GetUserQuery>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users: Vec<User> = if let Some(username) = &query.username {
        sqlx::query_as("SELECT id, username, created_at FROM users WHERE username = ? ORDER BY id")
            .bind(username)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT id, username, created_at FROM users ORDER BY id")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

/// Создать пользователя
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, created_at) VALUES (?, strftime('%s', 'now')) RETURNING id, username, created_at",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}

/// Получить пользователя по ID
pub async fn get_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Пользователь не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}
