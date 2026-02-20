use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{Card, CreateCard, UpdateCard, Label, CreateLabel, UpdateLabel, Attachment, ActivityLog};

/// Вспомогательная функция для получения board_id по list_id
async fn get_board_id_by_list_id(pool: &SqlitePool, list_id: i64) -> Option<i64> {
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT board_id FROM lists WHERE id = ?"
    )
    .bind(list_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    result.map(|r| r.0)
}

/// Создать карточку в списке
pub async fn create_card(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    // Валидация названия
    if payload.title.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Название не может быть пустым".to_string()));
    }
    if payload.title.len() > 200 {
        return Err((StatusCode::BAD_REQUEST, "Название слишком длинное".to_string()));
    }

    // Получаем board_id из list_id
    let board_id = get_board_id_by_list_id(&pool, list_id).await
        .ok_or((StatusCode::NOT_FOUND, "Список не найден".to_string()))?;

    let card: Card = sqlx::query_as::<_, Card>(
        "INSERT INTO cards (list_id, title, content, done, due_date) VALUES (?, ?, ?, 0, ?) RETURNING id, list_id, title, content, done, due_date",
    )
    .bind(list_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(payload.due_date)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    let _ = log_activity(
        &pool,
        board_id,
        None,
        "create",
        Some("card"),
        Some(card.id),
        &format!("Создана карточка \"{}\"", &payload.title),
        None,
    ).await;

    Ok(Json(card))
}

/// Обновить карточку
pub async fn update_card(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateCard>,
) -> Result<Json<Card>, (StatusCode, String)> {
    let current: Card = sqlx::query_as::<_, Card>(
        "SELECT id, list_id, title, content, done, due_date FROM cards WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Карточка не найдена".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let new_title = payload.title.clone().unwrap_or(current.title.clone());
    let new_content = payload.content.clone().or(current.content.clone());
    let new_list_id = payload.list_id.unwrap_or(current.list_id);
    let new_done = payload.done.unwrap_or(current.done);
    let new_due_date = payload.due_date.or(current.due_date);

    // Получаем board_id для логирования
    let board_id = get_board_id_by_list_id(&pool, new_list_id).await
        .ok_or((StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;

    let updated: Card = sqlx::query_as(
        "UPDATE cards SET title = ?, content = ?, list_id = ?, done = ?, due_date = ? WHERE id = ? RETURNING id, list_id, title, content, done, due_date"
    )
    .bind(&new_title)
    .bind(&new_content)
    .bind(new_list_id)
    .bind(new_done)
    .bind(new_due_date)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование изменений
    let mut changes = Vec::new();
    if payload.title.is_some() && payload.title.as_ref().map(|s| s.as_str()) != Some(current.title.as_str()) {
        changes.push(format!("название → \"{}\"", new_title));
    }
    if payload.content.is_some() && payload.content != current.content {
        changes.push("описание изменено".to_string());
    }
    if payload.done.is_some() && payload.done != Some(current.done) {
        changes.push(if new_done { "отмечена выполненной".to_string() } else { "возвращена в работу".to_string() });
    }
    if payload.due_date.is_some() && payload.due_date != current.due_date {
        changes.push("дедлайн изменён".to_string());
    }
    if payload.list_id.is_some() && payload.list_id != Some(current.list_id) {
        changes.push("перемещена в другой список".to_string());
    }

    if !changes.is_empty() {
        let _ = log_activity(
            &pool,
            board_id,
            None,
            "update",
            Some("card"),
            Some(id),
            &format!("Карточка \"{}\": {}", current.title, changes.join(", ")),
            None,
        ).await;
    }

    Ok(Json(updated))
}

/// Удалить карточку
pub async fn delete_card(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Получаем информацию о карточке перед удалением
    let card: Option<(String, i64)> = sqlx::query_as(
        "SELECT title, list_id FROM cards WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (title, list_id) = card.ok_or((StatusCode::NOT_FOUND, "Карточка не найдена".to_string()))?;

    // Получаем board_id
    let board_id = get_board_id_by_list_id(&pool, list_id).await
        .ok_or((StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;

    let result = sqlx::query("DELETE FROM cards WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Карточка не найдена".to_string()))
    } else {
        // Логирование
        let _ = log_activity(
            &pool,
            board_id,
            None,
            "delete",
            Some("card"),
            Some(id),
            &format!("Удалена карточка \"{}\"", title),
            None,
        ).await;
        Ok(Json(()))
    }
}

/// Вспомогательная функция для получения board_id по card_id
pub async fn get_board_id_by_card_id(pool: &SqlitePool, card_id: i64) -> Option<i64> {
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT b.id FROM boards b INNER JOIN lists l ON b.id = l.board_id INNER JOIN cards c ON l.id = c.list_id WHERE c.id = ?"
    )
    .bind(card_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    result.map(|r| r.0)
}

/// Получить метки карточки
pub async fn get_card_labels(
    Path(card_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Label>>, (StatusCode, String)> {
    let labels: Vec<Label> = sqlx::query_as(
        "SELECT id, card_id, name, color FROM labels WHERE card_id = ?"
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(labels))
}

/// Добавить метку к карточке
pub async fn create_label(
    Path(card_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateLabel>,
) -> Result<Json<Label>, (StatusCode, String)> {
    let color = payload.color.unwrap_or_else(|| "blue".to_string());

    let label: Label = sqlx::query_as::<_, Label>(
        "INSERT INTO labels (card_id, name, color) VALUES (?, ?, ?) RETURNING id, card_id, name, color",
    )
    .bind(card_id)
    .bind(&payload.name)
    .bind(&color)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
        let _ = log_activity(
            &pool,
            board_id,
            None,
            "create",
            Some("label"),
            Some(label.id),
            &format!("Добавлена метка \"{}\" к карточке", &payload.name),
            None,
        ).await;
    }

    Ok(Json(label))
}

/// Обновить метку
pub async fn update_label(
    Path((card_id, label_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateLabel>,
) -> Result<Json<Label>, (StatusCode, String)> {
    let current: Label = sqlx::query_as::<_, Label>(
        "SELECT id, card_id, name, color FROM labels WHERE id = ? AND card_id = ?"
    )
    .bind(label_id)
    .bind(card_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Метка не найдена".to_string()))?;

    let new_name = payload.name.clone().unwrap_or(current.name.clone());
    let new_color = payload.color.clone().unwrap_or(current.color.clone());

    let updated: Label = sqlx::query_as(
        "UPDATE labels SET name = ?, color = ? WHERE id = ? RETURNING id, card_id, name, color"
    )
    .bind(&new_name)
    .bind(&new_color)
    .bind(label_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
        let mut changes = Vec::new();
        if payload.name.is_some() && payload.name.as_ref().map(|s| s.as_str()) != Some(current.name.as_str()) {
            changes.push(format!("название → \"{}\"", new_name));
        }
        if payload.color.is_some() && payload.color != Some(current.color) {
            changes.push(format!("цвет → {}", new_color));
        }
        if !changes.is_empty() {
            let _ = log_activity(
                &pool,
                board_id,
                None,
                "update",
                Some("label"),
                Some(label_id),
                &format!("Метка \"{}\": {}", current.name, changes.join(", ")),
                None,
            ).await;
        }
    }

    Ok(Json(updated))
}

/// Удалить метку
pub async fn delete_label(
    Path((card_id, label_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Получаем название метки перед удалением
    let label_name: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM labels WHERE id = ? AND card_id = ?"
    )
    .bind(label_id)
    .bind(card_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let name = label_name.map(|l| l.0).unwrap_or_else(|| "неизвестно".to_string());

    let result = sqlx::query("DELETE FROM labels WHERE id = ? AND card_id = ?")
        .bind(label_id)
        .bind(card_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Метка не найдена".to_string()))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let _ = log_activity(
                &pool,
                board_id,
                None,
                "delete",
                Some("label"),
                Some(label_id),
                &format!("Удалена метка \"{}\"", name),
                None,
            ).await;
        }
        Ok(Json(()))
    }
}

/// Получить вложения карточки
pub async fn get_card_attachments(
    Path(card_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Attachment>>, (StatusCode, String)> {
    let attachments: Vec<Attachment> = sqlx::query_as(
        "SELECT id, card_id, user_id, filename, file_path, file_size, mime_type, created_at FROM attachments WHERE card_id = ? ORDER BY created_at DESC"
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attachments))
}

/// Удалить вложение
pub async fn delete_attachment(
    Path((card_id, attachment_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Сначала получаем путь к файлу
    let attachment: Attachment = sqlx::query_as::<_, Attachment>(
        "SELECT id, card_id, user_id, filename, file_path, file_size, mime_type, created_at FROM attachments WHERE id = ? AND card_id = ?"
    )
    .bind(attachment_id)
    .bind(card_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Вложение не найдено".to_string()))?;

    let filename = attachment.filename.clone();

    // Удаляем файл
    let _ = std::fs::remove_file(&attachment.file_path);

    // Удаляем запись из БД
    let result = sqlx::query("DELETE FROM attachments WHERE id = ? AND card_id = ?")
        .bind(attachment_id)
        .bind(card_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Вложение не найдено".to_string()))
    } else {
        // Логирование
        if let Some(board_id) = get_board_id_by_card_id(&pool, card_id).await {
            let _ = log_activity(
                &pool,
                board_id,
                None,
                "delete",
                Some("attachment"),
                Some(attachment_id),
                &format!("Удалено вложение \"{}\"", filename),
                None,
            ).await;
        }
        Ok(Json(()))
    }
}

/// Получить историю активности доски
pub async fn get_activity_log(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ActivityLog>>, (StatusCode, String)> {
    let activities: Vec<ActivityLog> = sqlx::query_as(
        "SELECT id, board_id, user_id, action_type, entity_type, entity_id, description, metadata, created_at FROM activity_log WHERE board_id = ? ORDER BY created_at DESC LIMIT 100"
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(activities))
}

/// Вспомогательная функция для логирования действий
pub async fn log_activity(
    pool: &SqlitePool,
    board_id: i64,
    user_id: Option<i64>,
    action_type: &str,
    entity_type: Option<&str>,
    entity_id: Option<i64>,
    description: &str,
    metadata: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO activity_log (board_id, user_id, action_type, entity_type, entity_id, description, metadata) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(board_id)
    .bind(user_id)
    .bind(action_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(description)
    .bind(metadata)
    .execute(pool)
    .await?;
    
    Ok(())
}
