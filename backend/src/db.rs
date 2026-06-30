use sqlx::postgres::PgPool;

pub async fn connect() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://trellouser@localhost/trello_db".to_string());

    let pool = PgPool::connect(&database_url).await?;

    // Миграции — каждый CREATE TABLE отдельным запросом
    let tables_sql = [
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
        "CREATE INDEX IF NOT EXISTS idx_card_versions_card_id ON card_versions(card_id)",
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
        "CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash)",
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)",
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
        "CREATE INDEX IF NOT EXISTS idx_checklists_card_id ON checklists(card_id)",
        "CREATE INDEX IF NOT EXISTS idx_checklist_items_checklist_id ON checklist_items(checklist_id)",
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
        "CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read)",
        "CREATE TABLE IF NOT EXISTS board_templates (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            is_public BOOLEAN NOT NULL DEFAULT FALSE,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
        "CREATE INDEX IF NOT EXISTS idx_board_templates_user_id ON board_templates(user_id)",
        "CREATE TABLE IF NOT EXISTS board_template_lists (
            id BIGSERIAL PRIMARY KEY,
            template_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            position BIGINT NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES board_templates(id) ON DELETE CASCADE
        )",
        "CREATE INDEX IF NOT EXISTS idx_board_template_lists_template_id ON board_template_lists(template_id)",
        "CREATE TABLE IF NOT EXISTS board_template_cards (
            id BIGSERIAL PRIMARY KEY,
            list_id BIGINT NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            position BIGINT NOT NULL DEFAULT 0,
            FOREIGN KEY (list_id) REFERENCES board_template_lists(id) ON DELETE CASCADE
        )",
        "CREATE INDEX IF NOT EXISTS idx_board_template_cards_list_id ON board_template_cards(list_id)",
        "CREATE TABLE IF NOT EXISTS oauth_accounts (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            provider TEXT NOT NULL,
            provider_user_id TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            expires_at BIGINT,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(provider, provider_user_id)
        )",
        "CREATE TABLE IF NOT EXISTS backups (
            id BIGSERIAL PRIMARY KEY,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size BIGINT NOT NULL,
            created_by BIGINT NOT NULL,
            created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
            description TEXT,
            FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
        )",
    ];

    for sql in &tables_sql {
        sqlx::query(sql).execute(&pool).await?;
    }

    // ALTER TABLE миграции
    let alter_sql = [
        "ALTER TABLE boards ADD COLUMN IF NOT EXISTS visibility TEXT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS email TEXT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_color TEXT DEFAULT '#0079bf'",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login BIGINT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_enabled BOOLEAN DEFAULT FALSE",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_secret TEXT",
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_enabled BOOLEAN DEFAULT FALSE",
        "ALTER TABLE lists ALTER COLUMN position TYPE DOUBLE PRECISION",
        "ALTER TABLE cards ALTER COLUMN position TYPE DOUBLE PRECISION",
    ];
    for sql in &alter_sql {
        sqlx::query(sql).execute(&pool).await.ok();
    }

    // Обновляем visibility для старых досок
    sqlx::query("UPDATE boards SET visibility = 'private' WHERE visibility IS NULL")
        .execute(&pool)
        .await
        .ok();

    // Создаём пользователя по умолчанию
    sqlx::query(
        "INSERT INTO users (id, username, created_at) VALUES (1, 'default', EXTRACT(EPOCH FROM NOW())::BIGINT) ON CONFLICT (id) DO NOTHING",
    )
    .execute(&pool)
    .await?;

    // Создаём тестовую доску
    sqlx::query(
        "INSERT INTO boards (id, title, owner_id, is_shared) VALUES (1, 'Моя первая доска', 1, FALSE) ON CONFLICT (id) DO NOTHING",
    )
    .execute(&pool)
    .await?;

    // Добавляем владельца
    sqlx::query(
        "INSERT INTO board_members (board_id, user_id, role) VALUES (1, 1, 'owner') ON CONFLICT (board_id, user_id) DO NOTHING",
    )
    .execute(&pool)
    .await?;

    // Инициализируем права по умолчанию
    let permissions = [
        (1i64, "owner",  true,  true,  true,  true,  true,  true,  true,  true,  true,  true),
        (1i64, "admin",  true,  true,  true,  true,  true,  true,  true,  true,  true,  false),
        (1i64, "member", true,  true,  false, false, false, false, false, false, false, false),
        (1i64, "viewer", true,  false, false, false, false, false, false, false, false, false),
    ];
    for (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings) in &permissions {
        sqlx::query(
            "INSERT INTO board_permissions (board_id, role, can_view, can_create_cards, can_edit_cards, can_delete_cards, can_move_cards, can_create_lists, can_edit_lists, can_delete_lists, can_manage_members, can_manage_settings)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT (board_id, role) DO NOTHING",
        )
        .bind(board_id)
        .bind(role)
        .bind(can_view)
        .bind(can_create_cards)
        .bind(can_edit_cards)
        .bind(can_delete_cards)
        .bind(can_move_cards)
        .bind(can_create_lists)
        .bind(can_edit_lists)
        .bind(can_delete_lists)
        .bind(can_manage_members)
        .bind(can_manage_settings)
        .execute(&pool)
        .await?;
    }

    // Сброс sequence после явных вставок id
    sqlx::query("SELECT setval('users_id_seq', COALESCE((SELECT MAX(id) FROM users), 0), true)")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("SELECT setval('boards_id_seq', COALESCE((SELECT MAX(id) FROM boards), 0), true)")
        .execute(&pool)
        .await
        .ok();

    println!("✅ База данных подключена: {} (PostgreSQL)", database_url);

    Ok(pool)
}
