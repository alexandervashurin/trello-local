// backend/src/main.rs
// MVC Architecture: Model-View-Controller
pub mod db;
pub mod models;
pub mod controllers;
pub mod views;
pub mod middleware;

use axum::{
    routing::{get, post, patch, delete},
    Router,
};
use axum::middleware::from_fn_with_state;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация tracing
    tracing_subscriber::fmt::init();

    let pool = db::connect().await?;

    // Создаём состояние для rate limiter
    let rate_limiter = middleware::rate_limit::RateLimiterState::with_defaults();
    let rate_limiter_clone = rate_limiter.clone();

    // Запускаем фоновую задачу для очистки старых записей
    tokio::spawn(async move {
        middleware::rate_limit::cleanup_old_entries(rate_limiter_clone).await;
    });

    // Абсолютные пути для frontend
    let frontend_dir = "/opt/trello-local/frontend";
    let index_html = "/opt/trello-local/frontend/index.html";
    let login_html = "/opt/trello-local/frontend/login.html";
    let invite_html = "/opt/trello-local/frontend/invite.html";

    // API роуты с middleware
    let api_routes = Router::new()
        // Auth (без аутентификации)
        .route("/auth/register", post(controllers::auth::register))
        .route("/auth/login", post(controllers::auth::login))
        // Сессии
        .route("/sessions", get(controllers::sessions::get_sessions).delete(controllers::sessions::delete_all_sessions))
        .route("/sessions/:id", delete(controllers::sessions::delete_session))
        // Пользователи
        .route("/users", get(controllers::users::get_users).post(controllers::users::create_user))
        .route("/users/:id", get(controllers::users::get_user))
        // Доски
        .route("/boards", get(controllers::boards::get_boards).post(controllers::boards::create_board))
        .route("/boards/:id", patch(controllers::boards::update_board).delete(controllers::boards::delete_board))
        .route("/boards/:board_id/members", get(controllers::boards::get_board_members).post(controllers::boards::add_board_member))
        .route("/boards/:board_id/members/:user_id", delete(controllers::boards::remove_board_member))
        .route("/boards/:board_id/members/:user_id/role", get(controllers::boards::get_user_board_role))
        .route("/boards/:board_id/invitations", get(controllers::boards::get_board_invitations).post(controllers::boards::create_invitation))
        .route("/boards/:board_id/invitations/:token", delete(controllers::boards::delete_invitation))
        .route("/invite/:token", post(controllers::boards::accept_invitation))
        .route("/users/:user_id/boards", get(controllers::boards::get_boards_for_user))
        // Списки
        .route("/boards/:board_id/lists", post(controllers::lists::create_list))
        .route("/lists/:id", patch(controllers::lists::update_list).delete(controllers::lists::delete_list))
        // Карточки
        .route("/lists/:list_id/cards", post(controllers::cards::create_card))
        .route("/cards/:id", patch(controllers::cards::update_card).delete(controllers::cards::delete_card))
        // Метки
        .route("/cards/:card_id/labels", get(controllers::cards::get_card_labels).post(controllers::cards::create_label))
        .route("/cards/:card_id/labels/:label_id", patch(controllers::cards::update_label).delete(controllers::cards::delete_label))
        // Вложения
        .route("/cards/:card_id/attachments", get(controllers::cards::get_card_attachments))
        .route("/cards/:card_id/attachments/:attachment_id", delete(controllers::cards::delete_attachment))
        .route("/cards/:card_id/boards/:board_id/attachments", post(controllers::attachments::upload_attachment))
        .route("/attachments/:attachment_id", get(controllers::attachments::download_attachment))
        // История активности
        .route("/boards/:board_id/activity", get(controllers::cards::get_activity_log))
        // Комментарии
        .route("/cards/:card_id/comments", get(controllers::comments::get_comments).post(controllers::comments::create_comment))
        .route("/comments/:id", patch(controllers::comments::update_comment).delete(controllers::comments::delete_comment))
        // Экспорт и статистика
        .route("/boards/:id/export/json", get(controllers::export::export_board_json))
        .route("/boards/:id/export/csv", get(controllers::export::export_board_csv))
        .route("/boards/:id/stats", get(controllers::export::get_board_stats))
        // Чек-листы
        .route("/cards/:card_id/checklists", get(controllers::checklists::get_card_checklists).post(controllers::checklists::create_checklist))
        .route("/cards/:card_id/checklists/:checklist_id", delete(controllers::checklists::delete_checklist))
        .route("/cards/:card_id/checklists/:checklist_id/items", post(controllers::checklists::create_checklist_item))
        .route("/cards/:card_id/checklists/:checklist_id/items/:item_id", patch(controllers::checklists::update_checklist_item).delete(controllers::checklists::delete_checklist_item))
        // Исполнители
        .route("/cards/:card_id/assignees", get(controllers::checklists::get_card_assignees).post(controllers::checklists::add_card_assignee))
        .route("/cards/:card_id/assignees/:user_id", delete(controllers::checklists::remove_card_assignee))
        // Применяем middleware для извлечения JWT claims
        .layer(from_fn_with_state(pool.clone(), middleware::auth::extract_claims))
        // Применяем rate limiter
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limit::rate_limit_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        // Страницы (Static Frontend)
        .nest_service("/login.html", ServeFile::new(login_html))
        .nest_service("/invite.html", ServeFile::new(invite_html))
        .fallback_service(
            ServeDir::new(frontend_dir)
                .fallback(ServeFile::new(index_html))
        )
        // Логирование запросов
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Trello Local запущен на http://{}", addr);
    println!("📁 База данных: ./data/trello.db");
    println!("🛑 Нажмите Ctrl+C для остановки сервера");

    let listener = TcpListener::bind(&addr).await?;
    
    // Запускаем сервер с обработкой Ctrl+C
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
            println!("\n👋 Остановка сервера...");
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("❌ Ошибка сервера: {}", e);
        });

    Ok(())
}
