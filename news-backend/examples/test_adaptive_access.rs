/// ADAPTIVE ACCESS TEST - Intelligent Multi-Strategy Article Collection
///
/// This test tries MULTIPLE strategies to access each site until one works:
/// 1. Direct HTTP (simple GET)
/// 2. Direct HTTP + anti-bot headers
/// 3. Try alternative RSS feeds (autodiscover)
/// 4. Try with cookie acceptance automation
/// 5. Try different URL patterns (blog/, news/, feed/)
/// 6. Mark as needing Playwright if all else fails
///
/// Run with: cargo run --example test_adaptive_access --release

use reqwest::{Client, header};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdaptiveTestResult {
    name: String,
    original_url: String,
    original_type: String,
    
    // Test results
    direct_http_works: bool,
    antibot_headers_works: bool,
    rss_autodiscover_works: bool,
    rss_autodiscover_url: Option<String>,
    alternative_url_works: bool,
    alternative_url: Option<String>,
    
    // Final result
    success: bool,
    working_strategy: String,
    working_url: String,
    needs_playwright: bool,
    blockers_found: Vec<String>,
    recommendation: String,
}

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  ADAPTIVE ACCESS TEST - Smart Strategy Testing");
    println!("  Trying multiple strategies until one works!");
    println!("═══════════════════════════════════════════════════════════\n");

    // Load sites from system_config.json
    println!("📋 Loading sites from system_config.json...\n");
    
    let config_content = fs::read_to_string("system_config.json")
        .expect("Failed to read system_config.json");
    let config: Value = serde_json::from_str(&config_content)
        .expect("Failed to parse system_config.json");
    
    let sites = config["sites"].as_object()
        .expect("No 'sites' key in config");
    
    let mut test_sites = Vec::new();
    
    for (site_key, site_config) in sites {
        if let Some(collectors) = site_config["collectors"].as_array() {
            for collector in collectors {
                let name = collector["name"].as_str().unwrap_or(site_key);
                let id = collector["id"].as_str().unwrap_or(site_key);
                let enabled = collector["enabled"].as_bool().unwrap_or(true);
                
                if !enabled {
                    continue;
                }
                
                // Skip Hive Hub and AIResearch (not online yet)
                if name.contains("Hive Hub") || name.contains("AIResearch News") {
                    println!("⏭️  Skipping {} (not online yet)", name);
                    continue;
                }
                
                let collector_type = collector["collector_type"].as_str()
                    .or_else(|| collector["type"].as_str())
                    .unwrap_or("unknown");
                
                let url = collector["feed_url"].as_str()
                    .or_else(|| collector["url"].as_str())
                    .or_else(|| collector["base_url"].as_str());
                
                if let Some(url) = url {
                    test_sites.push((name.to_string(), url.to_string(), id.to_string(), collector_type.to_string()));
                }
            }
        }
    }
    
    println!("✓ Found {} collectors to test (excluding offline sites)\n", test_sites.len());
    println!("🔬 Starting adaptive access test...\n");

    let mut results = Vec::new();
    
    for (idx, (name, url, _id, ctype)) in test_sites.iter().enumerate() {
        println!("  [{:3}/{}] Testing {}...", idx + 1, test_sites.len(), name);
        let result = test_adaptive_strategies(name, url, ctype).await;
        print_result(&result);
        results.push(result);
        
        // Delay to avoid rate limits
        if idx % 10 == 9 {
            println!("\n    ⏸️  Pausing 2s...\n");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
    }

    // Save results
    let json_results = serde_json::to_string_pretty(&results).unwrap();
    fs::write("adaptive_test_results.json", json_results).expect("Failed to write results");
    println!("\n💾 Results saved to adaptive_test_results.json");

    // Print summary
    print_summary(&results);
}

async fn test_adaptive_strategies(name: &str, url: &str, collector_type: &str) -> AdaptiveTestResult {
    let mut result = AdaptiveTestResult {
        name: name.to_string(),
        original_url: url.to_string(),
        original_type: collector_type.to_string(),
        direct_http_works: false,
        antibot_headers_works: false,
        rss_autodiscover_works: false,
        rss_autodiscover_url: None,
        alternative_url_works: false,
        alternative_url: None,
        success: false,
        working_strategy: String::new(),
        working_url: String::new(),
        needs_playwright: false,
        blockers_found: Vec::new(),
        recommendation: String::new(),
    };

    // Strategy 1: Direct HTTP (simple GET)
    let simple_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    
    if let Ok((accessible, has_content, blockers)) = test_url_full(&simple_client, url).await {
        result.blockers_found.extend(blockers.clone());
        if accessible && has_content {
            result.direct_http_works = true;
            result.success = true;
            result.working_strategy = "direct_http".to_string();
            result.working_url = url.to_string();
            result.recommendation = "Working perfectly with simple HTTP".to_string();
            return result;
        }
    }

    // Strategy 2: Anti-bot headers
    let antibot_client = build_antibot_client();
    if let Ok((accessible, has_content, blockers)) = test_url_full(&antibot_client, url).await {
        result.blockers_found.extend(blockers.clone());
        if accessible && has_content {
            result.antibot_headers_works = true;
            result.success = true;
            result.working_strategy = "antibot_headers".to_string();
            result.working_url = url.to_string();
            result.recommendation = "Works with anti-bot headers".to_string();
            return result;
        }
    }

    // Strategy 3: RSS Autodiscover (try to find RSS feed if not already RSS)
    if collector_type != "rss" {
        if let Some(rss_url) = autodiscover_rss(&antibot_client, url).await {
            if let Ok((accessible, has_content, _)) = test_url_full(&antibot_client, &rss_url).await {
                if accessible && has_content {
                    result.rss_autodiscover_works = true;
                    result.rss_autodiscover_url = Some(rss_url.clone());
                    result.success = true;
                    result.working_strategy = "rss_autodiscover".to_string();
                    result.working_url = rss_url;
                    result.recommendation = format!("Convert to RSS: {}", result.rss_autodiscover_url.as_ref().unwrap());
                    return result;
                }
            }
        }
    }

    // Strategy 4: Try alternative URL patterns
    let base_domain = extract_base_domain(url);
    let alternative_paths = vec![
        format!("{}/blog", base_domain),
        format!("{}/news", base_domain),
        format!("{}/feed", base_domain),
        format!("{}/rss", base_domain),
        format!("{}/blog/feed", base_domain),
        format!("{}/news/feed", base_domain),
        format!("{}/en/blog", base_domain),
    ];

    for alt_url in alternative_paths {
        if alt_url == url {
            continue; // Skip if same as original
        }
        
        if let Ok((accessible, has_content, _)) = test_url_full(&antibot_client, &alt_url).await {
            if accessible && has_content {
                result.alternative_url_works = true;
                result.alternative_url = Some(alt_url.clone());
                result.success = true;
                result.working_strategy = "alternative_url".to_string();
                result.working_url = alt_url;
                result.recommendation = format!("Use alternative URL: {}", result.alternative_url.as_ref().unwrap());
                return result;
            }
        }
    }

    // Strategy 5: Check if it's just a cookie banner (can be handled by Playwright)
    if result.blockers_found.contains(&"cookies".to_string()) && !result.blockers_found.contains(&"403".to_string()) {
        result.needs_playwright = true;
        result.recommendation = "Needs Playwright for cookie acceptance".to_string();
        return result;
    }

    // All strategies failed - needs Playwright
    result.needs_playwright = true;
    result.recommendation = format!("All strategies failed. Blockers: {:?}. Needs Playwright.", result.blockers_found);
    result
}

async fn test_url_full(client: &Client, url: &str) -> Result<(bool, bool, Vec<String>), Box<dyn std::error::Error>> {
    let mut blockers = Vec::new();
    
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            
            if status.as_u16() == 403 {
                blockers.push("403".to_string());
                return Ok((false, false, blockers));
            }
            
            if !status.is_success() {
                return Ok((false, false, vec!["http_error".to_string()]));
            }
            
            match response.text().await {
                Ok(content) => {
                    let accessible = true;
                    
                    // Check for blockers
                    if (content.contains("cookie") || content.contains("Cookie")) 
                        && content.contains("accept") && content.contains("button") {
                        blockers.push("cookies".to_string());
                    }
                    
                    if content.contains("captcha") || content.contains("recaptcha") {
                        blockers.push("captcha".to_string());
                    }
                    
                    if content.contains("paywall") || (content.contains("subscribe") && content.len() < 10000) {
                        blockers.push("paywall".to_string());
                    }
                    
                    // Check for actual content
                    let has_content = check_has_content(&content);
                    
                    if !has_content {
                        blockers.push("no_content".to_string());
                    }
                    
                    Ok((accessible, has_content && blockers.is_empty(), blockers))
                }
                Err(_) => Ok((false, false, vec!["read_error".to_string()])),
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("timeout") {
                Ok((false, false, vec!["timeout".to_string()]))
            } else {
                Ok((false, false, vec!["network_error".to_string()]))
            }
        }
    }
}

fn check_has_content(html: &str) -> bool {
    if html.len() < 1000 {
        return false;
    }
    
    let document = Html::parse_document(html);
    
    // Check for content indicators
    let content_selectors = vec![
        "article", "main", ".post", ".article", ".content", 
        ".blog-post", "[role='article']", ".entry-content",
        "item", "entry", // RSS/Atom
    ];
    
    for sel_str in content_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>();
                if text.len() > 200 {
                    return true;
                }
            }
        }
    }
    
    // Fallback: check if HTML has reasonable amount of text
    html.len() > 5000
}

async fn autodiscover_rss(client: &Client, url: &str) -> Option<String> {
    // Try to discover RSS feed
    match client.get(url).send().await {
        Ok(response) => {
            if let Ok(content) = response.text().await {
                let document = Html::parse_document(&content);
                
                // Look for RSS link in HTML
                if let Ok(selector) = Selector::parse(r#"link[type="application/rss+xml"], link[type="application/atom+xml"]"#) {
                    for element in document.select(&selector) {
                        if let Some(href) = element.value().attr("href") {
                            let rss_url = if href.starts_with("http") {
                                href.to_string()
                            } else if href.starts_with("/") {
                                format!("{}{}", extract_base_domain(url), href)
                            } else {
                                format!("{}/{}", extract_base_domain(url), href)
                            };
                            return Some(rss_url);
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
    
    // Try common RSS URLs
    let base_domain = extract_base_domain(url);
    let rss_candidates = vec![
        format!("{}/feed", base_domain),
        format!("{}/rss", base_domain),
        format!("{}/feed.xml", base_domain),
        format!("{}/rss.xml", base_domain),
        format!("{}/atom.xml", base_domain),
        format!("{}/blog/feed", base_domain),
        format!("{}/news/feed", base_domain),
    ];
    
    for rss_url in rss_candidates {
        if let Ok(response) = client.get(&rss_url).send().await {
            if response.status().is_success() {
                if let Ok(content) = response.text().await {
                    if content.contains("<rss") || content.contains("<feed") {
                        return Some(rss_url);
                    }
                }
            }
        }
    }
    
    None
}

fn extract_base_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(domain) = parsed.domain() {
            return format!("{}://{}", parsed.scheme(), domain);
        }
    }
    url.to_string()
}

fn build_antibot_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        header::HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-site"),
        header::HeaderValue::from_static("none"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-mode"),
        header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-user"),
        header::HeaderValue::from_static("?1"),
    );
    headers.insert(
        header::HeaderName::from_static("sec-fetch-dest"),
        header::HeaderValue::from_static("document"),
    );
    headers.insert(
        header::REFERER,
        header::HeaderValue::from_static("https://www.google.com/"),
    );

    Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("Failed to create HTTP client")
}

fn print_result(result: &AdaptiveTestResult) {
    if result.success {
        println!("      ✓ SUCCESS via {}", result.working_strategy);
        if result.working_url != result.original_url {
            println!("        New URL: {}", result.working_url);
        }
    } else if result.needs_playwright {
        println!("      ⚠ NEEDS PLAYWRIGHT");
        if !result.blockers_found.is_empty() {
            println!("        Blockers: {:?}", result.blockers_found);
        }
    } else {
        println!("      ✗ FAILED - All strategies exhausted");
        if !result.blockers_found.is_empty() {
            println!("        Blockers: {:?}", result.blockers_found);
        }
    }
}

fn print_summary(results: &[AdaptiveTestResult]) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  ADAPTIVE ACCESS TEST - FINAL SUMMARY");
    println!("═══════════════════════════════════════════════════════════\n");

    let total = results.len();
    let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
    let need_pw: Vec<_> = results.iter().filter(|r| r.needs_playwright && !r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success && !r.needs_playwright).collect();

    println!("📊 OVERALL RESULTS ({} sites tested):", total);
    println!("   ✅ Successfully accessible:  {:3} ({:5.1}%)", successful.len(), (successful.len() as f64 / total as f64) * 100.0);
    println!("   ⚠️  Need Playwright:         {:3} ({:5.1}%)", need_pw.len(), (need_pw.len() as f64 / total as f64) * 100.0);
    println!("   ✗  Failed (all strategies): {:3} ({:5.1}%)", failed.len(), (failed.len() as f64 / total as f64) * 100.0);

    // Strategy breakdown
    println!("\n📋 SUCCESSFUL STRATEGIES:");
    let mut strategy_counts = std::collections::HashMap::new();
    for r in &successful {
        *strategy_counts.entry(r.working_strategy.as_str()).or_insert(0) += 1;
    }
    let mut strategies: Vec<_> = strategy_counts.into_iter().collect();
    strategies.sort_by(|a, b| b.1.cmp(&a.1));
    for (strategy, count) in strategies {
        println!("   • {}: {} sites", strategy, count);
    }

    // Sites with new URLs
    let url_changes: Vec<_> = successful.iter().filter(|r| r.working_url != r.original_url).collect();
    if !url_changes.is_empty() {
        println!("\n🔄 SITES WITH NEW WORKING URLS ({}):", url_changes.len());
        for r in url_changes {
            println!("   • {}", r.name);
            println!("     Old: {}", r.original_url);
            println!("     New: {}", r.working_url);
        }
    }

    // RSS discoveries
    let rss_discoveries: Vec<_> = successful.iter().filter(|r| r.rss_autodiscover_works).collect();
    let rss_count = rss_discoveries.len();
    if !rss_discoveries.is_empty() {
        println!("\n🆕 RSS FEEDS DISCOVERED ({}):", rss_count);
        for r in &rss_discoveries {
            println!("   • {} → {}", r.name, r.rss_autodiscover_url.as_ref().unwrap());
        }
    }

    // Still need Playwright
    if !need_pw.is_empty() {
        println!("\n⚠️  STILL NEED PLAYWRIGHT ({}):", need_pw.len());
        for r in &need_pw {
            println!("   • {} - {:?}", r.name, r.blockers_found);
        }
    }

    // Complete failures
    if !failed.is_empty() {
        println!("\n✗ COMPLETE FAILURES ({}):", failed.len());
        for r in &failed {
            println!("   • {} - {:?}", r.name, r.blockers_found);
        }
    }

    println!("\n💡 RECOMMENDATIONS:");
    println!("   1. Update system_config.json with new working URLs");
    println!("   2. Convert {} sites to RSS (faster/more reliable)", rss_count);
    println!("   3. Add force_js: true for {} Playwright sites", need_pw.len());
    println!("   4. Investigate {} failed sites for manual fixes", failed.len());
    
    println!("\n═══════════════════════════════════════════════════════════\n");
}

