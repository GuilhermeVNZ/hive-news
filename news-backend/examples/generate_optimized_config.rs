/// Automatically update system_config.json with optimal collector settings
/// Based on intelligent_test_results.json from test_all_strategies
///
/// Run with: cargo run --example generate_optimized_config
///
/// This will:
/// 1. Read test results from intelligent_test_results.json
/// 2. Load current system_config.json
/// 3. Update each collector with optimal configuration:
///    - Switch HTML to RSS where available (30 sites!)
///    - Add force_js: true for Playwright-required sites (7 sites)
///    - Mark broken sites as disabled (6 sites)
/// 4. Backup original to system_config.json.backup
/// 5. Save optimized system_config.json

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
struct TestResult {
    name: String,
    original_url: String,
    original_type: String,
    original_id: String,
    rss_available: Option<RssFeedInfo>,
    html_works: bool,
    recommended_type: String,
    recommended_url: String,
    reason: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RssFeedInfo {
    url: String,
    works: bool,
}

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  GENERATING OPTIMIZED SYSTEM_CONFIG.JSON");
    println!("═══════════════════════════════════════════════════════════\n");

    // Load test results
    let results_json = fs::read_to_string("intelligent_test_results.json")
        .expect("Failed to read intelligent_test_results.json - run test_all_strategies first!");
    let test_results: Vec<TestResult> = serde_json::from_str(&results_json)
        .expect("Failed to parse test results");

    println!("✓ Loaded {} test results\n", test_results.len());

    // Load current system_config.json
    let config_path = "../news-backend/system_config.json";
    let config_json = fs::read_to_string(config_path)
        .expect("Failed to read system_config.json");
    let mut config: Value = serde_json::from_str(&config_json)
        .expect("Failed to parse system_config.json");

    // Backup original
    let backup_path = "../news-backend/system_config.json.backup";
    fs::write(backup_path, &config_json)
        .expect("Failed to create backup");
    println!("✓ Backup created: system_config.json.backup\n");

    // Build lookup map
    let mut lookup: HashMap<String, &TestResult> = HashMap::new();
    for result in &test_results {
        lookup.insert(result.original_id.clone(), result);
    }

    // Update collectors
    let mut stats = Stats::default();
    
    if let Some(sites) = config.get_mut("sites").and_then(|s| s.as_object_mut()) {
        for (_site_name, site_config) in sites.iter_mut() {
            if let Some(collectors) = site_config.get_mut("collectors").and_then(|c| c.as_array_mut()) {
                for collector in collectors.iter_mut() {
                    if let Some(collector_obj) = collector.as_object_mut() {
                        if let Some(id) = collector_obj.get("id").and_then(|i| i.as_str()) {
                            if let Some(test_result) = lookup.get(id) {
                                update_collector(collector_obj, test_result, &mut stats);
                            }
                        }
                    }
                }
            }
        }
    }

    // Save updated config
    let updated_json = serde_json::to_string_pretty(&config)
        .expect("Failed to serialize config");
    fs::write(config_path, updated_json)
        .expect("Failed to write system_config.json");

    println!("\n✓ system_config.json updated!\n");
    print_stats(&stats);
}

#[derive(Default)]
struct Stats {
    html_to_rss: usize,
    added_playwright: usize,
    disabled_broken: usize,
    unchanged: usize,
}

fn update_collector(collector: &mut serde_json::Map<String, Value>, test_result: &TestResult, stats: &mut Stats) {
    let id = test_result.original_id.as_str();
    
    match test_result.recommended_type.as_str() {
        "rss" if test_result.original_type == "html" => {
            // Found RSS for HTML site - convert to RSS!
            if let Some(rss_info) = &test_result.rss_available {
                println!("  🔄 {} -> RSS", id);
                println!("      Old URL: {}", test_result.original_url);
                println!("      New RSS: {}", rss_info.url);
                
                collector.insert("collector_type".to_string(), json!("rss"));
                collector.insert("feed_url".to_string(), json!(rss_info.url.clone()));
                
                // Keep base_url for reference, update if needed
                if let Ok(parsed) = url::Url::parse(&rss_info.url) {
                    if let Some(domain) = parsed.domain() {
                        let base = format!("{}://{}", parsed.scheme(), domain);
                        collector.insert("base_url".to_string(), json!(base));
                    }
                }
                
                // Remove HTML-specific fields
                collector.remove("selectors");
                
                // Ensure enabled
                collector.insert("enabled".to_string(), json!(true));
                
                stats.html_to_rss += 1;
            }
        }
        "playwright" => {
            // Needs Playwright
            println!("  ⚠️  {} -> Playwright required", id);
            println!("      Reason: {}", test_result.reason);
            
            // Add force_js config
            if let Some(config_obj) = collector.get_mut("config").and_then(|c| c.as_object_mut()) {
                config_obj.insert("force_js".to_string(), json!(true));
            } else {
                let mut new_config = serde_json::Map::new();
                new_config.insert("force_js".to_string(), json!(true));
                new_config.insert("max_results".to_string(), json!(5));
                collector.insert("config".to_string(), json!(new_config));
            }
            
            stats.added_playwright += 1;
        }
        "broken" => {
            // Disable broken collectors
            println!("  ❌ {} -> DISABLED (broken)", id);
            println!("      URL: {}", test_result.original_url);
            
            collector.insert("enabled".to_string(), json!(false));
            
            stats.disabled_broken += 1;
        }
        _ => {
            // Already optimal (RSS verified, or HTML working)
            stats.unchanged += 1;
        }
    }
}

fn print_stats(stats: &Stats) {
    println!("═══════════════════════════════════════════════════════════");
    println!("  OPTIMIZATION SUMMARY");
    println!("═══════════════════════════════════════════════════════════\n");
    
    println!("🔄 HTML → RSS conversions: {}", stats.html_to_rss);
    println!("⚠️  Playwright required: {}", stats.added_playwright);
    println!("❌ Disabled (broken): {}", stats.disabled_broken);
    println!("✓ Already optimal: {}", stats.unchanged);
    
    let total_changes = stats.html_to_rss + stats.added_playwright + stats.disabled_broken;
    println!("\n📊 Total changes: {}", total_changes);
    
    println!("\n✅ DONE! Your system_config.json is now optimized.");
    println!("   {} sites now using RSS (faster & more reliable)", stats.html_to_rss);
    println!("   {} sites configured for Playwright (for JS-heavy sites)", stats.added_playwright);
    println!("   {} broken sites disabled (won't waste time collecting)", stats.disabled_broken);
    
    println!("\n💡 Next: Rebuild Docker and test collection:");
    println!("   docker compose build backend");
    println!("   docker compose up backend");
    
    println!("\n═══════════════════════════════════════════════════════════\n");
}

