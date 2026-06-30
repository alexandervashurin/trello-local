use crate::controllers::cards::get_board_id_by_card_id;
use crate::views::Claims;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;

/// Запрос массового перемещения
#[derive(Deserialize)]
pub struct BulkMoveRequest {
    pub card_ids: Vec<i64>,
    pub list_id: i64,
}

/// Запрос массового обновления
#[derive(Deserialize)]
pub struct BulkUpdateRequest {
    pub card_ids: Vec<i64>,
    pub done: Option<bool>,
    pub due_date: Option<i64>,
}

/// Запрос массового удаления
#[derive(Deserialize)]
pub struct BulkDeleteRequest {
    pub card_ids: Vec<i64>,
}

/// Результат массовой операции
#[derive(serde::Serialize)]
pub struct BulkOperationResult {
    pub success: bool,
    pub processed_count: i64,
    pub failed_count: i64,
    pub errors: Vec<String>,
}

/// Массовое перемещение карточек в другой список
pub async fn bulk_move_cards(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<BulkMoveRequest>,
) -> Result<Json<BulkOperationResult>, (StatusCode, String)> {
    let mut processed = 0i64;
    let mut failed = 0i64;
    let mut errors = Vec::new();

    for card_id in payload.card_ids {
        // Проверка прав доступа
        let board_id = match get_board_id_by_card_id(&pool, card_id).await {
            Some(id) => id,
            None => {
                failed += 1;
                errors.push(format!("Карточка {} не найдена", card_id));
                continue;
            }
        };

        // Проверка доступа к доске
        let has_access: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM boards WHERE id = $1 AND (owner_id = $2 OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = $3 AND user_id = $4))",
        )
        .bind(board_id)
        .bind(claims.user_id)
        .bind(board_id)
        .bind(claims.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if has_access.is_none() {
            failed += 1;
            errors.push(format!("Нет доступа к карточке {}", card_id));
            continue;
        }

        // Перемещение карточки
        let result = sqlx::query("UPDATE cards SET list_id = $1 WHERE id = $2")
            .bind(payload.list_id)
            .bind(card_id)
            .execute(&pool)
            .await;

        match result {
            Ok(_) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Ошибка перемещения карточки {}: {}", card_id, e));
            }
        }
    }

    Ok(Json(BulkOperationResult {
        success: failed == 0,
        processed_count: processed,
        failed_count: failed,
        errors,
    }))
}

/// Массовое обновление карточек (статус, дедлайн)
pub async fn bulk_update_cards(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<BulkUpdateRequest>,
) -> Result<Json<BulkOperationResult>, (StatusCode, String)> {
    let mut processed = 0i64;
    let mut failed = 0i64;
    let mut errors = Vec::new();

    for card_id in payload.card_ids {
        // Проверка прав доступа
        let board_id = match get_board_id_by_card_id(&pool, card_id).await {
            Some(id) => id,
            None => {
                failed += 1;
                errors.push(format!("Карточка {} не найдена", card_id));
                continue;
            }
        };

        // Проверка доступа к доске
        let has_access: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM boards WHERE id = $1 AND (owner_id = $2 OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = $3 AND user_id = $4))",
        )
        .bind(board_id)
        .bind(claims.user_id)
        .bind(board_id)
        .bind(claims.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if has_access.is_none() {
            failed += 1;
            errors.push(format!("Нет доступа к карточке {}", card_id));
            continue;
        }

        // Построение динамического запроса
        let mut set_clauses = Vec::new();
        let mut param_idx = 1;

        if let Some(done) = payload.done {
            set_clauses.push(format!("done = ${}", param_idx));
            param_idx += 1;
        }
        if let Some(due_date) = payload.due_date {
            set_clauses.push(format!("due_date = ${}", param_idx));
            param_idx += 1;
        }

        if set_clauses.is_empty() {
            failed += 1;
            errors.push(format!("Нет полей для обновления карточки {}", card_id));
            continue;
        }

        let query = format!(
            "UPDATE cards SET {} WHERE id = ${}",
            set_clauses.join(", "),
            param_idx,
        );

        let mut q = sqlx::query(&query);
        if let Some(done) = payload.done {
            q = q.bind(done);
        }
        if let Some(due_date) = payload.due_date {
            q = q.bind(due_date);
        }
        q = q.bind(card_id);

        match q.execute(&pool).await {
            Ok(_) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Ошибка обновления карточки {}: {}", card_id, e));
            }
        }
    }

    Ok(Json(BulkOperationResult {
        success: failed == 0,
        processed_count: processed,
        failed_count: failed,
        errors,
    }))
}

/// Массовое удаление карточек
pub async fn bulk_delete_cards(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<BulkDeleteRequest>,
) -> Result<Json<BulkOperationResult>, (StatusCode, String)> {
    let mut processed = 0i64;
    let mut failed = 0i64;
    let mut errors = Vec::new();

    for card_id in payload.card_ids {
        // Проверка прав доступа
        let board_id = match get_board_id_by_card_id(&pool, card_id).await {
            Some(id) => id,
            None => {
                failed += 1;
                errors.push(format!("Карточка {} не найдена", card_id));
                continue;
            }
        };

        // Проверка доступа к доске
        let has_access: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM boards WHERE id = $1 AND (owner_id = $2 OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = $3 AND user_id = $4))",
        )
        .bind(board_id)
        .bind(claims.user_id)
        .bind(board_id)
        .bind(claims.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if has_access.is_none() {
            failed += 1;
            errors.push(format!("Нет доступа к карточке {}", card_id));
            continue;
        }

        // Удаление карточки
        let result = sqlx::query("DELETE FROM cards WHERE id = $1")
            .bind(card_id)
            .execute(&pool)
            .await;

        match result {
            Ok(_) => processed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Ошибка удаления карточки {}: {}", card_id, e));
            }
        }
    }

    Ok(Json(BulkOperationResult {
        success: failed == 0,
        processed_count: processed,
        failed_count: failed,
        errors,
    }))
}
