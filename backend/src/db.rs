// src/db.rs
use sqlx::SqlitePool;

pub async fn connect() -> Result<SqlitePool, sqlx::Error> {
    // Путь: ./data/trello.db (относительно cwd)
    std::fs::create_dir_all("data")?;
    let db_url = "sqlite://data/trello.db";
    let pool = SqlitePool::connect(db_url).await?;

    // Миграции
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS lists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            position REAL NOT NULL DEFAULT 0,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            position REAL NOT NULL DEFAULT 0,
            done BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}