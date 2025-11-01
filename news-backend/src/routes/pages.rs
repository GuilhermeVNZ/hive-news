use axum::{
    extract::{Extension, Path},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path as FsPath;

use crate::db::connection::Database;
use crate::utils::site_config_manager::SiteConfigManager;
use crate::utils::config_manager::ConfigManager;

#[derive(Debug, Serialize)]
struct PageInfo {
    id: String,
    name: String,
    sources: Vec<String>,
    frequency_minutes: u32,
    writing_style: String,
    active: bool,
    domain: Option<String>,
    is_online: bool, // Whether the domain is currently online/active
}

pub async fn list_pages(Extension(_db): Extension<std::sync::Arc<Database>>) -> Json<Value> {
    let config_path = FsPath::new("system_config.json");
    let site_manager = SiteConfigManager::new(config_path);
    
    let collectors_path = FsPath::new("collectors_config.json");
    let collectors_manager = ConfigManager::new(collectors_path);
    
    match site_manager.get_all_sites() {
        Ok(sites) => {
            // Get enabled collectors to show active sources
            let enabled_collectors = collectors_manager.get_enabled_collectors().unwrap_or_default();
            let enabled_collector_ids: std::collections::HashSet<String> = enabled_collectors
                .iter()
                .map(|c| c.id.clone())
                .collect();
            
            let pages: Vec<PageInfo> = sites
                .into_iter()
                .map(|site| {
                    // Get enabled collectors for this site
                    let site_sources: Vec<String> = site.collectors
                        .iter()
                        .filter(|c| c.enabled && enabled_collector_ids.contains(&c.id))
                        .map(|c| c.name.clone())
                        .collect();
                    
                    // Check if domain is online (for now, consider offline if no domain or domain not configured)
                    let is_online = site.domain.as_ref()
                        .map(|d| !d.is_empty())
                        .unwrap_or(false); // Consider offline if no domain set
                    
                    PageInfo {
                        id: site.id.clone(),
                        name: site.name.clone(),
                        sources: site_sources,
                        frequency_minutes: site.collection_frequency_minutes.unwrap_or(60),
                        writing_style: site.writing_style.clone().unwrap_or_else(|| "scientific".to_string()),
                        active: site.enabled,
                        domain: site.domain.clone(),
                        is_online,
                    }
                })
                .collect();
            
            Json(json!({
                "success": true,
                "pages": pages,
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to load pages: {}", e),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub frequency_minutes: Option<u32>,
    pub writing_style: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Json(request): Json<CreatePageRequest>,
) -> Json<Value> {
    let config_path = FsPath::new("system_config.json");
    let site_manager = SiteConfigManager::new(config_path);
    
    // Validate required fields
    if request.id.trim().is_empty() || request.id.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Page ID is required",
        }));
    }
    
    if request.name.trim().is_empty() || request.name.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Page name is required",
        }));
    }
    
    // Upsert behavior: if site already exists, update its basic fields instead of erroring
    if let Ok(Some(mut existing_site)) = site_manager.get_site_config(&request.id) {
        existing_site.name = request.name.clone();
        existing_site.domain = request.domain.clone();
        if let Some(freq) = request.frequency_minutes { existing_site.collection_frequency_minutes = Some(freq); }
        if let Some(style) = request.writing_style.clone() { existing_site.writing_style = Some(style); }
        if let Some(enabled) = request.enabled { existing_site.enabled = enabled; }

        return match site_manager.update_site_config(&request.id, existing_site) {
            Ok(_) => Json(json!({
                "success": true,
                "message": format!("Page '{}' updated", request.name),
            })),
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to update page: {}", e),
            })),
        };
    }
    
    // Load collectors config to add them to the new site
    let collectors_path = FsPath::new("collectors_config.json");
    let collectors_manager = ConfigManager::new(collectors_path);
    
    // Create default collectors from global config (all disabled initially)
    let collectors: Vec<crate::utils::site_config_manager::CollectorConfig> = match collectors_manager.load() {
        Ok(collectors_config) => {
            collectors_config.collectors
                .into_iter()
                .map(|collector| {
                    crate::utils::site_config_manager::CollectorConfig {
                        id: collector.id,
                        name: collector.name,
                        enabled: false, // All collectors disabled by default
                        api_key: collector.api_key,
                        collector_type: collector.collector_type,
                        feed_url: collector.feed_url,
                        base_url: collector.base_url,
                        selectors: collector.selectors,
                        config: collector.config,
                    }
                })
                .collect()
        }
        Err(_) => {
            // If collectors config doesn't exist or fails to load, create empty list
            vec![]
        }
    };
    
    // Create new site config
    let new_site = crate::utils::site_config_manager::SiteConfig {
        id: request.id.clone(),
        name: request.name.clone(),
        domain: request.domain,
        enabled: request.enabled.unwrap_or(true),
        collectors,
        writer: crate::utils::site_config_manager::WriterConfig {
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: None,
            base_url: Some("https://api.deepseek.com".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4000),
            enabled: true,
            use_compressor: Some(false),
        },
        education_sources: vec![],
        social_media: vec![
            crate::utils::site_config_manager::SocialMediaConfig {
                id: "youtube".to_string(),
                name: "YouTube".to_string(),
                enabled: false,
                api_key: None,
                api_secret: None,
                access_token: None,
                refresh_token: None,
                channel_id: None,
                username: None,
                config: serde_json::json!({}),
            },
            crate::utils::site_config_manager::SocialMediaConfig {
                id: "x".to_string(),
                name: "X (Twitter)".to_string(),
                enabled: false,
                api_key: None,
                api_secret: None,
                access_token: None,
                refresh_token: None,
                channel_id: None,
                username: None,
                config: serde_json::json!({}),
            },
        ],
        collection_frequency_minutes: request.frequency_minutes.or(Some(60)),
        writing_style: request.writing_style.or(Some("scientific".to_string())),
        prompt_article: Some("You are an expert writer. Generate a concise, factual summary.".to_string()),
        prompt_social: Some("Create a short, engaging social post with hashtags.".to_string()),
        prompt_blog: Some("Write a blog-style article with sections and professional tone.".to_string()),
        prompt_article_enabled: Some(true),
        prompt_social_enabled: Some(false),
        prompt_blog_enabled: Some(false),
    };
    
    match site_manager.update_site_config(&request.id, new_site) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Page '{}' created successfully", request.name),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to create page: {}", e),
        })),
    }
}

pub async fn get_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<i32>,
) -> Json<Value> {
    // TODO: Implement get page logic
    Json(serde_json::json!({ "message": format!("Get page {} endpoint", id) }))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub frequency_minutes: Option<u32>,
    pub writing_style: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn update_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(request): Json<UpdatePageRequest>,
) -> Json<Value> {
    let config_path = FsPath::new("system_config.json");
    let site_manager = SiteConfigManager::new(config_path);
    
    let mut site = match site_manager.get_site_config(&id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Json(json!({
                "success": false,
                "error": format!("Page not found: {}", id),
            }));
        }
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load page: {}", e),
            }));
        }
    };
    
    // Update fields
    if let Some(frequency) = request.frequency_minutes {
        site.collection_frequency_minutes = Some(frequency);
    }
    if let Some(style) = request.writing_style {
        site.writing_style = Some(style);
    }
    if let Some(enabled) = request.enabled {
        site.enabled = enabled;
    }
    
    match site_manager.update_site_config(&id, site) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Page {} updated", id),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update page: {}", e),
        })),
    }
}

pub async fn delete_page(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<i32>,
) -> Json<Value> {
    // TODO: Implement delete page logic
    Json(serde_json::json!({ "message": format!("Delete page {} endpoint", id) }))
}
