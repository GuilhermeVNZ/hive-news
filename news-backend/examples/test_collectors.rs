/// Test all news collectors to verify site accessibility
/// Run with: cargo run --example test_collectors --manifest-path news-backend/Cargo.toml
///
/// This tests HTTP access to all configured news sources with the same
/// headers and configuration used by the actual collectors.

use reqwest::Client;
use std::time::Instant;

#[derive(Debug)]
struct TestResult {
    name: String,
    url: String,
    collector_type: String,
    success: bool,
    content_length: Option<usize>,
    duration_ms: u128,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════");
    println!("  News Collectors Accessibility Test");
    println!("═══════════════════════════════════════════\n");

    // Initialize HTTP client with same headers as HtmlCollector
    let client = build_client();

    let mut results = Vec::new();

    // RSS Collectors
    println!("📡 Testing RSS Collectors...\n");
    
    let rss_sites = vec![
        ("OpenAI", "https://openai.com/blog/rss.xml"),
        ("Google AI", "https://blog.research.google/feeds/posts/default"),
        ("NVIDIA", "https://nvidianews.nvidia.com/rss.xml"),
        ("Alibaba DAMO", "https://damo.alibaba.com/news/rss"),
        ("Hugging Face", "https://huggingface.co/blog/feed.xml"),
        ("ElevenLabs", "https://blog.elevenlabs.io/feed"),
        ("Microsoft AI", "https://blogs.microsoft.com/ai/feed/"),
        ("IBM Research", "https://research.ibm.com/blog/feed"),
        ("Salesforce", "https://www.salesforce.com/news/feed/"),
        ("TechCrunch AI", "https://techcrunch.com/tag/artificial-intelligence/feed/"),
        ("Qualcomm", "https://www.qualcomm.com/news/rss/allnews.xml"),
    ];

    for (name, url) in rss_sites {
        let result = test_url(&client, name, url, "RSS").await;
        print_result(&result);
        results.push(result);
        
        // Polite delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // HTML Collectors
    println!("\n🌐 Testing HTML Collectors...\n");
    
    let html_sites = vec![
        ("Anthropic", "https://www.anthropic.com/news"),
        ("Meta AI", "https://ai.meta.com/blog/"),
        ("DeepSeek", "https://deepseek.ai/blog"),
        ("X.AI", "https://x.ai/news"),
        ("Mistral AI", "https://mistral.ai/news/"),
        ("Cohere", "https://txt.cohere.com/"),
        ("Perplexity", "https://www.perplexity.ai/discover/tech"),
        ("Stability AI", "https://stability.ai/news"),
        ("Character.AI", "https://blog.character.ai/"),
        ("Inflection AI", "https://inflection.ai/blog/enterprise"),
        ("Apple ML Highlights", "https://machinelearning.apple.com/highlights"),
        ("Apple ML Research", "https://machinelearning.apple.com/research"),
        ("Stanford HAI", "https://hai.stanford.edu/news?filterBy=news"),
        ("Berkeley AI", "https://bair.berkeley.edu/blog/archive/"),
        ("DeepMind Blog", "https://deepmind.google/discover/blog/"),
    ];

    for (name, url) in html_sites {
        let result = test_url(&client, name, url, "HTML").await;
        print_result(&result);
        results.push(result);
        
        // Polite delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Generate summary
    print_summary(&results);
}

fn build_client() -> Client {
    let mut headers = reqwest::header::HeaderMap::new();

    // User Agent - simula navegador Chrome real
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );

    // Accept headers
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );

    // Security headers
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        reqwest::header::HeaderValue::from_static("none"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-mode"),
        reqwest::header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-user"),
        reqwest::header::HeaderValue::from_static("?1"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-dest"),
        reqwest::header::HeaderValue::from_static("document"),
    );

    // Referer
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static("https://www.google.com/"),
    );

    // Upgrade-Insecure-Requests
    headers.insert(
        reqwest::header::HeaderName::from_static("upgrade-insecure-requests"),
        reqwest::header::HeaderValue::from_static("1"),
    );

    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .default_headers(headers)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("Failed to create HTTP client")
}

async fn test_url(client: &Client, name: &str, url: &str, collector_type: &str) -> TestResult {
    let start = Instant::now();
    
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            let success = status.is_success();
            
            match response.text().await {
                Ok(content) => {
                    let duration = start.elapsed().as_millis();
                    TestResult {
                        name: name.to_string(),
                        url: url.to_string(),
                        collector_type: collector_type.to_string(),
                        success,
                        content_length: Some(content.len()),
                        duration_ms: duration,
                        error: if !success {
                            Some(format!("HTTP {}", status_code))
                        } else {
                            None
                        },
                    }
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis();
                    TestResult {
                        name: name.to_string(),
                        url: url.to_string(),
                        collector_type: collector_type.to_string(),
                        success: false,
                        content_length: None,
                        duration_ms: duration,
                        error: Some(format!("HTTP {} - Failed to read body: {}", status_code, e)),
                    }
                }
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis();
            TestResult {
                name: name.to_string(),
                url: url.to_string(),
                collector_type: collector_type.to_string(),
                success: false,
                content_length: None,
                duration_ms: duration,
                error: Some(format!("{}", e)),
            }
        }
    }
}

fn print_result(result: &TestResult) {
    if result.success {
        println!(
            "  ✅ {} ({}) - {}KB in {}ms",
            result.name,
            result.collector_type,
            result.content_length.unwrap_or(0) / 1024,
            result.duration_ms
        );
    } else {
        let error_msg = result.error.as_ref().unwrap();
        let short_error = if error_msg.len() > 50 {
            format!("{}...", &error_msg[..50])
        } else {
            error_msg.clone()
        };
        
        // Classify error type
        let error_type = if error_msg.contains("403") || error_msg.contains("Forbidden") {
            "🚫 BLOCKED"
        } else if error_msg.contains("429") {
            "⚠️  RATE LIMITED"
        } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
            "⏱️  TIMEOUT"
        } else if error_msg.contains("404") {
            "❓ NOT FOUND"
        } else {
            "❌ FAILED"
        };
        
        println!(
            "  {} {} ({}) - {}ms - {}",
            error_type, result.name, result.collector_type, result.duration_ms, short_error
        );
    }
}

fn print_summary(results: &[TestResult]) {
    println!("\n═══════════════════════════════════════════");
    println!("  Summary Report");
    println!("═══════════════════════════════════════════\n");

    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total - successful;
    
    let rss_results: Vec<_> = results.iter().filter(|r| r.collector_type == "RSS").collect();
    let html_results: Vec<_> = results.iter().filter(|r| r.collector_type == "HTML").collect();
    
    let rss_success = rss_results.iter().filter(|r| r.success).count();
    let html_success = html_results.iter().filter(|r| r.success).count();

    println!("📊 Overall Statistics:");
    println!("   Total Sites: {}", total);
    println!("   ✅ Success: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("   ❌ Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!();
    println!("   RSS Feeds: {}/{} ({:.1}%)", rss_success, rss_results.len(), (rss_success as f64 / rss_results.len() as f64) * 100.0);
    println!("   HTML Sites: {}/{} ({:.1}%)", html_success, html_results.len(), (html_success as f64 / html_results.len() as f64) * 100.0);

    // Breakdown by error type
    let blocked: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("403") || e.contains("Forbidden"))).collect();
    let rate_limited: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("429"))).collect();
    let timeouts: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("timeout") || e.contains("timed out"))).collect();
    let not_found: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("404"))).collect();

    if !blocked.is_empty() {
        println!("\n🚫 BLOCKED SITES ({}):", blocked.len());
        for result in &blocked {
            println!("   - {} ({})", result.name, result.url);
        }
    }

    if !rate_limited.is_empty() {
        println!("\n⚠️  RATE LIMITED SITES ({}):", rate_limited.len());
        for result in &rate_limited {
            println!("   - {} ({})", result.name, result.url);
        }
    }

    if !timeouts.is_empty() {
        println!("\n⏱️  TIMEOUT SITES ({}):", timeouts.len());
        for result in &timeouts {
            println!("   - {} ({})", result.name, result.url);
        }
    }

    if !not_found.is_empty() {
        println!("\n❓ NOT FOUND (404) SITES ({}):", not_found.len());
        for result in &not_found {
            println!("   - {} ({})", result.name, result.url);
        }
    }

    // Performance stats
    let avg_duration = results.iter().map(|r| r.duration_ms).sum::<u128>() / total as u128;
    let total_content: usize = results.iter().filter(|r| r.success).filter_map(|r| r.content_length).sum();
    
    println!("\n📈 Performance:");
    println!("   Average Request Time: {}ms", avg_duration);
    println!("   Total Content Downloaded: {} MB", total_content / 1024 / 1024);
    if successful > 0 {
        println!("   Avg Content per Site: {} KB", (total_content / successful) / 1024);
    }

    // Recommendations
    println!("\n💡 RECOMMENDATIONS:");
    
    if !blocked.is_empty() {
        println!("   1. ✅ Site-specific headers already implemented in html_collector.rs");
        println!("   2. Consider enabling Playwright for these sites in system_config.json");
        println!("   3. Add proxy rotation if blocking persists");
    }
    
    if !timeouts.is_empty() {
        println!("   4. Increase timeout for slow sites (currently 30s in test, 60s in collector)");
        println!("   5. Enable JavaScript rendering for timeout sites");
    }
    
    if !rate_limited.is_empty() {
        println!("   6. Add exponential backoff for rate-limited sites");
        println!("   7. Increase delay between requests (currently 500ms)");
    }

    if successful == total {
        println!("\n🎉 Perfect! All sites accessible!");
    } else if (successful as f64 / total as f64) >= 0.8 {
        println!("\n👍 Good coverage! Most sites accessible.");
    } else {
        println!("\n⚠️  Many sites failing. Review recommendations above.");
    }

    println!("\n═══════════════════════════════════════════\n");
}
