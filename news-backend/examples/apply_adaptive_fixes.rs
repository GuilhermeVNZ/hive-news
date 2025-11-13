/// Apply Adaptive Test Fixes to system_config.json
///
/// Automatically updates system_config.json with:
/// - Corrected URLs from adaptive test
/// - force_js: true for Playwright-needed sites
/// - Anti-bot headers config where needed
///
/// Run with: cargo run --example apply_adaptive_fixes --release

use serde_json::{json, Value};
use std::fs;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct AdaptiveTestResult {
    name: String,
    original_url: String,
    success: bool,
    working_url: String,
    working_strategy: String,
    needs_playwright: bool,
}

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  APPLYING ADAPTIVE TEST FIXES");
    println!("═══════════════════════════════════════════════════════════\n");

    // Load adaptive test results
    let results_content = fs::read_to_string("adaptive_test_results.json")
        .expect("Failed to read adaptive_test_results.json - run test_adaptive_access first!");
    let results: Vec<AdaptiveTestResult> = serde_json::from_str(&results_content)
        .expect("Failed to parse adaptive_test_results.json");

    // Load system_config.json
    let config_content = fs::read_to_string("system_config.json")
        .expect("Failed to read system_config.json");
    let mut config: Value = serde_json::from_str(&config_content)
        .expect("Failed to parse system_config.json");

    // Backup original
    fs::write("system_config.json.backup_adaptive", &config_content)
        .expect("Failed to create backup");
    println!("✓ Backup created: system_config.json.backup_adaptive\n");

    // Create lookup map for quick access
    let mut result_map = HashMap::new();
    for result in &results {
        result_map.insert(result.name.clone(), result);
    }

    let mut url_updates = 0;
    let mut playwright_adds = 0;
    let mut header_adds = 0;

    // Iterate through config and apply fixes
    if let Some(sites) = config["sites"].as_object_mut() {
        for (_site_key, site_config) in sites.iter_mut() {
            if let Some(collectors) = site_config["collectors"].as_array_mut() {
                for collector in collectors.iter_mut() {
                    // Clone name to avoid borrow issues
                    let name = collector["name"].as_str().map(|s| s.to_string());
                    
                    if let Some(name) = name {
                        if let Some(result) = result_map.get(&name) {
                            // Apply fixes based on test results
                            
                            // 1. Update URL if different
                            if result.success && result.working_url != result.original_url {
                                let collector_type = collector["collector_type"].as_str()
                                    .or_else(|| collector["type"].as_str())
                                    .unwrap_or("unknown");
                                
                                if collector_type == "rss" {
                                    collector["feed_url"] = json!(result.working_url.clone());
                                } else {
                                    collector["url"] = json!(result.working_url.clone());
                                }
                                
                                println!("✓ Updated URL for {}", name);
                                println!("  {} -> {}", result.original_url, result.working_url);
                                url_updates += 1;
                            }
                            
                            // 2. Add force_js for Playwright-needed sites
                            if result.needs_playwright {
                                if let Some(config_obj) = collector["config"].as_object_mut() {
                                    config_obj.insert("force_js".to_string(), json!(true));
                                } else {
                                    // Create config object if it doesn't exist
                                    let mut new_config = serde_json::Map::new();
                                    new_config.insert("force_js".to_string(), json!(true));
                                    collector["config"] = json!(new_config);
                                }
                                
                                println!("✓ Added force_js: true for {}", name);
                                playwright_adds += 1;
                            }
                            
                            // 3. Add hint for sites that need anti-bot headers
                            if result.success && result.working_strategy == "antibot_headers" {
                                if let Some(config_obj) = collector["config"].as_object_mut() {
                                    config_obj.insert("needs_antibot_headers".to_string(), json!(true));
                                } else if let Some(config_obj) = collector.as_object_mut() {
                                    let mut new_config = serde_json::Map::new();
                                    new_config.insert("needs_antibot_headers".to_string(), json!(true));
                                    config_obj.insert("config".to_string(), json!(new_config));
                                }
                                
                                header_adds += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Save updated config
    let updated_json = serde_json::to_string_pretty(&config)
        .expect("Failed to serialize updated config");
    fs::write("system_config.json", updated_json)
        .expect("Failed to write updated system_config.json");

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  OPTIMIZATION SUMMARY");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("📝 Changes Applied:");
    println!("   • URL updates:              {}", url_updates);
    println!("   • force_js added:           {}", playwright_adds);
    println!("   • Anti-bot header hints:    {}", header_adds);

    println!("\n✅ system_config.json updated successfully!");
    println!("💾 Backup saved as: system_config.json.backup_adaptive");
    
    println!("\n📋 Next Steps:");
    println!("   1. Review changes: git diff system_config.json");
    println!("   2. Rebuild Docker: docker compose build backend");
    println!("   3. Test collection: docker compose up backend");
    
    println!("\n═══════════════════════════════════════════════════════════\n");
}

