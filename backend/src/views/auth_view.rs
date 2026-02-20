use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
}
