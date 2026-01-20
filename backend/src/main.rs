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
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect().await?;

    let app = Router::new()
        .route("/api/boards", get(handlers::boards::get_boards).post(handlers::boards::create_board))
        .route("/api/boards/:id", patch(handlers::boards::update_board).delete(handlers::boards::delete_board))
        .route("/api/boards/:board_id/lists", post(handlers::lists::create_list))
        .route("/api/lists/:id", patch(handlers::lists::update_list).delete(handlers::lists::delete_list))
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

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
