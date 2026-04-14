use sqlx::SqlitePool;
use std::path::PathBuf;

pub async fn connect() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    // Путь к базе данных (относительный для разработки)
    let db_path = PathBuf::from("data/trello.db");

    // Создаём директорию, если не существует
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Формируем URL с явным указанием создания БД если нет
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url).await?;

    // Миграции
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
            visibility TEXT NOT NULL DEFAULT 'private',
            FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS board_members (
            board_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            PRIMARY KEY (board_id, user_id),
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS board_permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            can_view BOOLEAN NOT NULL DEFAULT 1,
            can_create_cards BOOLEAN NOT NULL DEFAULT 1,
            can_edit_cards BOOLEAN NOT NULL DEFAULT 0,
            can_delete_cards BOOLEAN NOT NULL DEFAULT 0,
            can_move_cards BOOLEAN NOT NULL DEFAULT 0,
            can_create_lists BOOLEAN NOT NULL DEFAULT 0,
            can_edit_lists BOOLEAN NOT NULL DEFAULT 0,
            can_delete_lists BOOLEAN NOT NULL DEFAULT 0,
            can_manage_members BOOLEAN NOT NULL DEFAULT 0,
            can_manage_settings BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            UNIQUE(board_id, role)
        );
        CREATE TABLE IF NOT EXISTS board_invitations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL DEFAULT 'member',
            created_by INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            expires_at INTEGER,
            used BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
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
            due_date INTEGER,
            FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS card_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            done BOOLEAN NOT NULL DEFAULT 0,
            due_date INTEGER,
            list_id INTEGER NOT NULL,
            edited_by INTEGER NOT NULL,
            edited_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            change_summary TEXT NOT NULL,
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (edited_by) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_card_versions_card_id ON card_versions(card_id);
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS labels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'blue',
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS activity_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            user_id INTEGER,
            action_type TEXT NOT NULL,
            entity_type TEXT,
            entity_id INTEGER,
            description TEXT NOT NULL,
            metadata TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            user_agent TEXT,
            ip_address TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            expires_at INTEGER NOT NULL,
            last_activity INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS checklists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS checklist_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checklist_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            done BOOLEAN NOT NULL DEFAULT 0,
            position INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (checklist_id) REFERENCES checklists(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS card_assignees (
            card_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            assigned_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            assigned_by INTEGER NOT NULL,
            PRIMARY KEY (card_id, user_id),
            FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (assigned_by) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            notification_type TEXT NOT NULL DEFAULT 'info',
            is_read BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            link TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);
        CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_checklists_card_id ON checklists(card_id);
        CREATE INDEX IF NOT EXISTS idx_checklist_items_checklist_id ON checklist_items(checklist_id);
        CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
        CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read);
        CREATE TABLE IF NOT EXISTS board_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            is_public BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS board_template_lists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES board_templates(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS board_template_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (list_id) REFERENCES board_template_lists(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_board_templates_user_id ON board_templates(user_id);
        CREATE INDEX IF NOT EXISTS idx_board_template_lists_template_id ON board_template_lists(template_id);
        CREATE INDEX IF NOT EXISTS idx_board_template_cards_list_id ON board_template_cards(list_id);
        "#,
    )
    .execute(&pool)
    .await?;

    // Добавляем колонку password_hash если её нет (для существующих БД)
    sqlx::query("ALTER TABLE users ADD COLUMN password_hash TEXT")
        .execute(&pool)
        .await
        .ok(); // Игнорируем ошибку если колонка уже есть

    // Добавляем колонку visibility если её нет (для существующих БД)
    sqlx::query("ALTER TABLE boards ADD COLUMN visibility TEXT")
        .execute(&pool)
        .await
        .ok();

    // Обновляем существующие доски на 'private'
    sqlx::query("UPDATE boards SET visibility = 'private' WHERE visibility IS NULL")
        .execute(&pool)
        .await
        .ok();

    // Добавляем новые поля для управления профилями пользователей
    sqlx::query("ALTER TABLE users ADD COLUMN email TEXT")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("ALTER TABLE users ADD COLUMN avatar_color TEXT DEFAULT '#0079bf'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("ALTER TABLE users ADD COLUMN bio TEXT")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("ALTER TABLE users ADD COLUMN last_login INTEGER")
        .execute(&pool)
        .await
        .ok();

    // Добавляем поля для 2FA аутентификации
    sqlx::query("ALTER TABLE users ADD COLUMN two_factor_enabled BOOLEAN DEFAULT 0")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("ALTER TABLE users ADD COLUMN two_factor_secret TEXT")
        .execute(&pool)
        .await
        .ok();

    // Таблица для OAuth2 аккаунтов
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS oauth_accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            provider TEXT NOT NULL,
            provider_user_id TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            expires_at INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(provider, provider_user_id)
        )",
    )
    .execute(&pool)
    .await
    .ok();

    // Добавляем поля для OAuth2 в users
    sqlx::query("ALTER TABLE users ADD COLUMN oauth_enabled BOOLEAN DEFAULT 0")
        .execute(&pool)
        .await
        .ok();

    // Таблица для backup'ов
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            created_by INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            description TEXT,
            FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .ok();

    // Создаём пользователя по умолчанию
    sqlx::query(
        "INSERT OR IGNORE INTO users (id, username, created_at) VALUES (1, 'default', strftime('%s', 'now'))",
    )
    .execute(&pool)
    .await?;

    // Создаём тестовую доску для пользователя
    sqlx::query(
        "INSERT OR IGNORE INTO boards (id, title, owner_id, is_shared) VALUES (1, 'Моя первая доска', 1, 0)",
    )
    .execute(&pool)
    .await?;

    // Добавляем пользователя как владельца доски
    sqlx::query(
        "INSERT OR IGNORE INTO board_members (board_id, user_id, role) VALUES (1, 1, 'owner')",
    )
    .execute(&pool)
    .await?;

    // Инициализируем права по умолчанию для ролей
    sqlx::query(
        "INSERT OR IGNORE INTO board_permissions (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings)
         VALUES (1, 'owner', 1, 1, 1, 1, 1, 1, 1, 1, 1, 1)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO board_permissions (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings)
         VALUES (1, 'admin', 1, 1, 1, 1, 1, 1, 1, 1, 1, 0)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO board_permissions (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings)
         VALUES (1, 'member', 1, 1, 0, 0, 0, 0, 0, 0, 0, 0)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO board_permissions (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings)
         VALUES (1, 'viewer', 1, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
    )
    .execute(&pool)
    .await?;

    println!("✅ База данных подключена: {}", db_path.display());

    Ok(pool)
}
