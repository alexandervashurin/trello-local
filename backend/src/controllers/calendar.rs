use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::SqlitePool;

/// Параметры запроса календаря
#[derive(Deserialize, Default)]
pub struct CalendarQuery {
    pub year: Option<i32>,
    pub month: Option<u32>, // 1-12
}

/// Карточка с дедлайном для календаря
#[derive(serde::Serialize)]
pub struct CalendarCard {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub due_date: i64,
    pub list_id: i64,
    pub list_title: String,
    pub board_id: i64,
    pub board_title: String,
    pub is_overdue: bool,
}

/// День календаря
#[derive(serde::Serialize)]
pub struct CalendarDay {
    pub date: String, // YYYY-MM-DD
    pub day: u32,
    pub cards_count: u32,
    pub overdue_count: u32,
    pub has_today: bool,
}

/// Ответ календаря
#[derive(serde::Serialize)]
pub struct CalendarResponse {
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub days: Vec<CalendarDay>,
    pub total_cards: u32,
    pub overdue_cards: u32,
}

/// Тип для строки результата запроса карточек
type CardRow = (i64, String, bool, i64, i64, String, i64, String);

/// Получить календарь дедлайнов на месяц
pub async fn get_calendar(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let year = query.year.unwrap_or(now.year());
    let month = query.month.unwrap_or(now.month());

    // Название месяца
    let month_names = [
        "Январь",
        "Февраль",
        "Март",
        "Апрель",
        "Май",
        "Июнь",
        "Июль",
        "Август",
        "Сентябрь",
        "Октябрь",
        "Ноябрь",
        "Декабрь",
    ];
    let month_name = if (1..=12).contains(&month) {
        month_names[(month - 1) as usize]
    } else {
        "Неизвестно"
    };

    // Получаем количество дней в месяце
    let days_in_month = get_days_in_month(year, month);

    // Получаем карточки с дедлайнами на этот месяц
    let start_timestamp = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .expect("Некорректная дата начала месяца")
        .and_hms_opt(0, 0, 0)
        .expect("Некорректное время 00:00:00")
        .and_utc()
        .timestamp();

    let end_timestamp = chrono::NaiveDate::from_ymd_opt(year, month, days_in_month)
        .expect("Некорректная дата конца месяца")
        .and_hms_opt(23, 59, 59)
        .expect("Некорректное время 23:59:59")
        .and_utc()
        .timestamp();

    let cards: Vec<CardRow> = sqlx::query_as(
        r#"
        SELECT
            c.id, c.title, c.done, c.due_date,
            c.list_id, l.title as list_title,
            l.board_id, b.title as board_title
        FROM cards c
        INNER JOIN lists l ON c.list_id = l.id
        INNER JOIN boards b ON l.board_id = b.id
        WHERE l.board_id = ?
          AND c.due_date IS NOT NULL
          AND c.due_date >= ?
          AND c.due_date <= ?
        ORDER BY c.due_date, c.id
        "#,
    )
    .bind(board_id)
    .bind(start_timestamp)
    .bind(end_timestamp)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let now_timestamp = now.timestamp();

    // Группируем карточки по дням
    use std::collections::HashMap;
    let mut cards_by_day: HashMap<u32, Vec<CalendarCard>> = HashMap::new();
    let mut total_cards = 0u32;
    let mut overdue_cards = 0u32;

    for row in cards {
        let due_date = row.3;
        let is_overdue = !row.2 && due_date < now_timestamp; // Не выполнено и просрочено

        if is_overdue {
            overdue_cards += 1;
        }
        total_cards += 1;

        // Получаем день месяца из timestamp
        let day = chrono::DateTime::from_timestamp(due_date, 0)
            .expect("Некорректный timestamp дедлайна")
            .day();

        cards_by_day.entry(day).or_default().push(CalendarCard {
            id: row.0,
            title: row.1,
            done: row.2,
            due_date: row.3,
            list_id: row.4,
            list_title: row.5,
            board_id: row.6,
            board_title: row.7,
            is_overdue,
        });
    }

    // Создаём дни календаря
    let today = now.day();
    let current_month = now.month();
    let current_year = now.year();

    let mut days = Vec::new();
    for day in 1..=days_in_month {
        let cards_count = cards_by_day.get(&day).map(|v| v.len() as u32).unwrap_or(0);
        let day_overdue = cards_by_day
            .get(&day)
            .map(|v| v.iter().filter(|c| c.is_overdue).count() as u32)
            .unwrap_or(0);

        days.push(CalendarDay {
            date: format!("{:04}-{:02}-{:02}", year, month, day),
            day,
            cards_count,
            overdue_count: day_overdue,
            has_today: year == current_year && month == current_month && day == today,
        });
    }

    Ok(Json(CalendarResponse {
        year,
        month,
        month_name: month_name.to_string(),
        days,
        total_cards,
        overdue_cards,
    }))
}

/// Получить карточки на конкретный день
pub async fn get_cards_for_day(
    Path((board_id, year, month, day)): Path<(i64, i32, u32, u32)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<CalendarCard>>, (StatusCode, String)> {
    let start_timestamp = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .expect("Некорректная дата начала дня")
        .and_hms_opt(0, 0, 0)
        .expect("Некорректное время 00:00:00")
        .and_utc()
        .timestamp();

    let end_timestamp = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .expect("Некорректная дата конца дня")
        .and_hms_opt(23, 59, 59)
        .expect("Некорректное время 23:59:59")
        .and_utc()
        .timestamp();

    let now_timestamp = chrono::Utc::now().timestamp();

    let cards: Vec<CardRow> = sqlx::query_as(
        r#"
        SELECT 
            c.id, c.title, c.done, c.due_date,
            c.list_id, l.title as list_title,
            l.board_id, b.title as board_title
        FROM cards c
        INNER JOIN lists l ON c.list_id = l.id
        INNER JOIN boards b ON l.board_id = b.id
        WHERE l.board_id = ? 
          AND c.due_date IS NOT NULL
          AND c.due_date >= ? 
          AND c.due_date <= ?
        ORDER BY c.due_date, c.id
        "#,
    )
    .bind(board_id)
    .bind(start_timestamp)
    .bind(end_timestamp)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = cards
        .into_iter()
        .map(|row| {
            let is_overdue = !row.2 && row.3 < now_timestamp;
            CalendarCard {
                id: row.0,
                title: row.1,
                done: row.2,
                due_date: row.3,
                list_id: row.4,
                list_title: row.5,
                board_id: row.6,
                board_title: row.7,
                is_overdue,
            }
        })
        .collect();

    Ok(Json(result))
}

/// Получить количество дней в месяце
fn get_days_in_month(year: i32, month: u32) -> u32 {
    use chrono::Datelike;

    // Переходим к следующему месяцу и вычитаем день
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("Некорректная дата первого дня следующего месяца")
        .pred_opt()
        .expect("Невозможно получить предыдущий день")
        .day()
}
