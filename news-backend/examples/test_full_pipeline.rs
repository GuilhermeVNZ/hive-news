/// COMPLETE END-TO-END Collector Test
/// Tests not just RSS availability, but actual article accessibility
///
/// Run with: cargo run --example test_full_pipeline --release
///
/// For each site:
/// 1. Try RSS feed
/// 2. Extract article URLs from feed
/// 3. Try accessing actual articles (detect cookies, paywalls, 403s)
/// 4. If articles blocked, try with Playwright headers
/// 5. Only recommend RSS if articles are accessible
///
/// This ensures we don't just have RSS feeds, but can actually scrape content!
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PipelineTestResult {
    name: String,
    original_url: String,
    original_type: String,
    original_id: String,

    // RSS test
    rss_feed_works: bool,
    rss_feed_url: Option<String>,
    rss_article_count: usize,

    // Article accessibility test
    sample_article_urls: Vec<String>,
    articles_accessible: bool,
    article_access_rate: f64, // 0.0 to 1.0
    blocked_by: Vec<String>,  // "cookies", "paywall", "403", "timeout", etc

    // HTML direct test
    html_page_accessible: bool,
    html_has_content: bool,
    html_blocked_by: Vec<String>,

    // Final recommendation
    recommended_type: String, // "rss", "html", "playwright", or "broken"
    recommended_url: String,
    force_js: bool,
    reason: String,
}

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  COMPLETE END-TO-END PIPELINE TEST");
    println!("  Testing RSS feeds AND article accessibility");
    println!("═══════════════════════════════════════════════════════════\n");

    let client = build_client();
    let mut results = Vec::new();

    // Load ALL sites from system_config.json
    println!("📋 Loading sites from system_config.json...\n");

    let config_content =
        fs::read_to_string("system_config.json").expect("Failed to read system_config.json");
    let config: Value =
        serde_json::from_str(&config_content).expect("Failed to parse system_config.json");

    let sites = config["sites"]
        .as_object()
        .expect("No 'sites' key in config");

    let mut test_sites = Vec::new();

    for (site_key, site_config) in sites {
        if let Some(collectors) = site_config["collectors"].as_array() {
            for collector in collectors {
                let name = collector["name"].as_str().unwrap_or(site_key);
                let id = collector["id"].as_str().unwrap_or(site_key);
                let enabled = collector["enabled"].as_bool().unwrap_or(true);

                if !enabled {
                    continue; // Skip disabled collectors
                }

                // Try different field names for collector type
                let collector_type = collector["collector_type"]
                    .as_str()
                    .or_else(|| collector["type"].as_str())
                    .unwrap_or("unknown");

                // Try to find URL - check multiple possible fields
                let url = collector["feed_url"]
                    .as_str()
                    .or_else(|| collector["url"].as_str())
                    .or_else(|| collector["base_url"].as_str());

                if let Some(url) = url {
                    test_sites.push((
                        name.to_string(),
                        url.to_string(),
                        id.to_string(),
                        collector_type.to_string(),
                    ));
                }
            }
        }
    }

    println!("✓ Found {} collectors to test\n", test_sites.len());
    println!("🔬 Starting comprehensive test...\n");

    for (idx, (name, url, id, ctype)) in test_sites.iter().enumerate() {
        println!(
            "  [{:3}/{}] {} ({})...",
            idx + 1,
            test_sites.len(),
            name,
            ctype
        );
        let result = test_full_pipeline(&client, name, url, id).await;
        print_pipeline_result(&result);
        results.push(result);

        // Small delay to avoid hammering sites
        if idx % 10 == 9 {
            println!("\n    ⏸️  Pausing 2s to avoid rate limits...\n");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
    }

    // Save results
    let json_results = serde_json::to_string_pretty(&results).unwrap();
    fs::write("pipeline_test_results.json", json_results).expect("Failed to write results");
    println!("\n💾 Results saved to pipeline_test_results.json");

    // Generate summary
    print_pipeline_summary(&results);
}

async fn test_full_pipeline(
    client: &Client,
    name: &str,
    url: &str,
    id: &str,
) -> PipelineTestResult {
    let base_domain = extract_base_domain(url);

    // Step 1: Check for RSS feed
    let (rss_url, rss_works, article_urls) =
        test_rss_and_extract_articles(client, &base_domain).await;

    let mut sample_urls = article_urls.clone();
    sample_urls.truncate(3); // Test first 3 articles

    // Step 2: If RSS found, test if articles are accessible
    let mut articles_accessible = false;
    let mut article_access_rate = 0.0;
    let mut blocked_by = Vec::new();

    if rss_works && !sample_urls.is_empty() {
        let (accessible, rate, blocks) = test_article_accessibility(client, &sample_urls).await;
        articles_accessible = accessible;
        article_access_rate = rate;
        blocked_by = blocks;
    }

    // Step 3: Test HTML page accessibility AND content extraction
    let (html_accessible, html_has_content, html_blocked) = test_html_page_full(client, url).await;

    // Step 4: Decide recommendation
    let (recommended_type, recommended_url, force_js, reason) = decide_recommendation(
        rss_works,
        rss_url.as_deref(),
        articles_accessible,
        article_access_rate,
        html_accessible,
        html_has_content,
        url,
        &blocked_by,
        &html_blocked,
    );

    PipelineTestResult {
        name: name.to_string(),
        original_url: url.to_string(),
        original_type: "html".to_string(),
        original_id: id.to_string(),
        rss_feed_works: rss_works,
        rss_feed_url: rss_url,
        rss_article_count: article_urls.len(),
        sample_article_urls: sample_urls,
        articles_accessible,
        article_access_rate,
        blocked_by,
        html_page_accessible: html_accessible,
        html_has_content,
        html_blocked_by: html_blocked,
        recommended_type,
        recommended_url,
        force_js,
        reason,
    }
}

async fn test_rss_and_extract_articles(
    client: &Client,
    base_domain: &str,
) -> (Option<String>, bool, Vec<String>) {
    let rss_candidates = vec![
        format!("{}/feed", base_domain),
        format!("{}/rss", base_domain),
        format!("{}/feed.xml", base_domain),
        format!("{}/rss.xml", base_domain),
        format!("{}/blog/feed", base_domain),
        format!("{}/news/feed", base_domain),
    ];

    for rss_url in rss_candidates {
        if let Ok(response) = client.get(&rss_url).send().await {
            if response.status().is_success() {
                if let Ok(content) = response.text().await {
                    let article_urls = extract_article_urls_from_rss(&content);
                    if !article_urls.is_empty() {
                        return (Some(rss_url), true, article_urls);
                    }
                }
            }
        }
    }

    (None, false, Vec::new())
}

fn extract_article_urls_from_rss(rss_content: &str) -> Vec<String> {
    let mut urls = Vec::new();

    // Parse RSS/Atom feed for <link> tags
    let document = Html::parse_document(rss_content);

    // Try RSS <link> tags
    if let Ok(link_selector) = Selector::parse("item > link, entry > link") {
        for element in document.select(&link_selector) {
            if let Some(url) = element.text().next() {
                let trimmed = url.trim();
                if !trimmed.is_empty() && trimmed.starts_with("http") {
                    urls.push(trimmed.to_string());
                }
            }
            // Also check href attribute (Atom feeds)
            if let Some(href) = element.value().attr("href") {
                if href.starts_with("http") {
                    urls.push(href.to_string());
                }
            }
        }
    }

    urls.truncate(5); // Limit to 5 articles for testing
    urls
}

async fn test_article_accessibility(
    client: &Client,
    article_urls: &[String],
) -> (bool, f64, Vec<String>) {
    if article_urls.is_empty() {
        return (false, 0.0, vec!["no_articles".to_string()]);
    }

    let mut accessible_count = 0;
    let mut blocked_by = Vec::new();

    for url in article_urls {
        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    if let Ok(content) = response.text().await {
                        // Check for common blockers
                        if content.contains("cookie") && content.contains("accept")
                            || content.contains("gdpr")
                        {
                            if !blocked_by.contains(&"cookies".to_string()) {
                                blocked_by.push("cookies".to_string());
                            }
                        } else if content.contains("paywall")
                            || content.contains("subscribe") && content.len() < 5000
                        {
                            if !blocked_by.contains(&"paywall".to_string()) {
                                blocked_by.push("paywall".to_string());
                            }
                        } else if content.len() > 1000 {
                            // Probably accessible
                            accessible_count += 1;
                        }
                    }
                } else if status.as_u16() == 403 {
                    if !blocked_by.contains(&"403".to_string()) {
                        blocked_by.push("403".to_string());
                    }
                }
            }
            Err(_) => {
                if !blocked_by.contains(&"network_error".to_string()) {
                    blocked_by.push("network_error".to_string());
                }
            }
        }
    }

    let access_rate = accessible_count as f64 / article_urls.len() as f64;
    let accessible = access_rate >= 0.5; // At least 50% accessible

    (accessible, access_rate, blocked_by)
}

async fn test_html_page_full(client: &Client, url: &str) -> (bool, bool, Vec<String>) {
    let mut blocked_by = Vec::new();

    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();

            if status.as_u16() == 403 {
                blocked_by.push("403".to_string());
                return (false, false, blocked_by);
            }

            if !status.is_success() {
                return (false, false, vec!["http_error".to_string()]);
            }

            match response.text().await {
                Ok(content) => {
                    let accessible = true;

                    // Check for blockers
                    let has_cookie_wall = (content.contains("cookie")
                        || content.contains("Cookie"))
                        && (content.contains("accept") || content.contains("Accept"))
                        && content.contains("button");

                    let has_paywall = (content.contains("paywall")
                        || content.contains("subscribe")
                        || content.contains("Subscribe"))
                        && content.len() < 10000; // Short page = likely just paywall

                    let has_captcha = content.contains("captcha")
                        || content.contains("recaptcha")
                        || content.contains("hCaptcha");

                    let has_js_only = content.contains("JavaScript")
                        && content.contains("enable")
                        && content.len() < 5000;

                    // Check if has actual content
                    let document = Html::parse_document(&content);
                    let has_articles = check_for_article_content(&document);
                    let has_text = content.len() > 5000; // Reasonable amount of content

                    let has_content = has_articles || has_text;

                    // Track blockers
                    if has_cookie_wall {
                        blocked_by.push("cookies".to_string());
                    }
                    if has_paywall {
                        blocked_by.push("paywall".to_string());
                    }
                    if has_captcha {
                        blocked_by.push("captcha".to_string());
                    }
                    if has_js_only {
                        blocked_by.push("js_required".to_string());
                    }
                    if !has_content {
                        blocked_by.push("no_content".to_string());
                    }

                    (accessible, has_content && blocked_by.is_empty(), blocked_by)
                }
                Err(_) => (false, false, vec!["read_error".to_string()]),
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("timeout") {
                (false, false, vec!["timeout".to_string()])
            } else {
                (false, false, vec!["network_error".to_string()])
            }
        }
    }
}

fn check_for_article_content(document: &Html) -> bool {
    // Check for common article/content selectors
    let content_selectors = vec![
        "article",
        "main",
        ".post",
        ".article",
        ".content",
        ".blog-post",
        "[role='article']",
        ".entry-content",
    ];

    for sel_str in content_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>();
                if text.len() > 500 {
                    // Reasonable article length
                    return true;
                }
            }
        }
    }

    false
}

fn extract_base_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(domain) = parsed.domain() {
            return format!("{}://{}", parsed.scheme(), domain);
        }
    }
    url.to_string()
}

fn decide_recommendation(
    rss_works: bool,
    rss_url: Option<&str>,
    articles_accessible: bool,
    article_access_rate: f64,
    html_accessible: bool,
    html_has_content: bool,
    original_url: &str,
    rss_blocked_by: &[String],
    html_blocked_by: &[String],
) -> (String, String, bool, String) {
    // Priority 1: RSS with ACTUALLY accessible articles (BEST)
    if rss_works && articles_accessible && article_access_rate >= 0.5 {
        return (
            "rss".to_string(),
            rss_url.unwrap().to_string(),
            false,
            format!(
                "✓ RSS + {}% articles accessible",
                (article_access_rate * 100.0) as i32
            ),
        );
    }

    // Priority 2: HTML with actual content and no blockers (GOOD)
    if html_accessible && html_has_content && html_blocked_by.is_empty() {
        return (
            "html".to_string(),
            original_url.to_string(),
            false,
            "✓ HTML page with accessible content".to_string(),
        );
    }

    // Priority 3: RSS exists but articles blocked - need Playwright
    if rss_works && !rss_blocked_by.is_empty() {
        let needs_playwright = rss_blocked_by.contains(&"cookies".to_string())
            || rss_blocked_by.contains(&"403".to_string())
            || rss_blocked_by.contains(&"paywall".to_string());

        if needs_playwright {
            return (
                "rss+playwright".to_string(),
                rss_url.unwrap().to_string(),
                true,
                format!("RSS works but articles blocked: {:?}", rss_blocked_by),
            );
        }
    }

    // Priority 4: HTML accessible but has blockers - need Playwright
    if html_accessible && !html_blocked_by.is_empty() {
        return (
            "playwright".to_string(),
            original_url.to_string(),
            true,
            format!("HTML blocked by: {:?}", html_blocked_by),
        );
    }

    // Priority 5: Everything failed - need Playwright
    (
        "playwright".to_string(),
        original_url.to_string(),
        true,
        "All access methods blocked - requires JS rendering".to_string(),
    )
}

fn build_client() -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        reqwest::header::HeaderValue::from_static("none"),
    );
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static("https://www.google.com/"),
    );

    Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("Failed to create HTTP client")
}

fn print_pipeline_result(result: &PipelineTestResult) {
    let status = if result.recommended_type == "rss" {
        format!(
            "✓ RSS ({}% articles OK)",
            (result.article_access_rate * 100.0) as i32
        )
    } else if result.recommended_type == "rss+playwright" {
        format!("⚠ RSS+PW (RSS OK but articles: {:?})", result.blocked_by)
    } else if result.recommended_type == "html" {
        "✓ HTML (content accessible)".to_string()
    } else {
        format!("⚠ Playwright (HTML blocked: {:?})", result.html_blocked_by)
    };

    println!("      {}", status);

    if result.rss_feed_works {
        println!(
            "      RSS feed: {} ({} articles found)",
            result.rss_feed_url.as_ref().unwrap(),
            result.rss_article_count
        );
    }

    if !result.html_blocked_by.is_empty() {
        println!("      HTML issues: {:?}", result.html_blocked_by);
    }
}

fn print_pipeline_summary(results: &[PipelineTestResult]) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  PIPELINE SUMMARY");
    println!("═══════════════════════════════════════════════════════════\n");

    let rss_working: Vec<_> = results
        .iter()
        .filter(|r| r.recommended_type == "rss")
        .collect();
    let rss_playwright: Vec<_> = results
        .iter()
        .filter(|r| r.recommended_type == "rss+playwright")
        .collect();
    let html_working: Vec<_> = results
        .iter()
        .filter(|r| r.recommended_type == "html")
        .collect();
    let need_playwright: Vec<_> = results
        .iter()
        .filter(|r| r.recommended_type == "playwright")
        .collect();

    println!(
        "✅ RSS with accessible articles: {} sites",
        rss_working.len()
    );
    for r in &rss_working {
        println!(
            "   • {} ({}% accessible)",
            r.name,
            (r.article_access_rate * 100.0) as i32
        );
    }

    if !rss_playwright.is_empty() {
        println!(
            "\n⚠️  RSS exists but articles blocked: {} sites",
            rss_playwright.len()
        );
        for r in &rss_playwright {
            println!("   • {} - Blocked by: {:?}", r.name, r.blocked_by);
        }
    }

    if !html_working.is_empty() {
        println!(
            "\n✓ HTML working (no RSS or RSS not better): {} sites",
            html_working.len()
        );
        for r in &html_working {
            println!("   • {}", r.name);
        }
    }

    if !need_playwright.is_empty() {
        println!("\n⚠️  Need Playwright: {} sites", need_playwright.len());
        for r in &need_playwright {
            println!("   • {}", r.name);
        }
    }

    let total = results.len();
    let working_direct = rss_working.len() + html_working.len();
    let need_pw = rss_playwright.len() + need_playwright.len();

    println!("\n📊 OVERALL STATISTICS ({} collectors):", total);
    println!(
        "   🎯 WORKING DIRECTLY:     {:3} ({:5.1}%)",
        working_direct,
        (working_direct as f64 / total as f64) * 100.0
    );
    println!(
        "   🎯 NEED PLAYWRIGHT:      {:3} ({:5.1}%)",
        need_pw,
        (need_pw as f64 / total as f64) * 100.0
    );

    println!("\n📋 BLOCKER BREAKDOWN:");
    let mut blocker_counts = std::collections::HashMap::new();
    for r in results {
        for blocker in r.blocked_by.iter().chain(r.html_blocked_by.iter()) {
            *blocker_counts.entry(blocker.as_str()).or_insert(0) += 1;
        }
    }
    let mut blockers_vec: Vec<_> = blocker_counts.into_iter().collect();
    blockers_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (blocker, count) in blockers_vec {
        println!("   • {}: {} sites", blocker, count);
    }

    println!("\n💡 KEY INSIGHTS:");
    println!("   • RSS only recommended when articles ACTUALLY accessible");
    println!(
        "   • {} sites have RSS but articles need Playwright",
        rss_playwright.len()
    );
    println!("   • HTML must have extractable content, not just HTTP 200");

    println!("\n📝 NEXT: Run generate_optimized_config to update system_config.json");
    println!("\n═══════════════════════════════════════════════════════════\n");
}
