use crate::models::{CommentWithUser, CreateComment, UpdateComment};
use crate::views::Claims;
use axum::extract::Extension;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

/// Получить все комментарии к карточке
pub async fn get_comments(
    Path(card_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<CommentWithUser>>, (StatusCode, String)> {
    let comments: Vec<CommentWithUser> = sqlx::query_as(
        r#"
        SELECT c.id, c.card_id, c.user_id, c.content, c.created_at, u.username
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.card_id = ?
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(comments))
}

/// Добавить комментарий к карточке
pub async fn create_comment(
    Path(card_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateComment>,
) -> Result<Json<CommentWithUser>, (StatusCode, String)> {
    let comment: CommentWithUser = sqlx::query_as(
        r#"
        INSERT INTO comments (card_id, user_id, content)
        VALUES (?, ?, ?)
        RETURNING id, card_id, user_id, content, created_at, 
            (SELECT username FROM users WHERE id = ?) as username
        "#,
    )
    .bind(card_id)
    .bind(claims.user_id)
    .bind(&payload.content)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(comment))
}

/// Обновить комментарий
pub async fn update_comment(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateComment>,
) -> Result<Json<CommentWithUser>, (StatusCode, String)> {
    // Проверяем, что комментарий принадлежит пользователю
    let current: CommentWithUser = sqlx::query_as(
        r#"
        SELECT c.id, c.card_id, c.user_id, c.content, c.created_at, u.username
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.id = ?
        "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Комментарий не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Проверка: только автор может редактировать
    if current.user_id != claims.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "Только автор может редактировать комментарий".to_string(),
        ));
    }

    let new_content = payload.content.unwrap_or(current.content);

    let updated: CommentWithUser = sqlx::query_as(
        r#"
        UPDATE comments SET content = ? WHERE id = ?
        RETURNING id, card_id, user_id, content, created_at,
            (SELECT username FROM users WHERE id = user_id) as username
        "#,
    )
    .bind(new_content)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

/// Удалить комментарий
pub async fn delete_comment(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Проверяем, что комментарий принадлежит пользователю
    let comment_user_id: Option<i64> =
        sqlx::query_scalar("SELECT user_id FROM comments WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .ok();

    match comment_user_id {
        Some(user_id) if user_id != claims.user_id => {
            return Err((
                StatusCode::FORBIDDEN,
                "Только автор может удалить комментарий".to_string(),
            ));
        }
        None => {
            return Err((StatusCode::NOT_FOUND, "Комментарий не найден".to_string()));
        }
        _ => {}
    }

    let result = sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Комментарий не найден".to_string()))
    } else {
        Ok(Json(()))
    }
}
