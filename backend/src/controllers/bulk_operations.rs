use crate::controllers::cards::get_board_id_by_card_id;
use crate::views::Claims;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

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
    State(pool): State<SqlitePool>,
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
            "SELECT 1 FROM boards WHERE id = ? AND (owner_id = ? OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ?))",
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
        let result = sqlx::query("UPDATE cards SET list_id = ? WHERE id = ?")
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
    State(pool): State<SqlitePool>,
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
            "SELECT 1 FROM boards WHERE id = ? AND (owner_id = ? OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ?))",
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
        let mut updates = Vec::new();
        if let Some(done) = payload.done {
            updates.push(format!("done = {}", if done { 1 } else { 0 }));
        }
        if let Some(due_date) = payload.due_date {
            updates.push(format!("due_date = {}", due_date));
        }

        if updates.is_empty() {
            failed += 1;
            errors.push(format!("Нет полей для обновления карточки {}", card_id));
            continue;
        }

        let query = format!(
            "UPDATE cards SET {} WHERE id = {}",
            updates.join(", "),
            card_id
        );

        match sqlx::query(&query).execute(&pool).await {
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
    State(pool): State<SqlitePool>,
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
            "SELECT 1 FROM boards WHERE id = ? AND (owner_id = ? OR visibility = 'public' OR EXISTS (SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ?))",
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
        let result = sqlx::query("DELETE FROM cards WHERE id = ?")
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
