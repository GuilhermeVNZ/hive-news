use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub instructor: String,
    pub institution: Option<String>,
    pub category: String,
    pub duration: String,
    pub level: String,
    pub price: String,
    pub rating: Option<f64>,
    pub students: Option<u64>,
    pub description: String,
    pub url: String,
    pub language: Option<String>,
    pub image_url: Option<String>,
    pub affiliate: bool,
    pub certificate_available: bool,
    pub free: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseCatalog {
    pub courses: Vec<Course>,
    pub platform: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
