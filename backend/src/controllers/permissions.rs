// backend/src/controllers/permissions.rs
use crate::models::{BoardPermission, UpdateRolePermissions};
use crate::views::Claims;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sqlx::SqlitePool;

/// Получить все права для доски
pub async fn get_board_permissions(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<BoardPermission>>, (StatusCode, String)> {
    // Проверка прав: только owner или admin могут просматривать права
    let board: crate::models::Board =
        sqlx::query_as::<_, crate::models::Board>("SELECT * FROM boards WHERE id = ?")
            .bind(board_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;

    let has_permission = claims.user_id == board.owner_id
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"])
            .await
            .unwrap_or(false);

    if !has_permission {
        return Err((
            StatusCode::FORBIDDEN,
            "Нет прав на просмотр настроек прав доступа".to_string(),
        ));
    }

    let permissions =
        sqlx::query_as::<_, BoardPermission>("SELECT * FROM board_permissions WHERE board_id = ?")
            .bind(board_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(permissions))
}

/// Обновить права для роли
pub async fn update_role_permissions(
    Path((board_id, role)): Path<(i64, String)>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateRolePermissions>,
) -> Result<Json<BoardPermission>, (StatusCode, String)> {
    // Проверка прав: только owner может изменять права
    let board: crate::models::Board =
        sqlx::query_as::<_, crate::models::Board>("SELECT * FROM boards WHERE id = ?")
            .bind(board_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;

    if claims.user_id != board.owner_id {
        return Err((
            StatusCode::FORBIDDEN,
            "Только владелец доски может изменять права доступа".to_string(),
        ));
    }

    // Нельзя изменить права роли owner
    if role == "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "Нельзя изменить права роли владельца".to_string(),
        ));
    }

    // Проверяем, существует ли запись о правах
    let existing: Option<BoardPermission> = sqlx::query_as::<_, BoardPermission>(
        "SELECT * FROM board_permissions WHERE board_id = ? AND role = ?",
    )
    .bind(board_id)
    .bind(&role)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let permission = if let Some(perm) = existing {
        // Обновляем существующие права
        let mut updates = Vec::new();
        let mut bool_values = Vec::new();

        if let Some(val) = payload.can_view {
            updates.push("can_view = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_create_cards {
            updates.push("can_create_cards = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_edit_cards {
            updates.push("can_edit_cards = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_delete_cards {
            updates.push("can_delete_cards = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_move_cards {
            updates.push("can_move_cards = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_create_lists {
            updates.push("can_create_lists = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_edit_lists {
            updates.push("can_edit_lists = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_delete_lists {
            updates.push("can_delete_lists = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_manage_members {
            updates.push("can_manage_members = ?");
            bool_values.push(val);
        }
        if let Some(val) = payload.can_manage_settings {
            updates.push("can_manage_settings = ?");
            bool_values.push(val);
        }

        if updates.is_empty() {
            return Ok(Json(perm));
        }

        let query = format!(
            "UPDATE board_permissions SET {} WHERE board_id = ? AND role = ? RETURNING *",
            updates.join(", ")
        );

        let mut sqlx_query = sqlx::query_as::<_, BoardPermission>(&query);
        for val in bool_values {
            sqlx_query = sqlx_query.bind(val);
        }
        sqlx_query = sqlx_query.bind(board_id).bind(&role);

        sqlx_query
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        // Создаём новые права
        let can_view = payload.can_view.unwrap_or(true);
        let can_create_cards = payload.can_create_cards.unwrap_or(false);
        let can_edit_cards = payload.can_edit_cards.unwrap_or(false);
        let can_delete_cards = payload.can_delete_cards.unwrap_or(false);
        let can_move_cards = payload.can_move_cards.unwrap_or(false);
        let can_create_lists = payload.can_create_lists.unwrap_or(false);
        let can_edit_lists = payload.can_edit_lists.unwrap_or(false);
        let can_delete_lists = payload.can_delete_lists.unwrap_or(false);
        let can_manage_members = payload.can_manage_members.unwrap_or(false);
        let can_manage_settings = payload.can_manage_settings.unwrap_or(false);

        sqlx::query_as::<_, BoardPermission>(
            "INSERT INTO board_permissions 
             (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, 
              can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, 
              can_manage_members, can_manage_settings) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
             RETURNING *",
        )
        .bind(board_id)
        .bind(&role)
        .bind(can_view)
        .bind(can_create_cards)
        .bind(can_edit_cards)
        .bind(can_delete_cards)
        .bind(can_move_cards)
        .bind(can_create_lists)
        .bind(can_edit_lists)
        .bind(can_delete_lists)
        .bind(can_manage_members)
        .bind(can_manage_settings)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        board_id = board_id,
        role = role,
        event = "permissions_updated",
        "Права доступа обновлены"
    );

    Ok(Json(permission))
}

/// Проверка наличия роли у пользователя
async fn has_role(
    pool: &SqlitePool,
    board_id: i64,
    user_id: i64,
    roles: &[&str],
) -> Result<bool, sqlx::Error> {
    let placeholders = roles.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ? AND role IN ({})",
        placeholders
    );

    let mut sqlx_query = sqlx::query_as::<_, (i64,)>(&query)
        .bind(board_id)
        .bind(user_id);

    for role in roles {
        sqlx_query = sqlx_query.bind(*role);
    }

    let result: Option<(i64,)> = sqlx_query.fetch_optional(pool).await?;
    Ok(result.is_some())
}

/// Проверка конкретного права пользователя
pub async fn check_permission(
    Path((board_id, permission)): Path<(i64, String)>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<bool>, (StatusCode, String)> {
    let board: crate::models::Board =
        sqlx::query_as::<_, crate::models::Board>("SELECT * FROM boards WHERE id = ?")
            .bind(board_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;

    // Владелец всегда имеет все права
    if claims.user_id == board.owner_id {
        return Ok(Json(true));
    }

    // Получаем роль пользователя
    let member_role: Option<(String,)> =
        sqlx::query_as("SELECT role FROM board_members WHERE board_id = ? AND user_id = ?")
            .bind(board_id)
            .bind(claims.user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let role = match member_role {
        Some((r,)) => r,
        None => return Ok(Json(false)),
    };

    // Получаем права для роли
    let permission_row: Option<BoardPermission> = sqlx::query_as::<_, BoardPermission>(
        "SELECT * FROM board_permissions WHERE board_id = ? AND role = ?",
    )
    .bind(board_id)
    .bind(&role)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let has_permission = match permission_row {
        Some(p) => match permission.as_str() {
            "can_view" => p.can_view,
            "can_create_cards" => p.can_create_cards,
            "can_edit_cards" => p.can_edit_cards,
            "can_delete_cards" => p.can_delete_cards,
            "can_move_cards" => p.can_move_cards,
            "can_create_lists" => p.can_create_lists,
            "can_edit_lists" => p.can_edit_lists,
            "can_delete_lists" => p.can_delete_lists,
            "can_manage_members" => p.can_manage_members,
            "can_manage_settings" => p.can_manage_settings,
            _ => false,
        },
        None => false,
    };

    Ok(Json(has_permission))
}
