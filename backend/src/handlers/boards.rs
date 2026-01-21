use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::models::{Board, BoardWithLists, CreateBoard, UpdateBoard};

pub async fn get_boards(
    State(pool): State<sqlx::SqlitePool>,
) -> Result<Json<Vec<BoardWithLists>>, (StatusCode, String)> {
    let boards: Vec<Board> = sqlx::query_as("SELECT * FROM boards ORDER BY id")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let lists = sqlx::query_as::<_, crate::models::List>(
            "SELECT * FROM lists WHERE board_id = ? ORDER BY position",
        )
        .bind(board.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut lists_with_cards = Vec::new();
        for list in lists {
            let cards = sqlx::query_as::<_, crate::models::CardRow>(
                "SELECT * FROM cards WHERE list_id = ? ORDER BY position",
            )
            .bind(list.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let cards: Vec<_> = cards
                .into_iter()
                .map(|c| crate::models::Card {
                    id: c.id,
                    title: c.title,
                    content: c.content,
                    done: c.done,
                })
                .collect();

            lists_with_cards.push(crate::models::ListWithCards {
                id: list.id,
                title: list.title,
                cards,
            });
        }

        result.push(BoardWithLists {
            id: board.id,
            title: board.title,
            lists: lists_with_cards,
        });
    }

    Ok(Json(result))
}

pub async fn create_board(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let board = sqlx::query_as::<_, Board>(
        "INSERT INTO boards (title) VALUES (?) RETURNING id, title",
    )
    .bind(&payload.title)
    .fetch_one(&pool)
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
        "UPDATE boards SET title = ? WHERE id = ? RETURNING id, title",
    )
    .bind(&payload.title)
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