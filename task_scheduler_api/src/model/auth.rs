use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub merchant: Merchant,
    pub user: User,
}

impl AuthContext {
    pub fn new(merchant: Merchant, user: User) -> Self {
        Self { merchant, user }
    }
}
