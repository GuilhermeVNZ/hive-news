use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RawDocument {
    pub id: i32,
    pub portal_id: i32,
    pub title: String,
    pub source_url: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: Option<i64>,
    pub metadata: serde_json::Value,
    pub downloaded_at: DateTime<Utc>,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRawDocument {
    pub portal_id: i32,
    pub title: String,
    pub source_url: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    pub id: String,
    pub title: String,
    pub url: String,
    pub published_date: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResult {
    pub success: bool,
    pub documents_collected: i32,
    pub duration_ms: i64,
    pub errors: Vec<String>,
}
