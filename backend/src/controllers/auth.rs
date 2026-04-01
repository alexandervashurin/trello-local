use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{RegisterUser, LoginUser, User, TwoFASetup, TwoFACode, TwoFAEnable, TwoFAStatus, TwoFATempToken};
use crate::views::{AuthToken, Claims, ClaimsWith2FA, TwoFATempTokenResponse};
use jsonwebtoken::{encode, Header, EncodingKey, Validation, Algorithm, DecodingKey, decode};
use std::time::SystemTime;

use crate::controllers::sessions;

/// Получение JWT secret из переменной окружения
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .inspect_err(|_| {
            tracing::warn!("JWT_SECRET не установлен! Используйте уникальное значение в production");
        })
        .unwrap_or_else(|_| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Время не может идти вспять")
                .as_nanos();
            format!("dev-secret-{}", seed)
        })
        .into_bytes()
}

/// Генерация JWT токена
fn generate_token(user_id: i64, username: &str, two_factor_verified: bool) -> Result<String, (StatusCode, String)> {
    let expiration = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() + 60 * 60 * 24 * 7; // 7 дней

    let claims = ClaimsWith2FA {
        user_id,
        username: username.to_string(),
        exp: expiration as usize,
        two_factor_verified,
        user_agent: None,
        ip_address: None,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(&get_jwt_secret()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Генерация временного токена для 2FA (короткое время жизни)
fn generate_temp_token(user_id: i64, username: &str) -> Result<String, (StatusCode, String)> {
    let expiration = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Время не может идти вспять")
        .as_secs() + 300; // 5 минут

    let claims = ClaimsWith2FA {
        user_id,
        username: username.to_string(),
        exp: expiration as usize,
        two_factor_verified: false,
        user_agent: None,
        ip_address: None,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(&get_jwt_secret()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Регистрация нового пользователя
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    // Валидация имени пользователя
    if payload.username.trim().len() < 3 {
        return Err((StatusCode::BAD_REQUEST, "Имя пользователя должно быть не менее 3 символов".to_string()));
    }
    if payload.username.len() > 50 {
        return Err((StatusCode::BAD_REQUEST, "Имя пользователя слишком длинное".to_string()));
    }

    // Валидация пароля
    if payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Пароль должен быть не менее 8 символов".to_string()));
    }

    // Проверка сложности пароля
    let has_upper = payload.password.chars().any(|c| c.is_uppercase());
    let has_lower = payload.password.chars().any(|c| c.is_lowercase());
    let has_digit = payload.password.chars().any(|c| c.is_numeric());

    if !has_upper || !has_lower || !has_digit {
        return Err((StatusCode::BAD_REQUEST,
            "Пароль должен содержать заглавные и строчные буквы, а также цифры".to_string()));
    }

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, created_at, two_factor_enabled) VALUES (?, ?, strftime('%s', 'now'), 0) RETURNING id, username, email, avatar_color, bio, last_login, created_at, two_factor_enabled, two_factor_secret",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("UNIQUE constraint failed") {
            (StatusCode::CONFLICT, "Пользователь с таким именем уже существует".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    let token = generate_token(user.id, &user.username, true)?;

    // Сохраняем сессию
    let _ = sessions::save_session(&pool, user.id, &token, None, None).await;

    // Логирование успешной регистрации
    tracing::info!(
        target: "security",
        username = %payload.username,
        user_id = user.id,
        event = "register_success",
        "Успешная регистрация нового пользователя"
    );

    Ok(Json(AuthToken {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// Вход пользователя
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_with_password: crate::models::UserWithPassword = sqlx::query_as(
        "SELECT id, username, password_hash, email, avatar_color, bio, last_login, created_at, two_factor_enabled, two_factor_secret FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        tracing::warn!(
            target: "security",
            username = %payload.username,
            event = "login_failed",
            reason = "user_not_found",
            "Неудачная попытка входа: пользователь не найден"
        );
        (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string())
    })?;

    let valid = bcrypt::verify(&payload.password, &user_with_password.password_hash)
        .map_err(|e| {
            tracing::error!(
                target: "security",
                username = %payload.username,
                event = "login_error",
                reason = "bcrypt_error",
                error = %e,
                "Ошибка при проверке пароля"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if !valid {
        tracing::warn!(
            target: "security",
            username = %payload.username,
            user_id = user_with_password.id,
            event = "login_failed",
            reason = "invalid_password",
            "Неудачная попытка входа: неверный пароль"
        );
        return Err((StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль".to_string()));
    }

    // Обновляем last_login
    let _ = sqlx::query("UPDATE users SET last_login = strftime('%s', 'now') WHERE id = ?")
        .bind(user_with_password.id)
        .execute(&pool)
        .await;

    // Логирование успешной проверки пароля
    tracing::info!(
        target: "security",
        username = %payload.username,
        user_id = user_with_password.id,
        event = "login_password_success",
        "Успешная проверка пароля"
    );

    // Проверяем, включен ли 2FA
    let two_factor_enabled = user_with_password.two_factor_enabled.unwrap_or(false);
    
    if two_factor_enabled {
        // Генерируем временный токен для прохождения 2FA
        let temp_token = generate_temp_token(user_with_password.id, &user_with_password.username)?;
        
        return Ok(Json(serde_json::json!({
            "requires_2fa": true,
            "temp_token": temp_token,
            "user_id": user_with_password.id,
            "username": user_with_password.username
        })));
    }

    // Если 2FA не включен, выдаем полный токен
    let token = generate_token(user_with_password.id, &user_with_password.username, true)?;

    // Сохраняем сессию
    let _ = sessions::save_session(&pool, user_with_password.id, &token, None, None).await;

    tracing::info!(
        target: "security",
        username = %payload.username,
        user_id = user_with_password.id,
        event = "login_success",
        "Успешный вход пользователя"
    );

    Ok(Json(serde_json::json!({
        "requires_2fa": false,
        "token": token,
        "user_id": user_with_password.id,
        "username": user_with_password.username
    })))
}

/// Валидация токена
pub async fn validate_token(
    State(_pool): State<SqlitePool>,
    token: String,
) -> Result<Json<Claims>, (StatusCode, String)> {
    let result = decode::<ClaimsWith2FA>(
        &token,
        &DecodingKey::from_secret(&get_jwt_secret()),
        &Validation::new(Algorithm::HS256)
    );

    match result {
        Ok(data) => {
            let claims = Claims {
                user_id: data.claims.user_id,
                username: data.claims.username,
                exp: data.claims.exp,
                user_agent: data.claims.user_agent,
                ip_address: data.claims.ip_address,
            };
            Ok(Json(claims))
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Неверный токен".to_string()))
    }
}

/// Генерация 2FA setup для пользователя
pub async fn generate_2fa_setup(
    State(_pool): State<SqlitePool>,
    Json(_payload): Json<TwoFACode>,
) -> Result<Json<TwoFASetup>, (StatusCode, String)> {
    // Генерируем случайный секрет
    let secret_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let secret_base32 = base32::encode(base32::Alphabet::Crockford, &secret_bytes);
    
    // Создаем TOTP объект
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Trello Local".to_string()),
        "user@trello.local".to_string(),
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка создания TOTP: {}", e)))?;
    
    // Генерируем URI для Google Authenticator
    let uri = totp.get_url();
    
    // Генерируем QR код в PNG format
    let qr_image = qrcode::QrCode::new(&uri)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка генерации QR: {}", e)))?
        .render::<image::Luma<u8>>()
        .build();
    
    // Конвертируем в base64
    let mut png_data = Vec::new();
    use std::io::Cursor;
    qr_image.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка создания PNG: {}", e)))?;
    
    use base64::Engine;
    let qr_string = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png_data));
    
    Ok(Json(TwoFASetup {
        secret: secret_base32,
        uri,
        qr_code: qr_string,
    }))
}

/// Проверка и включение 2FA
pub async fn enable_2fa(
    State(pool): State<SqlitePool>,
    Json(payload): Json<TwoFAEnable>,
) -> Result<Json<TwoFAStatus>, (StatusCode, String)> {
    // Получаем ID пользователя из контекста (должен быть аутентифицирован)
    // В реальной реализации нужно извлекать из JWT токена
    let user_id = 1; // Заглушка, нужно извлекать из Claims

    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, email, avatar_color, bio, last_login, created_at, two_factor_enabled, two_factor_secret FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Пользователь не найден".to_string()))?;

    if payload.enable {
        // Проверяем TOTP код перед включением
        let secret = user.two_factor_secret
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "2FA секрет не установлен".to_string()))?;

        let secret_bytes = base32::decode(base32::Alphabet::Crockford, &secret)
            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка декодирования секрета".to_string()))?;

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("Trello Local".to_string()),
            user.username.clone(),
        ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка создания TOTP: {}", e)))?;

        let valid = totp.check_current(&payload.code)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка проверки TOTP: {}", e)))?;

        if !valid {
            tracing::warn!(
                target: "security",
                user_id = user_id,
                event = "2fa_enable_failed",
                "Неверный 2FA код при включении"
            );
            return Err((StatusCode::UNAUTHORIZED, "Неверный 2FA код".to_string()));
        }

        // Включаем 2FA
        sqlx::query("UPDATE users SET two_factor_enabled = 1, two_factor_secret = ? WHERE id = ?")
            .bind(&secret)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        tracing::info!(
            target: "security",
            user_id = user_id,
            event = "2fa_enabled",
            "2FA включен для пользователя"
        );

        Ok(Json(TwoFAStatus { enabled: true }))
    } else {
        // Выключаем 2FA
        sqlx::query("UPDATE users SET two_factor_enabled = 0, two_factor_secret = NULL WHERE id = ?")
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        tracing::info!(
            target: "security",
            user_id = user_id,
            event = "2fa_disabled",
            "2FA выключен для пользователя"
        );
        
        Ok(Json(TwoFAStatus { enabled: false }))
    }
}

/// Проверка 2FA кода после ввода пароля
pub async fn verify_2fa(
    State(pool): State<SqlitePool>,
    Json(payload): Json<TwoFACode>,
) -> Result<Json<AuthToken>, (StatusCode, String)> {
    // Извлекаем user_id из временного токена
    // В реальной реализации токен передается в заголовке

    // Заглушка для демонстрации
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, email, avatar_color, bio, last_login, created_at, two_factor_enabled, two_factor_secret FROM users WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Пользователь не найден".to_string()))?;

    let secret = user.two_factor_secret
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "2FA не настроен".to_string()))?;

    // Проверяем TOTP код
    let secret_bytes = base32::decode(base32::Alphabet::Crockford, &secret)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка декодирования секрета".to_string()))?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Trello Local".to_string()),
        user.username.clone(),
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка создания TOTP: {}", e)))?;

    let valid = totp.check_current(&payload.code)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ошибка проверки TOTP: {}", e)))?;

    if !valid {
        tracing::warn!(
            target: "security",
            user_id = user.id,
            event = "2fa_verification_failed",
            "Неверный 2FA код"
        );
        return Err((StatusCode::UNAUTHORIZED, "Неверный 2FA код".to_string()));
    }

    // Генерируем полный токен
    let token = generate_token(user.id, &user.username, true)?;

    // Сохраняем сессию
    let _ = sessions::save_session(&pool, user.id, &token, None, None).await;
    
    tracing::info!(
        target: "security",
        user_id = user.id,
        event = "2fa_verification_success",
        "Успешная проверка 2FA"
    );
    
    Ok(Json(AuthToken {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// Получение статуса 2FA для текущего пользователя
pub async fn get_2fa_status(
    State(pool): State<SqlitePool>,
) -> Result<Json<TwoFAStatus>, (StatusCode, String)> {
    // Заглушка - в реальности нужно извлекать user_id из токена
    let user_id = 1;

    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, email, avatar_color, bio, last_login, created_at, two_factor_enabled, two_factor_secret FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Пользователь не найден".to_string()))?;

    Ok(Json(TwoFAStatus {
        enabled: user.two_factor_enabled.unwrap_or(false),
    }))
}
