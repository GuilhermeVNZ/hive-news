// Sync collectors_config.json from system_config.json
// Usage: cargo run --bin sync-collectors-config

use std::path::Path;
use anyhow::{Context, Result};

// Import directly from the main crate's utils
// The utils are accessible via crate:: since all binaries share the same crate

fn main() -> Result<()> {
    println!("🔄 Syncing collectors_config.json from system_config.json...\n");

    // Try multiple paths for system_config.json
    let possible_system_paths = vec![
        Path::new("system_config.json"),
        Path::new("G:/Hive-Hub/News-main/news-backend/system_config.json"),
        Path::new("G:/Hive-Hub/News-main/system_config.json"),
    ];
    
    // Try multiple paths for collectors_config.json
    let possible_collectors_paths = vec![
        Path::new("collectors_config.json"),
        Path::new("G:/Hive-Hub/News-main/news-backend/collectors_config.json"),
        Path::new("G:/Hive-Hub/News-main/collectors_config.json"),
    ];
    
    // Find system_config.json
    let system_config_path = possible_system_paths.iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("system_config.json not found in any of the expected locations"))?;
    
    println!("✅ Found system_config.json: {}", system_config_path.display());
    
    // Find or create collectors_config.json path (use first found or default)
    let collectors_config_path = possible_collectors_paths.iter()
        .find(|p| p.exists())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new("collectors_config.json").to_path_buf());
    
    println!("📝 Target collectors_config.json: {}", collectors_config_path.display());
    println!();
    
    // We need to access ConfigManager from the crate
    // Since binaries can't directly access main.rs modules, we'll duplicate the sync logic here
    // OR we need to move the sync function to a library
    
    // For now, let's use a simpler approach: call the sync function directly
    // by including the necessary code or using a shared library structure
    
    // Actually, let's just use the function directly - if the binary is in the same crate,
    // it should have access to all public items
    
    // Check if we can import from crate::utils
    // If not, we'll need to duplicate the sync logic or create a lib.rs
    
    // Since we can't directly use crate::utils from a binary without lib.rs,
    // let's call the function via a helper that we'll create
    
    // Actually, the simplest solution: duplicate the sync logic here
    sync_collectors_config(system_config_path, &collectors_config_path)
        .context("Failed to sync collectors_config.json")?;
    
    println!();
    println!("✅ Sync completed successfully!");
    
    Ok(())
}

fn sync_collectors_config(system_config_path: &Path, collectors_config_path: &Path) -> Result<()> {
    use std::collections::HashMap;
    use std::fs;
    use serde_json::Value;
    
    // Load system_config.json
    let system_content = fs::read_to_string(system_config_path)
        .context(format!("Failed to read system_config.json: {}", system_config_path.display()))?;
    
    let system_config: Value = serde_json::from_str(&system_content)
        .context("Failed to parse system_config.json")?;
    
    let sites = system_config.get("sites")
        .ok_or_else(|| anyhow::anyhow!("'sites' not found in system_config.json"))?
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("'sites' is not an object"))?;
    
    println!("🔄 [SYNC] Loaded system_config.json with {} sites", sites.len());
    
    // Extract all enabled collectors from all enabled sites
    let mut collectors_map: HashMap<String, Value> = HashMap::new();
    
    for (site_id, site_value) in sites {
        let site = site_value.as_object()
            .ok_or_else(|| anyhow::anyhow!("Site '{}' is not an object", site_id))?;
        
        let enabled = site.get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if enabled {
            let site_collectors = site.get("collectors")
                .and_then(|v| v.as_array());
            
            let collectors_len = site_collectors.map(|arr| arr.len()).unwrap_or(0);
            println!("🔄 [SYNC] Processing site: {} ({} collectors)", site_id, collectors_len);
            
            if let Some(collectors_array) = site_collectors {
                for collector_value in collectors_array {
                if let Some(collector) = collector_value.as_object() {
                    if let Some(collector_id) = collector.get("id").and_then(|v| v.as_str()) {
                        // Deduplicate by ID
                        if !collectors_map.contains_key(collector_id) {
                            collectors_map.insert(collector_id.to_string(), collector_value.clone());
                        } else {
                            // If already exists, but new one is enabled and old one is not, update status
                            if let Some(existing) = collectors_map.get_mut(collector_id) {
                                let new_enabled = collector.get("enabled")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                
                                let existing_enabled = existing.get("enabled")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                
                                if new_enabled && !existing_enabled {
                                    if let Some(obj) = existing.as_object_mut() {
                                        obj.insert("enabled".to_string(), Value::Bool(true));
                                        println!("🔄 [SYNC] Updated collector '{}' status to enabled", collector_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }
        } else {
            println!("🔄 [SYNC] Skipping disabled site: {}", site_id);
        }
    }
    
    // Convert HashMap to Vec
    let all_collectors: Vec<Value> = collectors_map.into_values().collect();
    
    let enabled_count = all_collectors.iter()
        .filter(|c| c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();
    
    println!("🔄 [SYNC] Extracted {} collectors (deduplicated)", all_collectors.len());
    println!("🔄 [SYNC] Enabled collectors: {}", enabled_count);
    
    // Create CollectorsConfig JSON
    let collectors_config = serde_json::json!({
        "collectors": all_collectors,
        "updated_at": chrono::Utc::now().to_rfc3339()
    });
    
    // Save to collectors_config.json
    let collectors_content = serde_json::to_string_pretty(&collectors_config)
        .context("Failed to serialize collectors_config")?;
    
    fs::write(collectors_config_path, collectors_content)
        .context(format!("Failed to write collectors_config.json: {}", collectors_config_path.display()))?;
    
    println!("✅ [SYNC] Successfully synced collectors_config.json from system_config.json");
    
    Ok(())
}
