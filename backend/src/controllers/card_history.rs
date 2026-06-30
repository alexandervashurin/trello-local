use crate::controllers::cards::get_board_id_by_card_id;
use crate::models::card::CardVersionWithUser;
use crate::views::Claims;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

/// Получить историю изменений карточки
pub async fn get_card_history(
    Path(card_id): Path<i64>,
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<CardVersionWithUser>>, (StatusCode, String)> {
    // Проверка прав доступа
    let board_id = get_board_id_by_card_id(&pool, card_id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Карточка не найдена".to_string()))?;

    // Проверяем доступ к доске
    let has_access: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1 FROM boards b
        LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
        WHERE b.id = $2 AND (b.owner_id = $3 OR b.visibility = 'public' OR bm.user_id = $4)
        "#,
    )
    .bind(claims.user_id)
    .bind(board_id)
    .bind(claims.user_id)
    .bind(claims.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if has_access.is_none() {
        return Err((StatusCode::FORBIDDEN, "Нет доступа к карточке".to_string()));
    }

    // Получаем историю изменений
    #[allow(clippy::type_complexity)]
    let versions: Vec<(
        i64,
        i64,
        String,
        Option<String>,
        bool,
        Option<i64>,
        i64,
        i64,
        i64,
        String,
        String,
    )> = sqlx::query_as(
        r#"
        SELECT 
            cv.id, cv.card_id, cv.title, cv.content, cv.done, cv.due_date,
            cv.list_id, cv.edited_by, cv.edited_at, cv.change_summary,
            u.username as editor_username
        FROM card_versions cv
        INNER JOIN users u ON cv.edited_by = u.id
        WHERE cv.card_id = $1
        ORDER BY cv.edited_at DESC
        LIMIT 50
        "#,
    )
    .bind(card_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = versions
        .into_iter()
        .map(|row| CardVersionWithUser {
            id: row.0,
            card_id: row.1,
            title: row.2,
            content: row.3,
            done: row.4,
            due_date: row.5,
            list_id: row.6,
            edited_by: row.7,
            edited_at: row.8,
            change_summary: row.9,
            editor_username: row.10,
        })
        .collect();

    Ok(Json(result))
}

/// Сохранить версию карточки (вызывается при обновлении)
pub async fn save_card_version(
    pool: &PgPool,
    card_id: i64,
    user_id: i64,
    change_summary: &str,
) -> Result<(), sqlx::Error> {
    // Получаем текущие данные карточки
    #[allow(clippy::type_complexity)]
    let card: Option<(String, Option<String>, bool, Option<i64>, i64)> =
        sqlx::query_as("SELECT title, content, done, due_date, list_id FROM cards WHERE id = $1")
            .bind(card_id)
            .fetch_optional(pool)
            .await?;

    if let Some((title, content, done, due_date, list_id)) = card {
        sqlx::query(
            "INSERT INTO card_versions (card_id, title, content, done, due_date, list_id, edited_by, change_summary) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(card_id)
        .bind(&title)
        .bind(&content)
        .bind(done)
        .bind(due_date)
        .bind(list_id)
        .bind(user_id)
        .bind(change_summary)
        .execute(pool)
        .await?;
    }

    Ok(())
}
