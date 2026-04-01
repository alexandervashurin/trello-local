use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::models::{BoardTemplate, TemplateList, TemplateCard, CreateBoardTemplate, TemplateApplyResult};
use crate::views::Claims;

/// Получить все шаблоны пользователя
pub async fn get_templates(
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<BoardTemplate>>, (StatusCode, String)> {
    let templates: Vec<BoardTemplate> = sqlx::query_as(
        "SELECT id, user_id, title, description, is_public, created_at FROM board_templates WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(claims.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(templates))
}

/// Получить шаблон с списками и карточками
pub async fn get_template(
    Path(template_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<BoardTemplateWithLists>, (StatusCode, String)> {
    // Проверка прав доступа
    let template: BoardTemplate = sqlx::query_as(
        "SELECT id, user_id, title, description, is_public, created_at FROM board_templates WHERE id = ? AND (user_id = ? OR is_public = 1)",
    )
    .bind(template_id)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Шаблон не найден".to_string()))?;

    // Получаем списки
    let lists: Vec<TemplateList> = sqlx::query_as(
        "SELECT id, template_id, title, position FROM board_template_lists WHERE template_id = ? ORDER BY position",
    )
    .bind(template_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Получаем карточки для каждого списка
    let mut lists_with_cards = Vec::new();
    for list in lists {
        let cards: Vec<TemplateCard> = sqlx::query_as(
            "SELECT id, list_id, title, content, position FROM board_template_cards WHERE list_id = ? ORDER BY position",
        )
        .bind(list.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        lists_with_cards.push(TemplateListWithCards {
            id: list.id,
            template_id: list.template_id,
            title: list.title,
            position: list.position,
            cards,
        });
    }

    Ok(Json(BoardTemplateWithLists {
        id: template.id,
        user_id: template.user_id,
        title: template.title,
        description: template.description,
        is_public: template.is_public,
        created_at: template.created_at,
        lists: lists_with_cards,
    }))
}

/// Создать шаблон из существующей доски
pub async fn create_template_from_board(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateBoardTemplate>,
) -> Result<Json<BoardTemplate>, (StatusCode, String)> {
    // Проверка прав на доску
    let has_access: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM boards WHERE id = ? AND owner_id = ?",
    )
    .bind(board_id)
    .bind(claims.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if has_access.is_none() {
        return Err((StatusCode::FORBIDDEN, "Нет доступа к доске".to_string()));
    }

    // Создаём шаблон
    let is_public = payload.is_public.unwrap_or(false);
    let template: BoardTemplate = sqlx::query_as(
        "INSERT INTO board_templates (user_id, title, description, is_public) VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(claims.user_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(is_public)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Копируем списки
    let lists: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, title, position FROM lists WHERE board_id = ? ORDER BY position",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for (list_id, list_title, list_position) in lists {
        let new_list_id: i64 = sqlx::query_scalar(
            "INSERT INTO board_template_lists (template_id, title, position) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(template.id)
        .bind(&list_title)
        .bind(list_position)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Копируем карточки
        let cards: Vec<(String, Option<String>, i64)> = sqlx::query_as(
            "SELECT title, content, position FROM cards WHERE list_id = ? ORDER BY position",
        )
        .bind(list_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        for (card_title, card_content, card_position) in cards {
            let _: Result<i64, _> = sqlx::query_scalar(
                "INSERT INTO board_template_cards (list_id, title, content, position) VALUES (?, ?, ?, ?)",
            )
            .bind(new_list_id)
            .bind(&card_title)
            .bind(&card_content)
            .bind(card_position)
            .fetch_one(&pool)
            .await;
        }
    }

    Ok(Json(template))
}

/// Удалить шаблон
pub async fn delete_template(
    Path(template_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    let result = sqlx::query(
        "DELETE FROM board_templates WHERE id = ? AND user_id = ?",
    )
    .bind(template_id)
    .bind(claims.user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Шаблон не найден".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Создать доску из шаблона
pub async fn apply_template(
    Path(template_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ApplyTemplateRequest>,
) -> Result<Json<TemplateApplyResult>, (StatusCode, String)> {
    // Проверка прав на шаблон
    let template: BoardTemplate = sqlx::query_as(
        "SELECT id, user_id, title, description, is_public FROM board_templates WHERE id = ? AND (user_id = ? OR is_public = 1)",
    )
    .bind(template_id)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Шаблон не найден".to_string()))?;

    // Создаём новую доску
    let board_title = payload.title.unwrap_or_else(|| format!("{} (копия)", template.title));
    let board_id: i64 = sqlx::query_scalar(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES (?, ?, 0) RETURNING id",
    )
    .bind(&board_title)
    .bind(claims.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Добавляем владельца как участника
    let _ = sqlx::query(
        "INSERT INTO board_members (board_id, user_id, role) VALUES (?, ?, 'owner')",
    )
    .bind(board_id)
    .bind(claims.user_id)
    .execute(&pool)
    .await;

    // Получаем списки шаблона
    let template_lists: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, title, position FROM board_template_lists WHERE template_id = ? ORDER BY position",
    )
    .bind(template_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut lists_created = 0i64;
    let mut cards_created = 0i64;

    // Копируем списки и карточки
    for (template_list_id, list_title, list_position) in template_lists {
        let new_list_id: i64 = sqlx::query_scalar(
            "INSERT INTO lists (board_id, title, position) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(board_id)
        .bind(&list_title)
        .bind(list_position)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        lists_created += 1;

        // Копируем карточки
        let cards: Vec<(String, Option<String>, i64)> = sqlx::query_as(
            "SELECT title, content, position FROM board_template_cards WHERE list_id = ? ORDER BY position",
        )
        .bind(template_list_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        for (card_title, card_content, card_position) in cards {
            let _ = sqlx::query_scalar::<_, i64>(
                "INSERT INTO cards (list_id, title, content, position) VALUES (?, ?, ?, ?)",
            )
            .bind(new_list_id)
            .bind(&card_title)
            .bind(&card_content)
            .bind(card_position)
            .fetch_one(&pool)
            .await;

            cards_created += 1;
        }
    }

    Ok(Json(TemplateApplyResult {
        board_id,
        board_title,
        lists_created,
        cards_created,
    }))
}

/// Структуры для ответов
#[derive(serde::Serialize)]
pub struct BoardTemplateWithLists {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: i64,
    pub lists: Vec<TemplateListWithCards>,
}

#[derive(serde::Serialize)]
pub struct TemplateListWithCards {
    pub id: i64,
    pub template_id: i64,
    pub title: String,
    pub position: i64,
    pub cards: Vec<TemplateCard>,
}

#[derive(Deserialize)]
pub struct ApplyTemplateRequest {
    pub title: Option<String>,
}
