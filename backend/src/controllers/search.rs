use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use serde::Deserialize;

/// Параметры поиска карточек
#[derive(Deserialize, Default)]
pub struct CardSearchQuery {
    pub q: Option<String>,           // Поисковый запрос
    pub label_color: Option<String>, // Фильтр по цвету метки
    pub label_name: Option<String>,  // Фильтр по названию метки
    pub done: Option<bool>,          // Фильтр по статусу
}

/// Результат поиска карточки
#[derive(serde::Serialize)]
pub struct CardSearchResult {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub done: bool,
    pub due_date: Option<i64>,
    pub list_id: i64,
    pub list_title: String,
    pub board_id: i64,
    pub board_title: String,
    pub labels: Vec<LabelInfo>,
}

#[derive(serde::Serialize)]
pub struct LabelInfo {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Тип для строки результата поиска карточек
type CardSearchRow = (i64, String, Option<String>, bool, Option<i64>, i64, String, i64, String);

/// Поиск карточек на доске
pub async fn search_cards_on_board(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Query(query): Query<CardSearchQuery>,
) -> Result<Json<Vec<CardSearchResult>>, (StatusCode, String)> {
    // Построение запроса в зависимости от параметров
    let cards: Vec<CardSearchRow> = if query.q.is_none() && query.label_color.is_none() && query.label_name.is_none() && query.done.is_none() {
        // Без фильтров
        sqlx::query_as(
            r#"
            SELECT 
                c.id, c.title, c.content, c.done, c.due_date,
                c.list_id, l.title as list_title,
                l.board_id, b.title as board_title
            FROM cards c
            INNER JOIN lists l ON c.list_id = l.id
            INNER JOIN boards b ON l.board_id = b.id
            WHERE l.board_id = ?
            ORDER BY c.position, c.id
            "#,
        )
        .bind(board_id)
        .fetch_all(&pool)
        .await
    } else if let Some(q) = &query.q {
        // Только поиск по тексту
        let pattern = format!("%{}%", q);
        sqlx::query_as(
            r#"
            SELECT 
                c.id, c.title, c.content, c.done, c.due_date,
                c.list_id, l.title as list_title,
                l.board_id, b.title as board_title
            FROM cards c
            INNER JOIN lists l ON c.list_id = l.id
            INNER JOIN boards b ON l.board_id = b.id
            WHERE l.board_id = ? AND (c.title LIKE ? OR c.content LIKE ?)
            ORDER BY c.position, c.id
            "#,
        )
        .bind(board_id)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&pool)
        .await
    } else {
        // Сложный запрос с фильтрами
        let mut sql = String::from(
            r#"
            SELECT
                c.id, c.title, c.content, c.done, c.due_date,
                c.list_id, l.title as list_title,
                l.board_id, b.title as board_title
            FROM cards c
            INNER JOIN lists l ON c.list_id = l.id
            INNER JOIN boards b ON l.board_id = b.id
            WHERE l.board_id = ?
            "#,
        );

        if query.done.is_some() {
            sql.push_str(" AND c.done = ?");
        }

        if query.label_color.is_some() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM labels lbl WHERE lbl.card_id = c.id AND lbl.color = ?)");
        }

        if query.label_name.is_some() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM labels lbl WHERE lbl.card_id = c.id AND lbl.name = ?)");
        }

        sql.push_str(" ORDER BY c.position, c.id");

        let mut query_builder = sqlx::query_as::<_, (i64, String, Option<String>, bool, Option<i64>, i64, String, i64, String)>(&sql);
        query_builder = query_builder.bind(board_id);

        if let Some(done) = query.done {
            query_builder = query_builder.bind(done);
        }

        if let Some(label_color) = &query.label_color {
            query_builder = query_builder.bind(label_color);
        }

        if let Some(label_name) = &query.label_name {
            query_builder = query_builder.bind(label_name);
        }

        query_builder.fetch_all(&pool).await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Преобразуем результаты и добавляем метки
    let mut results = Vec::new();
    for row in cards {
        let card_id = row.0;
        
        // Загружаем метки для карточки
        let labels: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, name, color FROM labels WHERE card_id = ?"
        )
        .bind(card_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        results.push(CardSearchResult {
            id: row.0,
            title: row.1,
            content: row.2,
            done: row.3,
            due_date: row.4,
            list_id: row.5,
            list_title: row.6,
            board_id: row.7,
            board_title: row.8,
            labels: labels.into_iter().map(|l| LabelInfo {
                id: l.0,
                name: l.1,
                color: l.2,
            }).collect(),
        });
    }

    Ok(Json(results))
}

/// Получить все метки на доске (для фильтра)
pub async fn get_board_labels(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<LabelInfo>>, (StatusCode, String)> {
    let labels: Vec<(i64, String, String)> = sqlx::query_as(
        r#"
        SELECT DISTINCT lbl.id, lbl.name, lbl.color
        FROM labels lbl
        INNER JOIN cards c ON lbl.card_id = c.id
        INNER JOIN lists l ON c.list_id = l.id
        WHERE l.board_id = ?
        ORDER BY lbl.color, lbl.name
        "#,
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(labels.into_iter().map(|l| LabelInfo {
        id: l.0,
        name: l.1,
        color: l.2,
    }).collect()))
}

/// Быстрый поиск карточек по всем доскам (глобальный поиск)
pub async fn global_card_search(
    State(pool): State<SqlitePool>,
    Query(query): Query<CardSearchQuery>,
) -> Result<Json<Vec<CardSearchResult>>, (StatusCode, String)> {
    let search_pattern = query.q.map(|q| format!("%{}%", q));

    let cards: Vec<CardSearchRow> = if let Some(pattern) = &search_pattern {
        sqlx::query_as(
            r#"
            SELECT 
                c.id, c.title, c.content, c.done, c.due_date,
                c.list_id, l.title as list_title,
                l.board_id, b.title as board_title
            FROM cards c
            INNER JOIN lists l ON c.list_id = l.id
            INNER JOIN boards b ON l.board_id = b.id
            WHERE (b.visibility = 'public' OR b.owner_id = 1 OR EXISTS (
                SELECT 1 FROM board_members bm WHERE bm.board_id = b.id AND bm.user_id = 1
            ))
            AND (c.title LIKE ? OR c.content LIKE ?)
            ORDER BY b.title, l.title, c.position, c.id LIMIT 50
            "#,
        )
        .bind(pattern)
        .bind(pattern)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as(
            r#"
            SELECT 
                c.id, c.title, c.content, c.done, c.due_date,
                c.list_id, l.title as list_title,
                l.board_id, b.title as board_title
            FROM cards c
            INNER JOIN lists l ON c.list_id = l.id
            INNER JOIN boards b ON l.board_id = b.id
            WHERE (b.visibility = 'public' OR b.owner_id = 1 OR EXISTS (
                SELECT 1 FROM board_members bm WHERE bm.board_id = b.id AND bm.user_id = 1
            ))
            ORDER BY b.title, l.title, c.position, c.id LIMIT 50
            "#,
        )
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut results = Vec::new();
    for row in cards {
        let card_id = row.0;
        
        let labels: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, name, color FROM labels WHERE card_id = ?"
        )
        .bind(card_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        results.push(CardSearchResult {
            id: row.0,
            title: row.1,
            content: row.2,
            done: row.3,
            due_date: row.4,
            list_id: row.5,
            list_title: row.6,
            board_id: row.7,
            board_title: row.8,
            labels: labels.into_iter().map(|l| LabelInfo {
                id: l.0,
                name: l.1,
                color: l.2,
            }).collect(),
        });
    }

    Ok(Json(results))
}
