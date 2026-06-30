use crate::controllers::cards::{get_board_id_by_card_id, log_activity};
use crate::models::{
    AddCardAssignee, CardAssigneeWithUser, Checklist, ChecklistItem, CreateChecklist,
    CreateChecklistItem, UpdateChecklistItem,
};
use crate::views::Claims;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

// === Checklist Functions ===

/// Получить чек-листы карточки
pub async fn get_card_checklists(
    Path(card_id): Path<i64>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ChecklistWithItems>>, (StatusCode, String)> {
    let checklists: Vec<Checklist> = sqlx::query_as(
        "SELECT id, card_id, title, position, created_at FROM checklists WHERE card_id = $1 ORDER BY position, id",
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for checklist in checklists {
        let items: Vec<ChecklistItem> = sqlx::query_as(
            "SELECT id, checklist_id, title, done, position, created_at FROM checklist_items WHERE checklist_id = $1 ORDER BY position, id",
        )
        .bind(checklist.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ChecklistWithItems {
            id: checklist.id,
            card_id: checklist.card_id,
            title: checklist.title,
            position: checklist.position,
            created_at: checklist.created_at,
            items,
        });
    }

    Ok(Json(result))
}

/// Создать чек-лист
pub async fn create_checklist(
    Path(card_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateChecklist>,
) -> Result<Json<Checklist>, (StatusCode, String)> {
    let checklist: Checklist = sqlx::query_as::<_, Checklist>(
        "INSERT INTO checklists (card_id, title, position) VALUES ($1, $2, (SELECT COALESCE(MAX(position), -1) + 1 FROM checklists WHERE card_id = $3)) RETURNING id, card_id, title, position, created_at",
    )
    .bind(card_id)
    .bind(&payload.title)
    .bind(card_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
        let _ = log_activity(
            &pool,
            board_id,
            Some(claims.user_id),
            "create",
            Some("checklist"),
            Some(checklist.id),
            &format!("Добавлен чек-лист \"{}\"", &payload.title),
            None,
        )
        .await;
    }

    Ok(Json(checklist))
}

/// Удалить чек-лист
pub async fn delete_checklist(
    Path((card_id, checklist_id)): Path<(i64, i64)>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let checklist_title: Option<(String,)> =
        sqlx::query_as("SELECT title FROM checklists WHERE id = $1 AND card_id = $2")
            .bind(checklist_id)
            .bind(card_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let title = checklist_title
        .map(|t| t.0)
        .unwrap_or_else(|| "неизвестно".to_string());

    let result = sqlx::query("DELETE FROM checklists WHERE id = $1 AND card_id = $2")
        .bind(checklist_id)
        .bind(card_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Чек-лист не найден".to_string()))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let _ = log_activity(
                &pool,
                board_id,
                Some(claims.user_id),
                "delete",
                Some("checklist"),
                Some(checklist_id),
                &format!("Удалён чек-лист \"{}\"", title),
                None,
            )
            .await;
        }
        Ok(Json(()))
    }
}

/// Добавить элемент в чек-лист
pub async fn create_checklist_item(
    Path((card_id, checklist_id)): Path<(i64, i64)>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateChecklistItem>,
) -> Result<Json<ChecklistItem>, (StatusCode, String)> {
    let item: ChecklistItem = sqlx::query_as::<_, ChecklistItem>(
        "INSERT INTO checklist_items (checklist_id, title, position) VALUES ($1, $2, (SELECT COALESCE(MAX(position), -1) + 1 FROM checklist_items WHERE checklist_id = $3)) RETURNING id, checklist_id, title, done, position, created_at",
    )
    .bind(checklist_id)
    .bind(&payload.title)
    .bind(checklist_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
        let _ = log_activity(
            &pool,
            board_id,
            Some(claims.user_id),
            "create",
            Some("checklist_item"),
            Some(item.id),
            &format!("Добавлен элемент \"{}\" в чек-лист", &payload.title),
            None,
        )
        .await;
    }

    Ok(Json(item))
}

/// Обновить элемент чек-листа
pub async fn update_checklist_item(
    Path((card_id, checklist_id, item_id)): Path<(i64, i64, i64)>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateChecklistItem>,
) -> Result<Json<ChecklistItem>, (StatusCode, String)> {
    let current: ChecklistItem = sqlx::query_as::<_, ChecklistItem>(
        "SELECT id, checklist_id, title, done, position, created_at FROM checklist_items WHERE id = $1 AND checklist_id = $2",
    )
    .bind(item_id)
    .bind(checklist_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Элемент чек-листа не найден".to_string()))?;

    let current_title = current.title.clone();
    let new_title = payload.title.clone().unwrap_or(current.title.clone());
    let new_done = payload.done.unwrap_or(current.done);

    let updated: ChecklistItem = sqlx::query_as(
        "UPDATE checklist_items SET title = $1, done = $2 WHERE id = $3 RETURNING id, checklist_id, title, done, position, created_at",
    )
    .bind(&new_title)
    .bind(new_done)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
        let mut changes = Vec::new();
        if payload.title.is_some() && payload.title != Some(current_title.clone()) {
            changes.push(format!("название → \"{}\"", new_title));
        }
        if payload.done.is_some() && payload.done != Some(current.done) {
            changes.push(if new_done {
                "отмечен выполненным".to_string()
            } else {
                "возвращён в работу".to_string()
            });
        }
        if !changes.is_empty() {
            let _ = log_activity(
                &pool,
                board_id,
                Some(claims.user_id),
                "update",
                Some("checklist_item"),
                Some(item_id),
                &format!(
                    "Элемент чек-листа \"{}\": {}",
                    current_title,
                    changes.join(", ")
                ),
                None,
            )
            .await;
        }
    }

    Ok(Json(updated))
}

/// Удалить элемент чек-листа
pub async fn delete_checklist_item(
    Path((card_id, checklist_id, item_id)): Path<(i64, i64, i64)>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let item_title: Option<(String,)> =
        sqlx::query_as("SELECT title FROM checklist_items WHERE id = $1 AND checklist_id = $2")
            .bind(item_id)
            .bind(checklist_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let title = item_title
        .map(|t| t.0)
        .unwrap_or_else(|| "неизвестно".to_string());

    let result = sqlx::query("DELETE FROM checklist_items WHERE id = $1 AND checklist_id = $2")
        .bind(item_id)
        .bind(checklist_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            "Элемент чек-листа не найден".to_string(),
        ))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let _ = log_activity(
                &pool,
                board_id,
                Some(claims.user_id),
                "delete",
                Some("checklist_item"),
                Some(item_id),
                &format!("Удалён элемент чек-листа \"{}\"", title),
                None,
            )
            .await;
        }
        Ok(Json(()))
    }
}

// === Assignee Functions ===

/// Получить исполнителей карточки
pub async fn get_card_assignees(
    Path(card_id): Path<i64>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<CardAssigneeWithUser>>, (StatusCode, String)> {
    let rows: Vec<(i64, i64, i64, i64, String)> = sqlx::query_as(
        "SELECT ca.card_id, ca.user_id, ca.assigned_at, ca.assigned_by, u.username FROM card_assignees ca INNER JOIN users u ON ca.user_id = u.id WHERE ca.card_id = $1",
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let assignees = rows
        .into_iter()
        .map(|r| CardAssigneeWithUser {
            card_id: r.0,
            user_id: r.1,
            assigned_at: r.2,
            assigned_by: r.3,
            username: r.4,
        })
        .collect();

    Ok(Json(assignees))
}

/// Добавить исполнителя на карточку
pub async fn add_card_assignee(
    Path(card_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddCardAssignee>,
) -> Result<Json<()>, (StatusCode, String)> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() as i64;

    let result = sqlx::query(
        "INSERT INTO card_assignees (card_id, user_id, assigned_at, assigned_by) VALUES ($1, $2, $3, $4) ON CONFLICT (card_id, user_id) DO UPDATE SET assigned_at = EXCLUDED.assigned_at, assigned_by = EXCLUDED.assigned_by",
    )
    .bind(card_id)
    .bind(payload.user_id)
    .bind(now)
    .bind(claims.user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            "Карточка или пользователь не найдены".to_string(),
        ))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let username: Option<(String,)> =
                sqlx::query_as("SELECT username FROM users WHERE id = $1")
                    .bind(payload.user_id)
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten();

            let user_display = username
                .map(|u| u.0)
                .unwrap_or_else(|| format!("user_{}", payload.user_id));

            let _ = log_activity(
                &pool,
                board_id,
                Some(claims.user_id),
                "assign",
                Some("card_assignee"),
                Some(payload.user_id),
                &format!("Назначен исполнитель \"{}\"", user_display),
                None,
            )
            .await;
        }
        Ok(Json(()))
    }
}

/// Удалить исполнителя с карточки
pub async fn remove_card_assignee(
    Path((card_id, user_id)): Path<(i64, i64)>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM card_assignees WHERE card_id = $1 AND user_id = $2")
        .bind(card_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Исполнитель не найден".to_string()))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let _ = log_activity(
                &pool,
                board_id,
                Some(claims.user_id),
                "unassign",
                Some("card_assignee"),
                Some(user_id),
                &format!("Удалён исполнитель (user_id: {})", user_id),
                None,
            )
            .await;
        }
        Ok(Json(()))
    }
}

// === Helper Structs ===

#[derive(serde::Serialize, Clone, Debug)]
pub struct ChecklistWithItems {
    pub id: i64,
    pub card_id: i64,
    pub title: String,
    pub position: i64,
    pub created_at: i64,
    pub items: Vec<ChecklistItem>,
}
