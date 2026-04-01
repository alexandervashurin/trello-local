use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub avatar_color: Option<String>,
    pub bio: Option<String>,
    pub last_login: Option<i64>,
    pub created_at: i64,
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
}

#[derive(Serialize, FromRow, Clone)]
pub struct UserWithPassword {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub avatar_color: Option<String>,
    pub bio: Option<String>,
    pub last_login: Option<i64>,
    pub created_at: i64,
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Deserialize)]
pub struct UpdateProfile {
    pub email: Option<String>,
    pub avatar_color: Option<String>,
    pub bio: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_activity: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct SessionInfo {
    pub id: i64,
    pub user_id: i64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_activity: i64,
    pub is_current: bool,
}

/// Данные для включения 2FA
#[derive(Serialize, Deserialize, Debug)]
pub struct TwoFASetup {
    pub secret: String,
    pub uri: String,
    pub qr_code: String,
}

/// Данные для проверки 2FA кода
#[derive(Deserialize, Debug)]
pub struct TwoFACode {
    pub code: String,
}

/// Данные для включения/выключения 2FA
#[derive(Deserialize, Debug)]
pub struct TwoFAEnable {
    pub code: String,
    pub enable: bool,
}

/// Response с информацией о 2FA статусе
#[derive(Serialize, Debug)]
pub struct TwoFAStatus {
    pub enabled: bool,
}

/// Response для временного токена после успешного ввода пароля
#[derive(Serialize, Debug)]
pub struct TwoFATempToken {
    pub temp_token: String,
    pub user_id: i64,
    pub username: String,
}
