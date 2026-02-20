use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::Attachment;
use std::path::PathBuf;
use chrono::Utc;

/// Загрузить файл к карточке
pub async fn upload_attachment(
    Path((card_id, _board_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Json<Attachment>, (StatusCode, String)> {
    let user_id = 1; // По умолчанию первый пользователь

    // Создаём директорию для вложений
    let attachments_dir = PathBuf::from("/opt/trello-local/backend/data/attachments");
    std::fs::create_dir_all(&attachments_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Получаем файл из multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Ошибка multipart: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "Файл не найден".to_string()))?;

    let filename = field
        .file_name()
        .unwrap_or("unnamed")
        .to_string();
    
    // Получаем mime_type до вызова bytes()
    let mime_type = field.content_type().map(|s| s.to_string());
    
    // Генерируем уникальное имя файла
    let timestamp = Utc::now().timestamp();
    let safe_filename = format!("{}_{}", timestamp, filename.replace(" ", "_"));
    let file_path = attachments_dir.join(&safe_filename);

    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Ошибка чтения файла: {}", e)))?;

    let file_size = data.len() as i64;

    // Сохраняем файл
    std::fs::write(&file_path, &data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка сохранения файла: {}", e)))?;

    // Сохраняем запись в БД
    let attachment: Attachment = sqlx::query_as::<_, Attachment>(
        "INSERT INTO attachments (card_id, user_id, filename, file_path, file_size, mime_type) VALUES (?, ?, ?, ?, ?, ?) RETURNING id, card_id, user_id, filename, file_path, file_size, mime_type, created_at"
    )
    .bind(card_id)
    .bind(user_id)
    .bind(&filename)
    .bind(file_path.to_string_lossy().to_string())
    .bind(file_size)
    .bind(mime_type)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attachment))
}

/// Скачать файл
pub async fn download_attachment(
    Path(attachment_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let attachment: Attachment = sqlx::query_as::<_, Attachment>(
        "SELECT id, card_id, user_id, filename, file_path, file_size, mime_type, created_at FROM attachments WHERE id = ?"
    )
    .bind(attachment_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Вложение не найдено".to_string()))?;

    let file_data = std::fs::read(&attachment.file_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка чтения файла: {}", e)))?;

    let mime_type = attachment.mime_type.as_deref().unwrap_or("application/octet-stream");
    
    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", attachment.filename))
        .header("Content-Length", attachment.file_size.to_string())
        .body(axum::body::Body::from(file_data))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка создания ответа: {}", e)))?;

    Ok(response)
}
