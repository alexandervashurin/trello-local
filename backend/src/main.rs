// backend/src/main.rs
// MVC Architecture: Model-View-Controller
pub mod db;
pub mod models;
pub mod controllers;
pub mod views;

use axum::{
    routing::{get, post, patch, delete},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect().await?;

    // Абсолютные пути для frontend
    let frontend_dir = "/opt/trello-local/frontend";
    let index_html = "/opt/trello-local/frontend/index.html";
    let login_html = "/opt/trello-local/frontend/login.html";

    let app = Router::new()
        // Auth (Authentication Controller)
        .route("/api/auth/register", post(controllers::auth::register))
        .route("/api/auth/login", post(controllers::auth::login))
        // Пользователи (Users Controller)
        .route("/api/users", get(controllers::users::get_users).post(controllers::users::create_user))
        .route("/api/users/:id", get(controllers::users::get_user))
        // Доски (Boards Controller)
        .route("/api/boards", get(controllers::boards::get_boards).post(controllers::boards::create_board))
        .route("/api/boards/:id", patch(controllers::boards::update_board).delete(controllers::boards::delete_board))
        .route("/api/boards/:board_id/members", get(controllers::boards::get_board_members).post(controllers::boards::add_board_member))
        .route("/api/boards/:board_id/members/:user_id", delete(controllers::boards::remove_board_member))
        .route("/api/users/:user_id/boards", get(controllers::boards::get_boards_for_user))
        // Списки (Lists Controller)
        .route("/api/boards/:board_id/lists", post(controllers::lists::create_list))
        .route("/api/lists/:id", patch(controllers::lists::update_list).delete(controllers::lists::delete_list))
        // Карточки (Cards Controller)
        .route("/api/lists/:list_id/cards", post(controllers::cards::create_card))
        .route("/api/cards/:id", patch(controllers::cards::update_card).delete(controllers::cards::delete_card))
        // Страницы (Static Frontend)
        .nest_service("/login.html", ServeFile::new(login_html))
        .fallback_service(
            ServeDir::new(frontend_dir)
                .fallback(ServeFile::new(index_html))
        )
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
