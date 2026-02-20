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
