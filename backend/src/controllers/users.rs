use crate::models::{ChangePassword, CreateUser, UpdateProfile, User};
use crate::views::Claims;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct GetUserQuery {
    username: Option<String>,
}

/// Получить всех пользователей (с опциональным поиском по имени)
pub async fn get_users(
    State(pool): State<SqlitePool>,
    query: Query<GetUserQuery>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users: Vec<User> = if let Some(username) = &query.username {
        sqlx::query_as("SELECT id, username, email, avatar_color, bio, last_login, created_at FROM users WHERE username = ? ORDER BY id")
            .bind(username)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT id, username, email, avatar_color, bio, last_login, created_at FROM users ORDER BY id")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

/// Создать пользователя
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, created_at) VALUES (?, strftime('%s', 'now')) RETURNING id, username, email, avatar_color, bio, last_login, created_at",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}

/// Получить пользователя по ID
pub async fn get_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, email, avatar_color, bio, last_login, created_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Пользователь не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}

/// Получить профиль текущего пользователя
pub async fn get_profile(
    Extension(claims): Extension<Claims>,
    State(pool): State<SqlitePool>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, email, avatar_color, bio, last_login, created_at FROM users WHERE id = ?",
    )
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Пользователь не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(user))
}

/// Обновить профиль пользователя
pub async fn update_profile(
    Extension(claims): Extension<Claims>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateProfile>,
) -> Result<Json<User>, (StatusCode, String)> {
    // Валидация email если предоставлен
    if let Some(ref email) = payload.email {
        if !email.is_empty() && !email.contains('@') {
            return Err((StatusCode::BAD_REQUEST, "Неверный формат email".to_string()));
        }
    }

    let user: User = sqlx::query_as::<_, User>(
        "UPDATE users SET email = COALESCE(?, email), avatar_color = COALESCE(?, avatar_color), bio = COALESCE(?, bio) WHERE id = ? RETURNING id, username, email, avatar_color, bio, last_login, created_at",
    )
    .bind(&payload.email)
    .bind(&payload.avatar_color)
    .bind(&payload.bio)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(user))
}

/// Сменить пароль пользователя
pub async fn change_password(
    Extension(claims): Extension<Claims>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<ChangePassword>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Получаем текущий хэш пароля
    let user_with_password: crate::models::UserWithPassword = sqlx::query_as(
        "SELECT id, username, password_hash, email, avatar_color, bio, last_login, created_at FROM users WHERE id = ?",
    )
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Пользователь не найден".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Проверяем текущий пароль
    let valid = bcrypt::verify(&payload.current_password, &user_with_password.password_hash)
        .map_err(|e| {
            tracing::error!(
                target: "security",
                user_id = claims.user_id,
                username = %claims.username,
                event = "password_change_error",
                reason = "bcrypt_error",
                error = %e,
                "Ошибка при проверке текущего пароля"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if !valid {
        tracing::warn!(
            target: "security",
            user_id = claims.user_id,
            username = %claims.username,
            event = "password_change_failed",
            reason = "invalid_current_password",
            "Неудачная смена пароля: неверный текущий пароль"
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            "Неверный текущий пароль".to_string(),
        ));
    }

    // Валидация нового пароля
    if payload.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Новый пароль должен быть не менее 8 символов".to_string(),
        ));
    }

    // Проверка сложности нового пароля
    let has_upper = payload.new_password.chars().any(|c| c.is_uppercase());
    let has_lower = payload.new_password.chars().any(|c| c.is_lowercase());
    let has_digit = payload.new_password.chars().any(|c| c.is_numeric());

    if !has_upper || !has_lower || !has_digit {
        return Err((
            StatusCode::BAD_REQUEST,
            "Новый пароль должен содержать заглавные и строчные буквы, а также цифры".to_string(),
        ));
    }

    // Хэшируем и сохраняем новый пароль
    let new_password_hash = bcrypt::hash(&payload.new_password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_password_hash)
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование успешной смены пароля
    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        username = %claims.username,
        event = "password_change_success",
        "Пароль успешно изменён"
    );

    Ok(Json(()))
}

/// Удалить аккаунт пользователя
pub async fn delete_account(
    Extension(claims): Extension<Claims>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<DeleteAccount>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Проверяем пароль для подтверждения
    let user_with_password: crate::models::UserWithPassword = sqlx::query_as(
        "SELECT id, username, password_hash, email, avatar_color, bio, last_login, created_at FROM users WHERE id = ?",
    )
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let valid =
        bcrypt::verify(&payload.password, &user_with_password.password_hash).map_err(|e| {
            tracing::error!(
                target: "security",
                user_id = claims.user_id,
                username = %claims.username,
                event = "delete_account_error",
                reason = "bcrypt_error",
                error = %e,
                "Ошибка при проверке пароля для удаления аккаунта"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if !valid {
        tracing::warn!(
            target: "security",
            user_id = claims.user_id,
            username = %claims.username,
            event = "delete_account_failed",
            reason = "invalid_password",
            "Неудачное удаление аккаунта: неверный пароль"
        );
        return Err((StatusCode::UNAUTHORIZED, "Неверный пароль".to_string()));
    }

    // Логирование перед удалением аккаунта
    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        username = %claims.username,
        event = "delete_account_requested",
        "Запрошено удаление аккаунта"
    );

    // Удаляем пользователя (каскадно удалит все связанные данные)
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(claims.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование успешного удаления аккаунта
    tracing::info!(
        target: "security",
        user_id = claims.user_id,
        username = %claims.username,
        event = "delete_account_success",
        "Аккаунт успешно удалён"
    );

    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct DeleteAccount {
    pub password: String,
}
