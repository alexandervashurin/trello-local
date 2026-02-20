use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{Board, CreateBoard, UpdateBoard, AddBoardMember};
use crate::views::{BoardView, ListView};
use crate::models::{User, List, Card};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetBoardsQuery {
    search: Option<String>,
}

/// Получить все доски (с опциональным поиском)
pub async fn get_boards(
    State(pool): State<SqlitePool>,
    query: Query<GetBoardsQuery>,
) -> Result<Json<Vec<BoardView>>, (StatusCode, String)> {
    let boards: Vec<Board> = if let Some(search) = &query.search {
        sqlx::query_as("SELECT * FROM boards WHERE title LIKE ? ORDER BY id")
            .bind(format!("%{}%", search))
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT * FROM boards ORDER BY id")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let board_view = load_board_details(&pool, board).await?;
        result.push(board_view);
    }

    Ok(Json(result))
}

/// Получить доски для пользователя
pub async fn get_boards_for_user(
    Path(user_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<BoardView>>, (StatusCode, String)> {
    let boards: Vec<Board> = sqlx::query_as(
        "SELECT * FROM boards WHERE owner_id = ? OR is_shared = 1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let board_view = load_board_details(&pool, board).await?;
        result.push(board_view);
    }

    Ok(Json(result))
}

/// Создать доску
pub async fn create_board(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let owner_id = 1; // По умолчанию первый пользователь

    let board: Board = sqlx::query_as::<_, Board>(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES (?, ?, ?) RETURNING id, title, owner_id, is_shared",
    )
    .bind(&payload.title)
    .bind(owner_id)
    .bind(payload.is_shared)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Добавляем владельца как участника
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

/// Обновить доску
pub async fn update_board(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let board: Board = sqlx::query_as::<_, Board>(
        "UPDATE boards SET title = COALESCE(?, title), is_shared = COALESCE(?, is_shared) WHERE id = ? RETURNING id, title, owner_id, is_shared",
    )
    .bind(payload.title)
    .bind(payload.is_shared)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Доска не найдена".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(board))
}

/// Удалить доску
pub async fn delete_board(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
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

/// Получить участников доски
pub async fn get_board_members(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let members: Vec<User> = sqlx::query_as(
        "SELECT u.id, u.username, u.created_at FROM users u INNER JOIN board_members bm ON u.id = bm.user_id WHERE bm.board_id = ?",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(members))
}

/// Добавить участника в доску
pub async fn add_board_member(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
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

/// Удалить участника из доски
pub async fn remove_board_member(
    Path((board_id, user_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM board_members WHERE board_id = ? AND user_id = ?")
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

/// Вспомогательная функция для загрузки полной информации о доске
async fn load_board_details(
    pool: &SqlitePool,
    board: Board,
) -> Result<BoardView, (StatusCode, String)> {
    // Загружаем участников
    let members: Vec<User> = sqlx::query_as(
        "SELECT u.id, u.username, u.created_at FROM users u INNER JOIN board_members bm ON u.id = bm.user_id WHERE bm.board_id = ?",
    )
    .bind(board.id)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Загружаем списки
    let lists_rows: Vec<List> = sqlx::query_as(
        "SELECT id, board_id, title, position FROM lists WHERE board_id = ? ORDER BY position, id",
    )
    .bind(board.id)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Загружаем карточки для каждого списка
    let mut lists = Vec::new();
    for list in lists_rows {
        let cards: Vec<Card> = sqlx::query_as(
            "SELECT id, list_id, title, content, done FROM cards WHERE list_id = ? ORDER BY position, id",
        )
        .bind(list.id)
        .fetch_all(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        lists.push(ListView::from_list(list).with_cards(cards));
    }

    Ok(BoardView::from_board(board)
        .with_members(members)
        .with_lists(lists))
}
