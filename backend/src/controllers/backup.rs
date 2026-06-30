// backend/src/controllers/backup.rs
use crate::models::{Backup, BackupList, CreateBackup};
use crate::views::Claims;
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use chrono::Utc;
use sqlx::PgPool;
use std::path::PathBuf;
use tokio::fs;

const BACKUP_DIR: &str = "./backups";

/// Создать backup базы данных
pub async fn create_backup(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateBackup>,
) -> Result<Json<Backup>, (StatusCode, String)> {
    // Проверка прав: только admin может создавать backup
    if !is_admin(&pool, claims.user_id).await.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            "Только администратор может создавать backup".to_string(),
        ));
    }

    // Создаём директорию для backup'ов
    fs::create_dir_all(BACKUP_DIR).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка создания директории: {}", e),
        )
    })?;

    // Генерируем имя файла
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("backup_{}.db", timestamp);
    let file_path = PathBuf::from(BACKUP_DIR).join(&filename);

    // Копируем базу данных
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/trello.db".to_string());

    fs::copy(&db_path, &file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка копирования БД: {}", e),
        )
    })?;

    // Получаем размер файла
    let metadata = fs::metadata(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка получения метаданных: {}", e),
        )
    })?;
    let file_size = metadata.len() as i64;

    // Сохраняем запись в БД
    let backup = sqlx::query_as::<_, Backup>(
        "INSERT INTO backups (filename, file_path, file_size, created_by, description) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(&filename)
    .bind(file_path.to_str().unwrap())
    .bind(file_size)
    .bind(claims.user_id)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        backup_id = backup.id,
        event = "backup_created",
        "Создан backup базы данных"
    );

    Ok(Json(backup))
}

/// Получить список backup'ов
pub async fn list_backups(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<BackupList>>, (StatusCode, String)> {
    // Проверка прав: только admin может просматривать backup'ы
    if !is_admin(&pool, claims.user_id).await.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            "Только администратор может просматривать backup'ы".to_string(),
        ));
    }

    let backups = sqlx::query_as::<_, BackupList>(
        "SELECT b.id, b.filename, b.file_size, b.created_by, b.created_at, b.description, u.username as creator_username 
         FROM backups b 
         INNER JOIN users u ON b.created_by = u.id 
         ORDER BY b.created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(backups))
}

/// Скачать backup файл
pub async fn download_backup(
    Path(backup_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Response<Body>, (StatusCode, String)> {
    // Проверка прав
    if !is_admin(&pool, claims.user_id).await.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            "Только администратор может скачивать backup'ы".to_string(),
        ));
    }

    // Получаем информацию о backup
    let backup: Backup = sqlx::query_as::<_, Backup>("SELECT * FROM backups WHERE id = $1")
        .bind(backup_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Backup не найден".to_string()))?;

    // Проверяем существование файла
    let file_path = PathBuf::from(&backup.file_path);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Файл backup не найден".to_string()));
    }

    // Читаем файл
    let file_data = fs::read(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка чтения файла: {}", e),
        )
    })?;

    // Создаём response с заголовками для скачивания
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", backup.filename),
        )
        .header("Content-Length", file_data.len())
        .body(Body::from(file_data))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(response)
}

/// Восстановить из backup
pub async fn restore_backup(
    Path(backup_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Backup>, (StatusCode, String)> {
    // Проверка прав: только admin может восстанавливать
    if !is_admin(&pool, claims.user_id).await.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            "Только администратор может восстанавливать backup".to_string(),
        ));
    }

    // Получаем информацию о backup
    let backup: Backup = sqlx::query_as::<_, Backup>("SELECT * FROM backups WHERE id = $1")
        .bind(backup_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Backup не найден".to_string()))?;

    // Проверяем существование файла
    let file_path = PathBuf::from(&backup.file_path);
    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Файл backup не найден".to_string()));
    }

    // Получаем путь к базе данных
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/trello.db".to_string());

    // Копируем backup на место базы данных
    fs::copy(&file_path, &db_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка восстановления: {}", e),
        )
    })?;

    tracing::warn!(
        target: "security",
        user_id = claims.user_id,
        backup_id = backup_id,
        event = "backup_restored",
        "Восстановлен backup базы данных"
    );

    Ok(Json(backup))
}

/// Удалить backup
pub async fn delete_backup(
    Path(backup_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Проверка прав
    if !is_admin(&pool, claims.user_id).await.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            "Только администратор может удалять backup'ы".to_string(),
        ));
    }

    // Получаем информацию о backup
    let backup: Backup = sqlx::query_as::<_, Backup>("SELECT * FROM backups WHERE id = $1")
        .bind(backup_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Backup не найден".to_string()))?;

    // Удаляем файл
    let file_path = PathBuf::from(&backup.file_path);
    if file_path.exists() {
        fs::remove_file(&file_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка удаления файла: {}", e),
            )
        })?;
    }

    // Удаляем запись из БД
    sqlx::query("DELETE FROM backups WHERE id = $1")
        .bind(backup_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        backup_id = backup_id,
        event = "backup_deleted",
        "Удален backup базы данных"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Проверка прав администратора
async fn is_admin(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    // В простой реализации проверяем, есть ли у пользователя доски с ролью owner
    let result: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM board_members WHERE user_id = $1 AND role = 'owner' LIMIT 1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(result.is_some())
}
