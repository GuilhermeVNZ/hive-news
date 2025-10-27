use axum::extract::Extension;
use axum::response::Json;
use serde_json::Value;

use crate::db::connection::Database;

pub async fn list_sources(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement list sources logic
    Json(serde_json::json!({ "message": "List sources endpoint" }))
}

pub async fn create_source(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement create source logic
    Json(serde_json::json!({ "message": "Create source endpoint" }))
}

