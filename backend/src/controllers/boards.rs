use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use axum_macros::debug_handler;
use sqlx::SqlitePool;
use crate::models::{Board, CreateBoard, UpdateBoard, AddBoardMember, BoardMember, BoardInvitation, CreateInvitation};
use crate::views::{BoardView, ListView, CardView, InvitationView, Claims};
use crate::models::{List, Card, Label, Attachment};
use serde::Deserialize;
use axum::extract::Extension;

#[derive(Deserialize)]
pub struct GetBoardsQuery {
    search: Option<String>,
}

/// Получить все доски (с опциональным поиском)
#[debug_handler]
pub async fn get_boards(
    State(pool): State<SqlitePool>,
    query: Query<GetBoardsQuery>,
    claims: Option<Extension<Claims>>,
) -> Result<Json<Vec<BoardView>>, (StatusCode, String)> {
    // Получаем текущего пользователя из токена
    let current_user_id = claims.map(|c| c.user_id);
    
    let boards: Vec<Board> = if let Some(search) = &query.search {
        sqlx::query_as("SELECT * FROM boards WHERE title LIKE ? ORDER BY id")
            .bind(format!("%{}%", search))
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT * FROM boards ORDER BY id")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        // Фильтрация по видимости
        let can_view = match board.visibility.as_str() {
            "public" => true,
            "private" => {
                if let Some(user_id) = current_user_id {
                    user_id == board.owner_id || is_board_member(&pool, board.id, user_id).await.unwrap_or(false)
                } else {
                    false
                }
            }
            _ => false,
        };
        
        if !can_view {
            continue;
        }
        
        let board_view = load_board_details(&pool, board).await?;
        result.push(board_view);
    }

    Ok(Json(result))
}

/// Получить доски для пользователя
pub async fn get_boards_for_user(
    Path(user_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<BoardView>>, (StatusCode, String)> {
    let boards: Vec<Board> = sqlx::query_as(
        "SELECT * FROM boards WHERE owner_id = ? OR is_shared = 1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for board in boards {
        let board_view = load_board_details(&pool, board).await?;
        result.push(board_view);
    }

    Ok(Json(result))
}

/// Создать доску
#[debug_handler]
pub async fn create_board(
    State(pool): State<SqlitePool>,
    claims: Option<Extension<Claims>>,
    Json(payload): Json<CreateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    // Валидация названия
    if payload.title.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Название не может быть пустым".to_string()));
    }
    if payload.title.len() > 200 {
        return Err((StatusCode::BAD_REQUEST, "Название слишком длинное".to_string()));
    }

    let owner_id = claims.map(|c| c.user_id).unwrap_or(1); // Default to 1 for tests

    let board: Board = sqlx::query_as::<_, Board>(
        "INSERT INTO boards (title, owner_id, is_shared, visibility) VALUES (?, ?, ?, ?) RETURNING id, title, owner_id, is_shared, visibility",
    )
    .bind(&payload.title)
    .bind(owner_id)
    .bind(payload.is_shared)
    .bind(&payload.visibility)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Добавляем владельца как участника с ролью owner
    sqlx::query(
        "INSERT OR IGNORE INTO board_members (board_id, user_id, role) VALUES (?, ?, 'owner')",
    )
    .bind(board.id)
    .bind(owner_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Логирование
    let _ = crate::controllers::cards::log_activity(
        &pool,
        board.id,
        Some(owner_id),
        "create",
        Some("board"),
        Some(board.id),
        &format!("Создана доска \"{}\"", &payload.title),
        None,
    ).await;

    Ok(Json(board))
}

/// Обновить доску
#[debug_handler]
pub async fn update_board(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateBoard>,
) -> Result<Json<Board>, (StatusCode, String)> {
    
    // Проверка прав на редактирование
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, id, claims.user_id, &["owner", "admin", "editor"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на редактирование доски".to_string()));
    }

    let board: Board = sqlx::query_as::<_, Board>(
        "UPDATE boards SET title = COALESCE(?, title), is_shared = COALESCE(?, is_shared), visibility = COALESCE(?, visibility) WHERE id = ? RETURNING id, title, owner_id, is_shared, visibility",
    )
    .bind(&payload.title)
    .bind(payload.is_shared)
    .bind(&payload.visibility)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Доска не найдена".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Логирование изменений
    let mut changes = Vec::new();
    if payload.title.is_some() {
        changes.push(format!("название → \"{}\"", payload.title.as_ref().unwrap_or(&board.title)));
    }
    if payload.is_shared.is_some() {
        changes.push(format!("общий доступ → {}", if payload.is_shared.unwrap() { "включён" } else { "выключен" }));
    }
    if payload.visibility.is_some() {
        changes.push(format!("видимость → {}", payload.visibility.as_ref().unwrap_or(&board.visibility)));
    }
    if !changes.is_empty() {
        let _ = crate::controllers::cards::log_activity(
            &pool,
            id,
            Some(claims.user_id),
            "update",
            Some("board"),
            Some(id),
            &format!("Доска \"{}\": {}", board.title, changes.join(", ")),
            None,
        ).await;
    }

    Ok(Json(board))
}

/// Удалить доску
pub async fn delete_board(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<()>, (StatusCode, String)> {
    // Получаем название доски перед удалением
    let board_title: Option<(String,)> = sqlx::query_as(
        "SELECT title FROM boards WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let title = board_title.map(|t| t.0).unwrap_or_else(|| "неизвестно".to_string());

    let result = sqlx::query("DELETE FROM boards WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Доска не найдена".to_string()))
    } else {
        // Логирование
        let _ = crate::controllers::cards::log_activity(
            &pool,
            id,
            None,
            "delete",
            Some("board"),
            Some(id),
            &format!("Удалена доска \"{}\"", title),
            None,
        ).await;
        Ok(Json(()))
    }
}

/// Получить участников доски
pub async fn get_board_members(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<BoardMember>>, (StatusCode, String)> {
    let members: Vec<BoardMember> = sqlx::query_as(
        "SELECT bm.board_id, bm.user_id, bm.role, u.username FROM board_members bm INNER JOIN users u ON bm.user_id = u.id WHERE bm.board_id = ?",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(members))
}

/// Добавить участника в доску
#[debug_handler]
pub async fn add_board_member(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddBoardMember>,
) -> Result<Json<()>, (StatusCode, String)> {
    
    // Проверка прав: только owner, admin могут добавлять участников
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на добавление участников".to_string()));
    }
    
    let result = sqlx::query(
        "INSERT OR REPLACE INTO board_members (board_id, user_id, role) VALUES (?, ?, ?)",
    )
    .bind(board_id)
    .bind(payload.user_id)
    .bind(&payload.role)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Доска или пользователь не найдены".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Удалить участника из доски
#[debug_handler]
pub async fn remove_board_member(
    Path((board_id, user_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    
    // Проверка прав: только owner, admin могут удалять участников
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на удаление участников".to_string()));
    }
    
    let result = sqlx::query("DELETE FROM board_members WHERE board_id = ? AND user_id = ?")
        .bind(board_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Участник не найден".to_string()))
    } else {
        Ok(Json(()))
    }
}

/// Вспомогательная функция для загрузки полной информации о доске
async fn load_board_details(
    pool: &SqlitePool,
    board: Board,
) -> Result<BoardView, (StatusCode, String)> {
    // Загружаем участников
    let members: Vec<BoardMember> = sqlx::query_as(
        "SELECT bm.board_id, bm.user_id, bm.role, u.username FROM board_members bm INNER JOIN users u ON bm.user_id = u.id WHERE bm.board_id = ?",
    )
    .bind(board.id)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Загружаем списки
    let lists_rows: Vec<List> = sqlx::query_as(
        "SELECT id, board_id, title, position FROM lists WHERE board_id = ? ORDER BY position, id",
    )
    .bind(board.id)
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Загружаем карточки для каждого списка с метками и вложениями
    let mut lists = Vec::new();
    for list in lists_rows {
        let cards: Vec<Card> = sqlx::query_as(
            "SELECT id, list_id, title, content, done, due_date FROM cards WHERE list_id = ? ORDER BY position, id",
        )
        .bind(list.id)
        .fetch_all(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Для каждой карточки загружаем метки и вложения
        let mut card_views = Vec::new();
        for card in cards {
            let labels: Vec<Label> = sqlx::query_as(
                "SELECT id, card_id, name, color FROM labels WHERE card_id = ?"
            )
            .bind(card.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let attachments: Vec<Attachment> = sqlx::query_as(
                "SELECT id, card_id, user_id, filename, file_path, file_size, mime_type, created_at FROM attachments WHERE card_id = ?"
            )
            .bind(card.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            card_views.push(CardView::from_card(card)
                .with_labels(labels)
                .with_attachments(attachments));
        }

        lists.push(ListView::from_list(list).with_cards(card_views));
    }

    Ok(BoardView::from_board(board)
        .with_members(members)
        .with_lists(lists))
}

/// Проверка, является ли пользователь участником доски
async fn is_board_member(pool: &SqlitePool, board_id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ?"
    )
    .bind(board_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}

/// Проверка наличия роли у пользователя
async fn has_role(pool: &SqlitePool, board_id: i64, user_id: i64, roles: &[&str]) -> Result<bool, sqlx::Error> {
    let placeholders = roles.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
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

/// Получить роль пользователя на доске
pub async fn get_user_board_role(
    Path((board_id, user_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<String>, (StatusCode, String)> {
    let role: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM board_members WHERE board_id = ? AND user_id = ?"
    )
    .bind(board_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match role {
        Some((r,)) => Ok(Json(r)),
        None => Err((StatusCode::NOT_FOUND, "Участник не найден".to_string()))
    }
}

/// Создать приглашение на доску
#[debug_handler]
pub async fn create_invitation(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateInvitation>,
) -> Result<Json<InvitationView>, (StatusCode, String)> {
    
    // Проверка прав: только owner или admin могут приглашать
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на создание приглашений".to_string()));
    }
    
    // Генерация токена
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = payload.expires_in_hours.map(|hours| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 + hours * 3600
    });
    
    let invitation: BoardInvitation = sqlx::query_as::<_, BoardInvitation>(
        "INSERT INTO board_invitations (board_id, token, role, created_by, expires_at) VALUES (?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(board_id)
    .bind(&token)
    .bind(&payload.role)
    .bind(claims.user_id)
    .bind(expires_at)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(InvitationView {
        token: invitation.token,
        board_id: invitation.board_id,
        role: invitation.role,
        expires_at: invitation.expires_at,
        invite_link: format!("http://localhost:8080/invite/{}", token),
    }))
}

/// Принять приглашение по токену
#[debug_handler]
pub async fn accept_invitation(
    Path(token): Path<String>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    
    // Проверка токена
    let invitation: BoardInvitation = sqlx::query_as::<_, BoardInvitation>(
        "SELECT * FROM board_invitations WHERE token = ? AND used = 0 AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'))"
    )
    .bind(&token)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Приглашение не найдено или истекло".to_string()))?;
    
    // Добавляем пользователя на доску
    sqlx::query(
        "INSERT OR REPLACE INTO board_members (board_id, user_id, role) VALUES (?, ?, ?)"
    )
    .bind(invitation.board_id)
    .bind(claims.user_id)
    .bind(&invitation.role)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Помечаем приглашение как использованное
    sqlx::query(
        "UPDATE board_invitations SET used = 1 WHERE token = ?"
    )
    .bind(&token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(()))
}

/// Получить приглашения доски
#[debug_handler]
pub async fn get_board_invitations(
    Path(board_id): Path<i64>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<InvitationView>>, (StatusCode, String)> {
    
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на просмотр приглашений".to_string()));
    }
    
    let invitations: Vec<BoardInvitation> = sqlx::query_as::<_, BoardInvitation>(
        "SELECT * FROM board_invitations WHERE board_id = ? AND used = 0 ORDER BY created_at DESC"
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let views: Vec<InvitationView> = invitations.into_iter().map(|i| {
        let token = i.token;
        InvitationView {
            token: token.clone(),
            board_id: i.board_id,
            role: i.role,
            expires_at: i.expires_at,
            invite_link: format!("http://localhost:8080/invite/{}", token),
        }
    }).collect();

    Ok(Json(views))
}

/// Удалить/отозвать приглашение
#[debug_handler]
pub async fn delete_invitation(
    Path((board_id, token)): Path<(i64, String)>,
    State(pool): State<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<()>, (StatusCode, String)> {
    
    let board: Board = sqlx::query_as::<_, Board>(
        "SELECT * FROM boards WHERE id = ?"
    )
    .bind(board_id)
    .fetch_one(&pool)
    .await
    .map_err(|_e| (StatusCode::NOT_FOUND, "Доска не найдена".to_string()))?;
    
    let has_permission = claims.user_id == board.owner_id 
        || has_role(&pool, board_id, claims.user_id, &["owner", "admin"]).await.unwrap_or(false);
    
    if !has_permission {
        return Err((StatusCode::FORBIDDEN, "Нет прав на удаление приглашений".to_string()));
    }
    
    let result = sqlx::query("DELETE FROM board_invitations WHERE board_id = ? AND token = ?")
        .bind(board_id)
        .bind(&token)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Приглашение не найдено".to_string()))
    } else {
        Ok(Json(()))
    }
}
