use axum::{extract::Extension, response::Json};
use serde_json::{json, Value};

use crate::db::connection::Database;

pub async fn login(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement login logic
    Json(json!({ "message": "Login endpoint" }))
}

pub async fn logout(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement logout logic
    Json(json!({ "message": "Logout endpoint" }))
}

pub async fn get_me(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement get current user logic
    Json(json!({ "message": "Get current user endpoint" }))
}

