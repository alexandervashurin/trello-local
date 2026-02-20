use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use sqlx::SqlitePool;
use tower::util::ServiceExt;

// Создаём тестовую базу данных в памяти
async fn create_test_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool")
}

// Инициализируем БД таблицами
async fn init_db(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );
        CREATE TABLE IF NOT EXISTS boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            owner_id INTEGER NOT NULL DEFAULT 1,
            is_shared BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS board_members (
            board_id INTEGER NOT NULL,
            user_id NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            PRIMARY KEY (board_id, user_id),
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
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
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create tables");

    // Создаём тестового пользователя
    sqlx::query("INSERT INTO users (id, username, created_at) VALUES (1, 'test', strftime('%s', 'now'))")
        .execute(pool)
        .await
        .expect("Failed to create test user");
}

#[tokio::test]
async fn test_create_board() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::boards;
    let app = axum::Router::new()
        .route("/api/boards", axum::routing::get(boards::get_boards).post(boards::create_board))
        .with_state(pool);

    // Создаём доску
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Тестовая доска", "is_shared": false }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_boards() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::boards;
    let app = axum::Router::new()
        .route("/api/boards", axum::routing::get(boards::get_boards).post(boards::create_board))
        .with_state(pool);

    // Создаём доску
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Тестовая доска", "is_shared": false }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Получаем список досок
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/boards")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_list() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::lists;
    use backend::models::Board;

    // Сначала создаём доску
    let board: Board = sqlx::query_as("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, 0) RETURNING *")
        .fetch_one(&pool)
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/api/boards/:board_id/lists", axum::routing::post(lists::create_list))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/boards/{}/lists", board.id))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Тестовый список" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_card() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::cards;
    use backend::models::{Board, List};

    // Создаём доску и список
    let board: Board = sqlx::query_as("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, 0) RETURNING *")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let list: List = sqlx::query_as("INSERT INTO lists (board_id, title, position) VALUES (?, 'Test List', 0) RETURNING *")
        .bind(board.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/api/lists/:list_id/cards", axum::routing::post(cards::create_card))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/lists/{}/cards", list.id))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "title": "Тестовая карточка", "content": "Описание" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_register() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::auth;
    let app = axum::Router::new()
        .route("/api/auth/register", axum::routing::post(auth::register))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "username": "newuser", "password": "password123" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_login() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::auth;

    // Создаём пользователя с паролем
    let password_hash = bcrypt::hash("password123", 12).unwrap();
    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind("loginuser")
        .bind(password_hash)
        .execute(&pool)
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/api/auth/login", axum::routing::post(auth::login))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "username": "loginuser", "password": "password123" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_search_boards() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::boards;

    // Создаём доски
    sqlx::query("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Project A', 1, 0)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Project B', 1, 0)").execute(&pool).await.unwrap();

    let app = axum::Router::new()
        .route("/api/boards", axum::routing::get(boards::get_boards))
        .with_state(pool);

    // Поиск по "Project"
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/boards?search=Project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_comments() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::models::{Board, List, Card};

    // Создаём доску, список и карточку
    let board: Board = sqlx::query_as("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, 0) RETURNING *")
        .fetch_one(&pool)
        .await
        .unwrap();

    let list: List = sqlx::query_as("INSERT INTO lists (board_id, title, position) VALUES (?, 'Test List', 0) RETURNING *")
        .bind(board.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let card: Card = sqlx::query_as("INSERT INTO cards (list_id, title, content, done) VALUES (?, 'Test Card', 'Content', 0) RETURNING *")
        .bind(list.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    // Создаём пользователя с паролем для JWT
    let password_hash = bcrypt::hash("password123", 12).unwrap();
    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (2, 'commenter', ?)")
        .bind(password_hash)
        .execute(&pool)
        .await
        .unwrap();

    // Тест создания комментария напрямую через БД
    let result: Result<(i64,), _> = sqlx::query_as(
        "INSERT INTO comments (card_id, user_id, content) VALUES (?, ?, ?) RETURNING id"
    )
    .bind(card.id)
    .bind(2)
    .bind("Тестовый комментарий")
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
}
