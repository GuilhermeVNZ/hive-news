use axum::{extract::Extension, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::path::Path;

use crate::utils::site_config_manager::{SiteConfigManager, EducationSourceConfig};
use crate::utils::env_sync;

/// Get all sites configuration
pub async fn get_all_sites(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    match config_manager.get_all_sites() {
        Ok(sites) => {
            println!("📋 get_all_sites: Found {} sites", sites.len());
            for site in &sites {
                println!("   - {}: {} (enabled: {})", site.id, site.name, site.enabled);
            }
            Json(json!({
                "success": true,
                "sites": sites,
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to load sites: {}", e);
            Json(json!({
                "success": false,
                "error": format!("Failed to load config: {}", e),
            }))
        }
    }
}

/// Get configuration for a specific site
pub async fn get_site_config(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    match config_manager.get_site_config(&site_id) {
        Ok(Some(site)) => Json(json!({
            "success": true,
            "site": site,
        })),
        Ok(None) => Json(json!({
            "success": false,
            "error": format!("Site not found: {}", site_id),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to load config: {}", e),
        })),
    }
}

/// Update writer configuration for a site
#[derive(Debug, Deserialize)]
pub struct UpdateWriterRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub enabled: Option<bool>,
    pub use_compressor: Option<bool>,
    pub writing_style: Option<String>,
    // Prompt templates per channel
    pub prompt_article: Option<String>,
    pub prompt_social: Option<String>,
    pub prompt_blog: Option<String>,
    // Enable flags per channel
    pub prompt_article_enabled: Option<bool>,
    pub prompt_social_enabled: Option<bool>,
    pub prompt_blog_enabled: Option<bool>,
}

pub async fn update_writer_config(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(request): Json<UpdateWriterRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    // Load current config
    let mut current_site = match config_manager.get_site_config(&site_id) {
        Ok(Some(site)) => site,
        Ok(None) => {
            return Json(json!({
                "success": false,
                "error": format!("Site not found: {}", site_id),
            }));
        }
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load config: {}", e),
            }));
        }
    };

    let mut writer = current_site.writer;
    
    if let Some(provider) = request.provider {
        writer.provider = provider.clone();
        // Auto-update model based on provider if model not explicitly provided
        if request.model.is_none() {
            writer.model = match provider.as_str() {
                "deepseek" => "deepseek-chat".to_string(),
                "openai" => "gpt-4".to_string(),
                "anthropic" => "claude-3-opus".to_string(),
                _ => writer.model.clone(), // Keep existing model if provider unknown
            };
        }
    }
    if let Some(model) = request.model {
        writer.model = model;
    }
    if let Some(api_key) = request.api_key {
        writer.api_key = Some(api_key);
    }
    if let Some(base_url) = request.base_url {
        writer.base_url = Some(base_url);
    }
    if let Some(temperature) = request.temperature {
        writer.temperature = Some(temperature);
    }
    if let Some(max_tokens) = request.max_tokens {
        writer.max_tokens = Some(max_tokens);
    }
    if let Some(enabled) = request.enabled {
        writer.enabled = enabled;
    }
    if let Some(use_comp) = request.use_compressor {
        writer.use_compressor = Some(use_comp);
    }

    // Update prompt style at site level if provided
    if let Some(style) = request.writing_style {
        current_site.writing_style = Some(style);
    }
    if let Some(p) = request.prompt_article { current_site.prompt_article = Some(p); }
    if let Some(p) = request.prompt_social { current_site.prompt_social = Some(p); }
    if let Some(p) = request.prompt_blog { current_site.prompt_blog = Some(p); }
    if let Some(e) = request.prompt_article_enabled { current_site.prompt_article_enabled = Some(e); }
    if let Some(e) = request.prompt_social_enabled { current_site.prompt_social_enabled = Some(e); }
    if let Some(e) = request.prompt_blog_enabled { current_site.prompt_blog_enabled = Some(e); }

    // Save both writer and possibly updated site fields
    current_site.writer = writer.clone();
    
    match config_manager.update_site_config(&site_id, current_site) {
        Ok(_) => {
            // Sync .env file automatically after successfully updating writer config
            println!("🔄 Syncing .env file after writer config update...");
            if let Err(e) = env_sync::sync_env_from_config(Path::new("system_config.json")) {
                eprintln!("⚠️  Failed to sync .env file (non-critical): {}", e);
                // Don't fail the request if .env sync fails
            }
            
            Json(json!({
                "success": true,
                "message": format!("Writer config updated for site {}", site_id),
            }))
        },
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update config: {}", e),
        })),
    }
}

/// Update collector status for a site
#[derive(Debug, Deserialize)]
pub struct UpdateCollectorStatusRequest {
    pub enabled: bool,
}

pub async fn update_collector_status(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path((site_id, collector_id)): axum::extract::Path<(String, String)>,
    Json(request): Json<UpdateCollectorStatusRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    match config_manager.update_collector_status(&site_id, &collector_id, request.enabled) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Collector {} {} for site {}", 
                collector_id, 
                if request.enabled { "enabled" } else { "disabled" },
                site_id
            ),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update collector: {}", e),
        })),
    }
}

/// Update social media status for a site
#[derive(Debug, Deserialize)]
pub struct UpdateSocialStatusRequest {
    pub enabled: bool,
}

pub async fn update_social_status(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path((site_id, social_id)): axum::extract::Path<(String, String)>,
    Json(request): Json<UpdateSocialStatusRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    match config_manager.update_social_status(&site_id, &social_id, request.enabled) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Social media {} {} for site {}", 
                social_id, 
                if request.enabled { "enabled" } else { "disabled" },
                site_id
            ),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update social media: {}", e),
        })),
    }
}

/// Update social media configuration
#[derive(Debug, Deserialize)]
pub struct UpdateSocialConfigRequest {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub channel_id: Option<String>,
    pub username: Option<String>,
    pub config: Option<serde_json::Value>,
}

pub async fn update_social_config(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path((site_id, social_id)): axum::extract::Path<(String, String)>,
    Json(request): Json<UpdateSocialConfigRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let config_manager = SiteConfigManager::new(config_path);

    // Load current site config
    let mut site = match config_manager.get_site_config(&site_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Json(json!({
                "success": false,
                "error": format!("Site not found: {}", site_id),
            }));
        }
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load config: {}", e),
            }));
        }
    };

    // Find and update social media config
    if let Some(social) = site.social_media.iter_mut().find(|s| s.id == social_id) {
        if let Some(enabled) = request.enabled {
            social.enabled = enabled;
        }
        if let Some(api_key) = request.api_key {
            social.api_key = Some(api_key);
        }
        if let Some(api_secret) = request.api_secret {
            social.api_secret = Some(api_secret);
        }
        if let Some(access_token) = request.access_token {
            social.access_token = Some(access_token);
        }
        if let Some(refresh_token) = request.refresh_token {
            social.refresh_token = Some(refresh_token);
        }
        if let Some(channel_id) = request.channel_id {
            social.channel_id = Some(channel_id);
        }
        if let Some(username) = request.username {
            social.username = Some(username);
        }
        if let Some(config) = request.config {
            social.config = config;
        }
    } else {
        return Json(json!({
            "success": false,
            "error": format!("Social media not found: {}", social_id),
        }));
    }

    match config_manager.update_site_config(&site_id, site) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Social media config updated for site {}", site_id),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to save config: {}", e),
        })),
    }
}

// === Education endpoints ===
#[derive(Debug, Deserialize)]
pub struct UpdateEducationStatusRequest { pub enabled: bool }

#[derive(Debug, Deserialize)]
pub struct UpdateEducationConfigRequest { pub enabled: Option<bool>, pub api_key: Option<String>, pub config: Option<serde_json::Value> }

pub async fn update_education_status(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path((site_id, source_id)): axum::extract::Path<(String, String)>,
    Json(req): Json<UpdateEducationStatusRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let manager = SiteConfigManager::new(config_path);

    let mut site = match manager.get_site_config(&site_id) {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!({"success": false, "error": format!("Site not found: {}", site_id)})),
        Err(e) => return Json(json!({"success": false, "error": format!("Failed to load config: {}", e)})),
    };

    if let Some(src) = site.education_sources.iter_mut().find(|s| s.id == source_id) {
        src.enabled = req.enabled;
    } else {
        // If not present, add a default entry
        site.education_sources.push(EducationSourceConfig { id: source_id.clone(), name: source_id.clone(), enabled: req.enabled, api_key: None, config: serde_json::json!({}) });
    }

    match manager.update_site_config(&site_id, site) {
        Ok(_) => Json(json!({"success": true, "message": format!("Education {} {} for site {}", source_id, if req.enabled {"enabled"} else {"disabled"}, site_id)})),
        Err(e) => Json(json!({"success": false, "error": format!("Failed to update education: {}", e)})),
    }
}

pub async fn update_education_config(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path((site_id, source_id)): axum::extract::Path<(String, String)>,
    Json(req): Json<UpdateEducationConfigRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let manager = SiteConfigManager::new(config_path);

    let mut site = match manager.get_site_config(&site_id) {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!({"success": false, "error": format!("Site not found: {}", site_id)})),
        Err(e) => return Json(json!({"success": false, "error": format!("Failed to load config: {}", e)})),
    };

    if let Some(src) = site.education_sources.iter_mut().find(|s| s.id == source_id) {
        if let Some(enabled) = req.enabled { src.enabled = enabled; }
        if let Some(api_key) = req.api_key { src.api_key = Some(api_key); }
        if let Some(cfg) = req.config { src.config = cfg; }
    } else {
        site.education_sources.push(EducationSourceConfig {
            id: source_id.clone(), name: source_id.clone(), enabled: req.enabled.unwrap_or(false), api_key: req.api_key, config: req.config.unwrap_or(serde_json::json!({}))
        });
    }

    match manager.update_site_config(&site_id, site) {
        Ok(_) => Json(json!({"success": true, "message": format!("Education config updated for site {}", site_id)})),
        Err(e) => Json(json!({"success": false, "error": format!("Failed to save education config: {}", e)})),
    }
}

// === Quick action: start collect (stub) ===
#[derive(Debug, Serialize)]
pub struct CollectStartResponse { pub success: bool, pub message: String }

pub async fn start_collect_for_site(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Json<CollectStartResponse> {
    // TODO: trigger real collector service; for now return success stub
    Json(CollectStartResponse{ success: true, message: format!("Collect triggered for {}", site_id) })
}










