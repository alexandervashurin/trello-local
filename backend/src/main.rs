// backend/src/main.rs
mod db;
mod models;
mod handlers;

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

    let app = Router::new()
        // Пользователи
        .route("/api/users", get(handlers::users::get_users).post(handlers::users::create_user))
        .route("/api/users/:id", get(handlers::users::get_user))
        // Доски
        .route("/api/boards", get(handlers::boards::get_boards).post(handlers::boards::create_board))
        .route("/api/boards/:id", patch(handlers::boards::update_board).delete(handlers::boards::delete_board))
        .route("/api/boards/:board_id/members", get(handlers::boards::get_board_members).post(handlers::boards::add_board_member))
        .route("/api/boards/:board_id/members/:user_id", delete(handlers::boards::remove_board_member))
        .route("/api/users/:user_id/boards", get(handlers::boards::get_boards_for_user))
        // Списки
        .route("/api/boards/:board_id/lists", post(handlers::lists::create_list))
        .route("/api/lists/:id", patch(handlers::lists::update_list).delete(handlers::lists::delete_list))
        // Карточки
        .route("/api/lists/:list_id/cards", post(handlers::cards::create_card))
        .route("/api/cards/:id", patch(handlers::cards::update_card).delete(handlers::cards::delete_card))
        .fallback_service(
            ServeDir::new("frontend")
                .fallback(ServeFile::new("../frontend/index.html"))
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
