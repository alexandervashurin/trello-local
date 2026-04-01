use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
}

/// Claims с 2FA флагом для временного токена
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClaimsWith2FA {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
    pub two_factor_verified: bool,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
}

#[derive(Serialize)]
pub struct TwoFATempTokenResponse {
    pub temp_token: String,
    pub user_id: i64,
    pub username: String,
}
