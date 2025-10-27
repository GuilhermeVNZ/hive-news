use axum::{
    extract::{Extension, Path},
    response::Json,
};
use serde_json::Value;

use crate::db::connection::Database;

pub async fn list_pages(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement list pages logic
    Json(serde_json::json!({ "message": "List pages endpoint" }))
}

pub async fn create_page(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implement create page logic
    Json(serde_json::json!({ "message": "Create page endpoint" }))
}

pub async fn get_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<i32>,
) -> Json<Value> {
    // TODO: Implement get page logic
    Json(serde_json::json!({ "message": format!("Get page {} endpoint", id) }))
}

pub async fn update_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<i32>,
) -> Json<Value> {
    // TODO: Implement update page logic
    Json(serde_json::json!({ "message": format!("Update page {} endpoint", id) }))
}

pub async fn delete_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<i32>,
) -> Json<Value> {
    // TODO: Implement delete page logic
    Json(serde_json::json!({ "message": format!("Delete page {} endpoint", id) }))
}
