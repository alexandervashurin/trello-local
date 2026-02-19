use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{Board, BoardWithMembers, User, CreateBoard, UpdateBoard, AddBoardMember};

pub async fn get_boards(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<Vec<BoardWithMembers>>, (StatusCode, String)> {
    let boards: Vec<Board> = sqlx::query_as("SELECT * FROM boards ORDER BY id")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let members: Vec<User> = sqlx::query_as(
            "SELECT u.id, u.username, u.created_at 
             FROM users u 
             INNER JOIN board_members bm ON u.id = bm.user_id 
             WHERE bm.board_id = ?",
        )
        .bind(board.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        result.push(BoardWithMembers {
            id: board.id,
            title: board.title,
            owner_id: board.owner_id,
            is_shared: board.is_shared,
            members,
        });
    }

    Ok(Json(result))
}

pub async fn get_boards_for_user(
    Path(user_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<Vec<BoardWithMembers>>, (StatusCode, String)> {
    let boards: Vec<Board> = sqlx::query_as(
        "SELECT * FROM boards 
         WHERE owner_id = ? OR is_shared = 1 
         ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let members: Vec<User> = sqlx::query_as(
            "SELECT u.id, u.username, u.created_at 
             FROM users u 
             INNER JOIN board_members bm ON u.id = bm.user_id 
             WHERE bm.board_id = ?",
        )
        .bind(board.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        result.push(BoardWithMembers {
            id: board.id,
            title: board.title,
            owner_id: board.owner_id,
            is_shared: board.is_shared,
            members,
        });
    }

    Ok(Json(result))
}

pub async fn create_board(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let owner_id = 1; // По умолчанию первый пользователь (можно изменить на параметр)
    
    let board = sqlx::query_as::<_, Board>(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES (?, ?, ?) RETURNING id, title, owner_id, is_shared",
    )
    .bind(&payload.title)
    .bind(owner_id)
    .bind(payload.is_shared)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Добавляем владельца как участника доски
    sqlx::query(
        "INSERT OR IGNORE INTO board_members (board_id, user_id, role) VALUES (?, ?, 'owner')",
    )
    .bind(board.id)
    .bind(owner_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(board))
}

pub async fn update_board(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<UpdateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let board = sqlx::query_as::<_, Board>(
        "UPDATE boards SET 
         title = COALESCE(?, title), 
         is_shared = COALESCE(?, is_shared) 
         WHERE id = ? 
         RETURNING id, title, owner_id, is_shared",
    )
    .bind(payload.title)
    .bind(payload.is_shared)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Доска не найдена".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(board))
}

pub async fn delete_board(
    Path(id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM boards WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Доска не найдена".to_string()))
    } else {
        Ok(Json(()))
    }
}

pub async fn get_board_members(
    Path(board_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let members: Vec<User> = sqlx::query_as(
        "SELECT u.id, u.username, u.created_at 
         FROM users u 
         INNER JOIN board_members bm ON u.id = bm.user_id 
         WHERE bm.board_id = ?",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(members))
}

pub async fn add_board_member(
    Path(board_id): Path<i64>,
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<AddBoardMember>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT OR REPLACE INTO board_members (board_id, user_id, role) VALUES (?, ?, ?)",
    )
    .bind(board_id)
    .bind(payload.user_id)
    .bind(&payload.role)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Доска или пользователь не найдены".to_string()))
    } else {
        Ok(Json(()))
    }
}

pub async fn remove_board_member(
    Path((board_id, user_id)): Path<(i64, i64)>,
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query(
        "DELETE FROM board_members WHERE board_id = ? AND user_id = ?",
    )
    .bind(board_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Участник не найден".to_string()))
    } else {
        Ok(Json(()))
    }
}