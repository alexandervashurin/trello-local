use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::Attachment;
use std::path::PathBuf;
use chrono::Utc;

/// Максимальный размер загружаемого файла (10 MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Разрешённые MIME-типы для загружаемых файлов
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "application/pdf",
    "text/plain",
    "application/json",
    "application/zip",
    "application/x-zip-compressed",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

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

    // Валидация имени файла
    if filename.is_empty() || filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Недопустимое имя файла".to_string()));
    }

    // Получаем mime_type до вызова bytes()
    let mime_type = field.content_type().map(|s| s.to_string());

    // Валидация MIME-типа
    if let Some(ref mt) = mime_type {
        if !ALLOWED_MIME_TYPES.contains(&mt.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!(
                "Недопустимый тип файла. Разрешены: {}",
                ALLOWED_MIME_TYPES.join(", ")
            )));
        }
    }

    // Генерируем уникальное имя файла
    let timestamp = Utc::now().timestamp();
    let safe_filename = format!("{}_{}", timestamp, filename.replace(" ", "_"));
    let file_path = attachments_dir.join(&safe_filename);

    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Ошибка чтения файла: {}", e)))?;

    // Валидация размера файла
    let file_size = data.len() as u64;
    if file_size > MAX_FILE_SIZE {
        return Err((StatusCode::BAD_REQUEST, format!(
            "Файл слишком большой. Максимальный размер: {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        )));
    }

    if file_size == 0 {
        return Err((StatusCode::BAD_REQUEST, "Пустой файл".to_string()));
    }

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
    .bind(file_size as i64)
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
