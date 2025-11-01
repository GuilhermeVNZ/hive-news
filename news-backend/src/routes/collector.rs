use crate::db::connection::Database;
use axum::{
    extract::{Extension, Path},
    response::Json,
};
use serde_json::Value;

/// Inicia coleta para um portal específico
pub async fn start_collection(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    // TODO: Implementar lógica de coleta
    Json(serde_json::json!({
        "message": "Collection started",
        "success": true
    }))
}

/// Obtém status da última coleta
pub async fn get_collection_status(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(portal_id): Path<i32>,
) -> Json<Value> {
    // TODO: Implementar busca de status
    Json(serde_json::json!({
        "portal_id": portal_id,
        "status": "success",
        "last_collection": "2025-10-27T14:30:00Z"
    }))
}

/// Lista logs de coletas
pub async fn list_collection_logs(
    Extension(_db): Extension<std::sync::Arc<Database>>,
) -> Json<Value> {
    // TODO: Implementar listagem de logs
    Json(serde_json::json!({
        "logs": []
    }))
}
