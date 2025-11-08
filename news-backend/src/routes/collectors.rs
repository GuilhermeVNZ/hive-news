use axum::{extract::Extension, response::Json};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::utils::config_manager::ConfigManager;
use crate::utils::path_resolver::resolve_workspace_path;
use crate::utils::site_config_manager::SiteConfigManager;

/// Get the system_config.json path (try multiple locations)
fn get_system_config_path() -> PathBuf {
    // Try multiple locations
    let possible_paths = vec![
        PathBuf::from("system_config.json"), // Current directory
        PathBuf::from("news-backend/system_config.json"), // From parent
        resolve_workspace_path("system_config.json"),
        resolve_workspace_path("news-backend/system_config.json"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Default to current directory if not found
    PathBuf::from("system_config.json")
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectorRequest {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// Get all collectors configuration with site assignments
/// Reads from system_config.json to get all collectors from all sites
pub async fn get_collectors(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    let site_config_path = get_system_config_path();
    let site_manager = crate::utils::site_config_manager::SiteConfigManager::new(&site_config_path);

    match site_manager.get_all_sites() {
        Ok(sites) => {
            // Build a consolidated map: collector_id -> collector_info with all sites that use it
            use std::collections::HashMap;
            let mut collectors_map: HashMap<String, Value> = HashMap::new();

            for site in &sites {
                for collector in &site.collectors {
                    // Create a unique key for the collector (includes collector_type if available)
                    let collector_key = collector.id.clone();

                    // Get or create collector entry
                    if let Some(existing) = collectors_map.get_mut(&collector_key) {
                        // Collector already exists - add this site to assigned_sites if enabled
                        if collector.enabled {
                            if let Some(assigned_sites) = existing
                                .get_mut("assigned_sites")
                                .and_then(|v| v.as_array_mut())
                            {
                                assigned_sites.push(json!({
                                    "id": site.id,
                                    "name": site.name,
                                }));
                            }
                            // Update enabled status if any site has it enabled
                            if let Some(enabled) = existing.get_mut("enabled") {
                                *enabled = json!(true);
                            }
                        }
                    } else {
                        // New collector - create entry
                        let mut assigned_sites = Vec::new();
                        if collector.enabled {
                            assigned_sites.push(json!({
                                "id": site.id,
                                "name": site.name,
                            }));
                        }

                        collectors_map.insert(
                            collector_key.clone(),
                            json!({
                                "id": collector.id,
                                "name": collector.name,
                                "enabled": collector.enabled,
                                "api_key": collector.api_key,
                                "collector_type": collector.collector_type,
                                "feed_url": collector.feed_url,
                                "base_url": collector.base_url,
                                "selectors": collector.selectors,
                                "destinations": collector.destinations.clone().unwrap_or_default(),
                                "config": collector.config,
                                "assigned_sites": assigned_sites,
                            }),
                        );
                    }
                }
            }

            // Convert HashMap to Vec sorted by collector name
            let mut collectors_with_sites: Vec<Value> = collectors_map.into_values().collect();
            collectors_with_sites.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });

            Json(json!({
                "success": true,
                "collectors": collectors_with_sites,
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
/// Updates the collector status in all sites in system_config.json
pub async fn update_collector_status(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(collector_id): axum::extract::Path<String>,
    Json(request): Json<UpdateCollectorRequest>,
) -> Json<Value> {
    let site_config_path = get_system_config_path();
    let site_manager = SiteConfigManager::new(&site_config_path);

    if let Some(enabled) = request.enabled {
        match site_manager.get_all_sites() {
            Ok(sites) => {
                let mut updated_count = 0;
                for site in sites {
                    // Try to update collector status in this site
                    if let Err(e) =
                        site_manager.update_collector_status(&site.id, &collector_id, enabled)
                    {
                        // If collector doesn't exist in this site, that's okay - skip it
                        if e.to_string().contains("not found") {
                            continue;
                        }
                        return Json(json!({
                            "success": false,
                            "error": format!("Failed to update collector in site {}: {}", site.id, e),
                        }));
                    }
                    updated_count += 1;
                }

                Json(json!({
                    "success": true,
                    "message": format!("Collector {} {} in {} sites",
                        collector_id,
                        if enabled { "enabled" } else { "disabled" },
                        updated_count
                    ),
                }))
            }
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to load sites: {}", e),
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
    let config_path = get_system_config_path();
    let site_manager = SiteConfigManager::new(&config_path);

    let all_sites = match site_manager.get_all_sites() {
        Ok(sites) => sites,
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to load sites: {}", e),
            }));
        }
    };

    // Update destinations for the collector in ALL sites where it exists
    // The destinations field indicates where content from this collector should be sent
    let mut updated_count = 0;

    for site in &all_sites {
        let mut updated_site = site.clone();
        let mut site_updated = false;

        // Find the collector in this site's collectors
        if let Some(collector) = updated_site
            .collectors
            .iter_mut()
            .find(|c| c.id == collector_id)
        {
            // Update destinations field to store which sites should receive content from this collector
            collector.destinations = Some(request.site_ids.clone());

            // Also update enabled status: enable if this site is in destinations, otherwise disable
            let should_be_enabled = request.site_ids.contains(&updated_site.id);
            collector.enabled = should_be_enabled;

            site_updated = true;
        }

        // If collector doesn't exist in this site but should be added
        if !site_updated && request.site_ids.contains(&updated_site.id) {
            // Find collector template in another site
            let collector_template = all_sites
                .iter()
                .flat_map(|s| &s.collectors)
                .find(|c| c.id == collector_id)
                .cloned();

            if let Some(mut base_collector) = collector_template {
                // Copy collector configuration from template
                base_collector.enabled = true;
                base_collector.destinations = Some(request.site_ids.clone());
                updated_site.collectors.push(base_collector);
                site_updated = true;
            } else {
                eprintln!(
                    "[WARNING] Collector {} not found in any site. Cannot add to site {}",
                    collector_id, updated_site.id
                );
            }
        }

        // Save the updated site if it was modified
        if site_updated {
            let site_id_clone = updated_site.id.clone();
            if let Err(e) = site_manager.update_site_config(&site_id_clone, updated_site) {
                return Json(json!({
                    "success": false,
                    "error": format!("Failed to update site {}: {}", site.id, e),
                }));
            }
            updated_count += 1;
        }
    }

    Json(json!({
        "success": true,
        "message": format!("Collector {} assigned to {} sites", collector_id, request.site_ids.len()),
        "updated_count": updated_count,
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
    let current_collector = config
        .collectors
        .iter()
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
