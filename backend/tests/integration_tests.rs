use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use sqlx::PgPool;
use tower::util::ServiceExt;

async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://trellouser@localhost/trello_db".to_string());
    PgPool::connect(&database_url)
        .await
        .expect("Failed to create test pool")
}

async fn init_db(pool: &PgPool) {
    // Drop existing tables in reverse dependency order
    let drop_tables = [
        "DROP TABLE IF EXISTS board_template_cards CASCADE",
        "DROP TABLE IF EXISTS board_template_lists CASCADE",
        "DROP TABLE IF EXISTS board_templates CASCADE",
        "DROP TABLE IF EXISTS notifications CASCADE",
        "DROP TABLE IF EXISTS card_assignees CASCADE",
        "DROP TABLE IF EXISTS checklist_items CASCADE",
        "DROP TABLE IF EXISTS checklists CASCADE",
        "DROP TABLE IF EXISTS sessions CASCADE",
        "DROP TABLE IF EXISTS activity_log CASCADE",
        "DROP TABLE IF EXISTS attachments CASCADE",
        "DROP TABLE IF EXISTS labels CASCADE",
        "DROP TABLE IF EXISTS comments CASCADE",
        "DROP TABLE IF EXISTS card_versions CASCADE",
        "DROP TABLE IF EXISTS cards CASCADE",
        "DROP TABLE IF EXISTS lists CASCADE",
        "DROP TABLE IF EXISTS board_invitations CASCADE",
        "DROP TABLE IF EXISTS board_permissions CASCADE",
        "DROP TABLE IF EXISTS board_members CASCADE",
        "DROP TABLE IF EXISTS boards CASCADE",
        "DROP TABLE IF EXISTS oauth_accounts CASCADE",
        "DROP TABLE IF EXISTS backups CASCADE",
        "DROP TABLE IF EXISTS users CASCADE",
    ];
    for sql in &drop_tables {
        sqlx::query(sql).execute(pool).await.ok();
    }

    let create_tables = [
        "CREATE TABLE IF NOT EXISTS users (
            id BIGSERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT,
            email TEXT,
            avatar_color TEXT DEFAULT '#0079bf',
            bio TEXT,
            last_login BIGINT,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            two_factor_enabled BOOLEAN DEFAULT FALSE,
            two_factor_secret TEXT,
            oauth_enabled BOOLEAN DEFAULT FALSE
        )",
        "CREATE TABLE IF NOT EXISTS boards (
            id BIGSERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            owner_id BIGINT NOT NULL DEFAULT 1,
            is_shared BOOLEAN NOT NULL DEFAULT FALSE,
            visibility TEXT NOT NULL DEFAULT 'private',
            FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS board_members (
            board_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            PRIMARY KEY (board_id, user_id),
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS board_permissions (
            id BIGSERIAL PRIMARY KEY,
            board_id BIGINT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            can_view BOOLEAN NOT NULL DEFAULT TRUE,
            can_create_cards BOOLEAN NOT NULL DEFAULT TRUE,
            can_edit_cards BOOLEAN NOT NULL DEFAULT FALSE,
            can_delete_cards BOOLEAN NOT NULL DEFAULT FALSE,
            can_move_cards BOOLEAN NOT NULL DEFAULT FALSE,
            can_create_lists BOOLEAN NOT NULL DEFAULT FALSE,
            can_edit_lists BOOLEAN NOT NULL DEFAULT FALSE,
            can_delete_lists BOOLEAN NOT NULL DEFAULT FALSE,
            can_manage_members BOOLEAN NOT NULL DEFAULT FALSE,
            can_manage_settings BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            UNIQUE(board_id, role)
        )",
        "CREATE TABLE IF NOT EXISTS board_invitations (
            id BIGSERIAL PRIMARY KEY,
            board_id BIGINT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL DEFAULT 'member',
            created_by BIGINT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            expires_at BIGINT,
            used BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS lists (
            id BIGSERIAL PRIMARY KEY,
            board_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            position DOUBLE PRECISION NOT NULL DEFAULT 0,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS cards (
            id BIGSERIAL PRIMARY KEY,
            list_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            position DOUBLE PRECISION NOT NULL DEFAULT 0,
            done BOOLEAN NOT NULL DEFAULT FALSE,
            due_date BIGINT,
            FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS card_versions (
            id BIGSERIAL PRIMARY KEY,
            card_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            done BOOLEAN NOT NULL DEFAULT FALSE,
            due_date BIGINT,
            list_id BIGINT NOT NULL,
            edited_by BIGINT NOT NULL,
            edited_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            change_summary TEXT NOT NULL,
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (edited_by) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS comments (
            id BIGSERIAL PRIMARY KEY,
            card_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            content TEXT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS labels (
            id BIGSERIAL PRIMARY KEY,
            card_id BIGINT NOT NULL,
            name TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'blue',
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS attachments (
            id BIGSERIAL PRIMARY KEY,
            card_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size BIGINT NOT NULL,
            mime_type TEXT,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS activity_log (
            id BIGSERIAL PRIMARY KEY,
            board_id BIGINT NOT NULL,
            user_id BIGINT,
            action_type TEXT NOT NULL,
            entity_type TEXT,
            entity_id BIGINT,
            description TEXT NOT NULL,
            metadata TEXT,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        )",
        "CREATE TABLE IF NOT EXISTS sessions (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            user_agent TEXT,
            ip_address TEXT,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            expires_at BIGINT NOT NULL,
            last_activity BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS checklists (
            id BIGSERIAL PRIMARY KEY,
            card_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            position BIGINT NOT NULL DEFAULT 0,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS checklist_items (
            id BIGSERIAL PRIMARY KEY,
            checklist_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL DEFAULT FALSE,
            position BIGINT NOT NULL DEFAULT 0,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (checklist_id) REFERENCES checklists(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS card_assignees (
            card_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            assigned_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            assigned_by BIGINT NOT NULL,
            PRIMARY KEY (card_id, user_id),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (assigned_by) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE TABLE IF NOT EXISTS notifications (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            notification_type TEXT NOT NULL DEFAULT 'info',
            is_read BOOLEAN NOT NULL DEFAULT FALSE,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            link TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
    ];
    for sql in &create_tables {
        sqlx::query(sql).execute(pool).await.expect("Failed to create table");
    }

    sqlx::query(
        "INSERT INTO users (id, username, created_at) VALUES (1, 'test', EXTRACT(EPOCH FROM NOW())::BIGINT) ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    // Сброс sequence после явной вставки id=1
    sqlx::query("SELECT setval('users_id_seq', 1, true)")
        .execute(pool)
        .await
        .ok();
    sqlx::query("SELECT setval('boards_id_seq', COALESCE((SELECT MAX(id) FROM boards), 0), true)")
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_create_board() {
    let pool = create_test_pool().await;
    init_db(&pool).await;

    use backend::controllers::boards;
    let app: axum::Router = axum::Router::new()
        .route(
            "/api/boards",
            axum::routing::get(boards::get_boards).post(boards::create_board),
        )
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "title": "Тестовая доска", "is_shared": false }).to_string(),
                ))
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
    let app: axum::Router = axum::Router::new()
        .route(
            "/api/boards",
            axum::routing::get(boards::get_boards).post(boards::create_board),
        )
        .with_state(pool);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "title": "Тестовая доска", "is_shared": false }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

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

    let board: Board = sqlx::query_as(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, FALSE) RETURNING *",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let app: axum::Router = axum::Router::new()
        .route(
            "/api/boards/:board_id/lists",
            axum::routing::post(lists::create_list),
        )
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/boards/{}/lists", board.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "title": "Тестовый список" }).to_string(),
                ))
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

    let board: Board = sqlx::query_as(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, FALSE) RETURNING *",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let list: List = sqlx::query_as(
        "INSERT INTO lists (board_id, title, position) VALUES ($1, 'Test List', 0) RETURNING *",
    )
    .bind(board.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let app: axum::Router = axum::Router::new()
        .route(
            "/api/lists/:list_id/cards",
            axum::routing::post(cards::create_card),
        )
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/lists/{}/cards", list.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "title": "Тестовая карточка", "content": "Описание" }).to_string(),
                ))
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
    let app: axum::Router = axum::Router::new()
        .route("/api/auth/register", axum::routing::post(auth::register))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "username": "newuser", "password": "Password123" }).to_string(),
                ))
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

    let password_hash = bcrypt::hash("password123", 12).unwrap();
    sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
        .bind("loginuser")
        .bind(password_hash)
        .execute(&pool)
        .await
        .unwrap();

    let app: axum::Router = axum::Router::new()
        .route("/api/auth/login", axum::routing::post(auth::login))
        .with_state(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "username": "loginuser", "password": "password123" }).to_string(),
                ))
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

    sqlx::query("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Project A', 1, FALSE)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO boards (title, owner_id, is_shared) VALUES ('Project B', 1, FALSE)")
        .execute(&pool)
        .await
        .unwrap();

    let app: axum::Router = axum::Router::new()
        .route("/api/boards", axum::routing::get(boards::get_boards))
        .with_state(pool);

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

    use backend::models::{Board, Card, List};

    let board: Board = sqlx::query_as(
        "INSERT INTO boards (title, owner_id, is_shared) VALUES ('Test', 1, FALSE) RETURNING *",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let list: List = sqlx::query_as(
        "INSERT INTO lists (board_id, title, position) VALUES ($1, 'Test List', 0) RETURNING *",
    )
    .bind(board.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let card: Card = sqlx::query_as("INSERT INTO cards (list_id, title, content, done) VALUES ($1, 'Test Card', 'Content', FALSE) RETURNING *")
        .bind(list.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let password_hash = bcrypt::hash("password123", 12).unwrap();
    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (2, 'commenter', $1)")
        .bind(password_hash)
        .execute(&pool)
        .await
        .unwrap();

    let result: Result<(i64,), _> = sqlx::query_as(
        "INSERT INTO comments (card_id, user_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(card.id)
    .bind(2i64)
    .bind("Тестовый комментарий")
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
}
