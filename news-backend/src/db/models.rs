use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Page {
    pub id: i32,
    pub name: String,
    pub sources: Vec<String>,
    pub frequency_minutes: i32,
    pub writing_style: String,
    pub linked_accounts: serde_json::Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageRequest {
    pub name: String,
    pub sources: Vec<String>,
    pub frequency_minutes: i32,
    pub writing_style: String,
    pub linked_accounts: serde_json::Value,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePageRequest {
    pub name: Option<String>,
    pub sources: Option<Vec<String>>,
    pub frequency_minutes: Option<i32>,
    pub writing_style: Option<String>,
    pub linked_accounts: Option<serde_json::Value>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
}

