use axum::{extract::Extension, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use anyhow::Result;
use std::path::Path;

use crate::utils::config_manager::{ConfigManager, CollectorConfig};
use crate::utils::site_config_manager::SiteConfigManager;

#[derive(Debug, Deserialize)]
pub struct UpdateCollectorRequest {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// Get all collectors configuration with site assignments
pub async fn get_collectors(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    let config_path = Path::new("collectors_config.json");
    let config_manager = ConfigManager::new(config_path);
    
    let site_config_path = Path::new("system_config.json");
    let site_manager = crate::utils::site_config_manager::SiteConfigManager::new(site_config_path);

    match config_manager.load() {
        Ok(config) => {
            // Get all sites to show which sites each collector is assigned to
            let sites = site_manager.get_all_sites().unwrap_or_default();
            
            // Build a map: collector_id -> [site_ids where enabled]
            let mut collectors_with_sites: Vec<Value> = Vec::new();
            
            for collector in &config.collectors {
                let assigned_sites: Vec<Value> = sites
                    .iter()
                    .filter(|site| {
                        site.collectors.iter().any(|c| 
                            c.id == collector.id && c.enabled
                        )
                    })
                    .map(|site| json!({
                        "id": site.id,
                        "name": site.name,
                    }))
                    .collect();
                
                collectors_with_sites.push(json!({
                    "id": collector.id,
                    "name": collector.name,
                    "enabled": collector.enabled,
                    "api_key": collector.api_key,
                    "config": collector.config,
                    "assigned_sites": assigned_sites,
                }));
            }
            
            Json(json!({
                "success": true,
                "collectors": collectors_with_sites,
                "updated_at": config.updated_at,
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to load config: {}", e),
        })),
    }
}

/// Get enabled collectors only
pub async fn get_enabled_collectors(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    let config_path = Path::new("collectors_config.json");
    let config_manager = ConfigManager::new(config_path);

    match config_manager.get_enabled_collectors() {
        Ok(collectors) => Json(json!({
            "success": true,
            "collectors": collectors,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get enabled collectors: {}", e),
        })),
    }
}

/// Update collector status (enable/disable)
pub async fn update_collector_status(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(collector_id): axum::extract::Path<String>,
    Json(request): Json<UpdateCollectorRequest>,
) -> Json<Value> {
    let config_path = Path::new("collectors_config.json");
    let config_manager = ConfigManager::new(config_path);

    if let Some(enabled) = request.enabled {
        match config_manager.update_collector_status(&collector_id, enabled) {
            Ok(_) => Json(json!({
                "success": true,
                "message": format!("Collector {} {}", 
                    collector_id, 
                    if enabled { "enabled" } else { "disabled" }
                ),
            })),
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to update collector: {}", e),
            })),
        }
    } else {
        Json(json!({
            "success": false,
            "error": "enabled field is required",
        }))
    }
}

/// Update collector site assignments (which pages/sites use this collector)
#[derive(Debug, Deserialize)]
pub struct UpdateCollectorSitesRequest {
    pub site_ids: Vec<String>, // List of site IDs to assign this collector to
}

pub async fn update_collector_sites(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(collector_id): axum::extract::Path<String>,
    Json(request): Json<UpdateCollectorSitesRequest>,
) -> Json<Value> {
    let config_path = Path::new("system_config.json");
    let site_manager = SiteConfigManager::new(config_path);
    
    let all_sites = match site_manager.get_all_sites() {
        Ok(sites) => sites,
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load sites: {}", e),
            }));
        }
    };
    
    // Update each site's collectors configuration
    for site in all_sites {
        let mut updated_site = site.clone();
        
        // Find the collector in this site's collectors
        if let Some(collector) = updated_site.collectors.iter_mut().find(|c| c.id == collector_id) {
            // If collector_id is in request.site_ids, enable it; otherwise, disable it
            let should_be_enabled = request.site_ids.contains(&updated_site.id);
            collector.enabled = should_be_enabled;
        } else {
            // Collector doesn't exist in this site, check if we should add it
            if request.site_ids.contains(&updated_site.id) {
                // Add collector to this site (need to load from collectors_config.json)
                let collectors_path = Path::new("collectors_config.json");
                let collectors_manager = ConfigManager::new(collectors_path);
                if let Ok(collectors_config) = collectors_manager.load() {
                    if let Some(base_collector) = collectors_config.collectors.iter().find(|c| c.id == collector_id) {
                        let mut mapped = crate::utils::site_config_manager::CollectorConfig {
                            id: base_collector.id.clone(),
                            name: base_collector.name.clone(),
                            enabled: true,
                            api_key: base_collector.api_key.clone(),
                            config: base_collector.config.clone(),
                        };
                        updated_site.collectors.push(mapped);
                    }
                }
            }
        }
        
        // Save the updated site
        let site_id_clone = updated_site.id.clone();
        if let Err(e) = site_manager.update_site_config(&site_id_clone, updated_site) {
            return Json(json!({
                "success": false,
                "error": format!("Failed to update site {}: {}", site.id, e),
            }));
        }
    }
    
    Json(json!({
        "success": true,
        "message": format!("Collector {} assigned to {} sites", collector_id, request.site_ids.len()),
    }))
}

/// Update collector configuration
pub async fn update_collector_config(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(collector_id): axum::extract::Path<String>,
    Json(request): Json<UpdateCollectorRequest>,
) -> Json<Value> {
    let config_path = Path::new("collectors_config.json");
    let config_manager = ConfigManager::new(config_path);

    // Load current config to preserve other fields
    let config = match config_manager.load() {
        Ok(c) => c,
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load config: {}", e),
            }));
        }
    };

    // Find current collector to preserve fields
    let current_collector = config.collectors.iter()
        .find(|c| c.id == collector_id)
        .cloned();

    let mut updated_collector = if let Some(collector) = current_collector {
        collector
    } else {
        return Json(json!({
            "success": false,
            "error": format!("Collector not found: {}", collector_id),
        }));
    };

    // Apply updates
    if let Some(enabled) = request.enabled {
        updated_collector.enabled = enabled;
    }
    if let Some(api_key) = request.api_key {
        updated_collector.api_key = Some(api_key);
    }
    if let Some(new_config) = request.config {
        updated_collector.config = new_config;
    }

    match config_manager.update_collector_config(&collector_id, updated_collector) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Collector {} updated", collector_id),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update config: {}", e),
        })),
    }
}

