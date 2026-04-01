// backend/src/models/oauth.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone, Debug)]
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct OAuthUrl {
    pub url: String,
    pub state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
}
