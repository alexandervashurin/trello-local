use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: i64,
}

#[derive(Serialize, FromRow, Clone)]
pub struct UserWithPassword {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
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
