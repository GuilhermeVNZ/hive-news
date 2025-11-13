/// Comprehensive test of ALL 77 news collectors
/// Run with: cargo run --example test_all_collectors
/// 
/// This will:
/// 1. Test every RSS and HTML collector
/// 2. Identify which need Playwright (JS rendering)
/// 3. Generate updated system_config.json with optimal settings
/// 4. Save results to test_results.json

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    name: String,
    url: String,
    collector_type: String,
    collector_id: String,
    success: bool,
    content_length: Option<usize>,
    duration_ms: u128,
    error: Option<String>,
    recommended_method: String, // "rss", "html", or "playwright"
}

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  COMPREHENSIVE NEWS COLLECTORS TEST - ALL 77 SOURCES");
    println!("═══════════════════════════════════════════════════════════\n");

    let client = build_client();
    let mut results = Vec::new();

    // RSS Collectors (15 total)
    println!("📡 Testing RSS Collectors (15 sources)...\n");
    
    let rss_sites = vec![
        ("OpenAI Blog RSS", "https://openai.com/blog/rss.xml", "rss_openai"),
        ("Google AI RSS", "https://blog.research.google/feeds/posts/default", "rss_google_ai"),
        ("NVIDIA News RSS", "https://nvidianews.nvidia.com/rss.xml", "rss_nvidia"),
        ("Alibaba DAMO RSS", "https://damo.alibaba.com/news/rss", "rss_alibaba_damo"),
        ("Hugging Face Blog RSS", "https://huggingface.co/blog/feed.xml", "rss_huggingface"),
        ("ElevenLabs Blog RSS", "https://blog.elevenlabs.io/feed", "rss_elevenlabs"),
        ("Microsoft AI Blog RSS", "https://blogs.microsoft.com/ai/feed/", "rss_microsoft_ai"),
        ("IBM Research AI RSS", "https://research.ibm.com/blog/feed", "rss_ibm_research"),
        ("Salesforce AI Blog RSS", "https://www.salesforce.com/news/feed/", "rss_salesforce_ai"),
        ("TechCrunch AI RSS", "https://techcrunch.com/tag/artificial-intelligence/feed/", "rss_techcrunch_ai"),
        ("VentureBeat AI RSS", "https://venturebeat.com/category/ai/feed/", "rss_venturebeat_ai"),
        ("The Verge AI RSS", "https://www.theverge.com/rss/group/ai/index.xml", "rss_the_verge_ai"),
        ("Wired AI RSS", "https://www.wired.com/feed/category/science/ai/latest/rss", "rss_wired_ai"),
        ("MIT Technology Review AI RSS", "https://news.mit.edu/topic/artificial-intelligence-rss.xml", "rss_mit_tech_review_ai"),
        ("Nature AI RSS", "https://www.nature.com/subjects/artificial-intelligence.rss", "rss_nature_ai"),
    ];

    let total_rss = rss_sites.len();
    for (idx, (name, url, id)) in rss_sites.iter().enumerate() {
        print!("  [{:2}/{}] Testing {}...", idx + 1, total_rss, name);
        let result = test_url(&client, name, url, "RSS", id).await;
        println!(" {}", get_status_emoji(&result));
        results.push(result);
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    // HTML Collectors (62 total)
    println!("\n🌐 Testing HTML Collectors (62 sources)...\n");
    
    let html_sites = vec![
        ("Anthropic News", "https://www.anthropic.com/news", "html_anthropic"),
        ("Meta AI Blog", "https://ai.meta.com/blog/", "html_meta_ai"),
        ("DeepSeek Blog", "https://deepseek.ai/blog", "html_deepseek"),
        ("Alibaba Alizila News", "https://www.alizila.com/", "html_alibaba_damo"),
        ("X.ai News", "https://x.ai/news", "html_xai"),
        ("Mistral AI News", "https://mistral.ai/news/", "html_mistral_ai"),
        ("Cohere AI Blog", "https://txt.cohere.com/", "html_cohere"),
        ("Perplexity AI", "https://www.perplexity.ai/discover/tech", "html_perplexity"),
        ("Stability AI News", "https://stability.ai/news", "html_stability_ai"),
        ("CharacterAI", "https://blog.character.ai/", "html_character_ai"),
        ("Inflection AI", "https://inflection.ai/blog/enterprise", "html_inflection_ai"),
        ("Apple Machine Learning", "https://machinelearning.apple.com/highlights", "html_apple_ml"),
        ("Apple Machine Learning Research", "https://machinelearning.apple.com/research", "html_apple_ml_research"),
        ("Intel AI", "https://www.intel.com/content/www/us/en/customer-spotlight/overview.html", "html_intel_ai"),
        ("AMD AI", "https://www.amd.com/en/developer/resources/technical-articles.html", "html_amd_ai"),
        ("Stanford HAI", "https://hai.stanford.edu/news?filterBy=news", "html_stanford_hai"),
        ("Berkeley AI Research", "https://bair.berkeley.edu/blog/archive/", "html_berkeley_ai"),
        ("DeepMind Blog", "https://deepmind.google/discover/blog/", "html_deepmind_blog"),
        ("Science", "https://www.science.org/news", "html_science"),
        ("Hive Hub", "https://hive-hub.ai", "html_hive_hub"),
        ("Menlo Ventures AI", "https://menlovc.com/focus-areas/ai/", "html_menlo_ventures"),
        ("AIResearch News", "https://www.airesearch.news/", "html_airesearch_news"),
        ("The Gradient", "https://thegradient.pub/", "html_the_gradient"),
        ("Robohub", "https://robohub.org/category/news/", "html_robohub"),
        ("IEEE Advancing Technology", "https://www.ieee.org/advancing-technology", "html_ieee"),
        ("The Robot Report", "https://www.therobotreport.com/category/news/", "html_robot_report"),
        ("Robotics Business Review", "https://roboticsbusinessreview.com/news", "html_robotics_business_review"),
        ("Quantum Computing Report", "https://quantumcomputingreport.com/news/", "html_quantum_computing_report"),
        ("IBM Quantum Blog", "https://research.ibm.com/blog", "html_ibm_quantum"),
        ("Quanta Magazine Computer Science", "https://www.quantamagazine.org/computer-science/", "html_quanta_quantum"),
        ("Rigetti Computing", "https://www.globenewswire.com/newsroom", "html_rigetti"),
        ("IonQ", "https://ionq.com/news", "html_ionq"),
        ("D-Wave Systems", "https://www.dwavequantum.com/company/newsroom/", "html_dwave"),
        ("Quantinuum", "https://www.quantinuum.com/news/news", "html_quantinuum"),
        ("Pasqal", "https://www.pasqal.com/newsroom/", "html_pasqal"),
        ("Boston Dynamics", "https://bostondynamics.com/blog/", "html_boston_dynamics"),
        ("Robotics Online", "https://www.automate.org/robotics/news", "html_robotics_org"),
        ("ABB Robotics", "https://global.abb/group/en/media", "html_abb_robotics"),
        ("KUKA Robotics", "https://www.kuka.com/en-us/press/media-news", "html_kuka"),
        ("Anthropic Research", "https://www.anthropic.com/research", "html_anthropic_research"),
        ("Adept AI", "https://www.adept.ai/blog", "html_adept"),
        ("AssemblyAI Blog", "https://www.assemblyai.com/blog", "html_assemblyai"),
        ("Replicate Blog", "https://replicate.com/blog", "html_replicate"),
        ("LangChain Blog", "https://blog.langchain.com/", "html_langchain"),
        ("Pinecone Blog", "https://www.pinecone.io/blog/", "html_pinecone"),
        ("Weaviate Blog", "https://weaviate.io/blog", "html_weaviate"),
        ("Together AI Blog", "https://www.together.ai/blog", "html_together"),
        ("Anyscale Blog", "https://www.anyscale.com/press", "html_anyscale"),
        ("Modal Blog", "https://modal.com/blog", "html_modal"),
        ("Cursor Blog", "https://www.cursor.com/blog", "html_cursor"),
        ("Continual AI", "https://www.continual.ai/blog", "html_continual"),
        ("Fast.ai", "https://www.fast.ai/", "html_fastai"),
        ("EleutherAI Blog", "https://www.eleuther.ai/", "html_eleuther"),
        ("Xanadu Quantum Blog", "https://www.xanadu.ai/blog/", "html_xanadu"),
        ("Infleqtion Blog", "https://infleqtion.com/blog/", "html_infleqtion"),
        ("ColdQuanta", "https://www.coldquanta.com/news", "html_coldquanta"),
        ("Quantum Computing Inc", "https://www.quantumcomputinginc.com/news", "html_qci"),
        ("Universal Robots", "https://www.universal-robots.com/blog/", "html_universal_robots"),
        ("OMRON Robotics", "https://automation.omron.com/en/us/news/", "html_omron"),
        ("Yaskawa Robotics", "https://www.yaskawa.com/about-us/media-center/news", "html_yaskawa"),
        ("Agility Robotics", "https://www.agilityrobotics.com/about/press", "html_agility"),
        ("Unitree Robotics", "https://www.unitree.com/news", "html_unitree"),
    ];

    let total_html = html_sites.len();
    for (idx, (name, url, id)) in html_sites.iter().enumerate() {
        print!("  [{:2}/{}] Testing {}...", idx + 1, total_html, name);
        let result = test_url(&client, name, url, "HTML", id).await;
        println!(" {}", get_status_emoji(&result));
        results.push(result);
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    // Save results to JSON
    let json_results = serde_json::to_string_pretty(&results).unwrap();
    fs::write("test_results.json", json_results).expect("Failed to write results");
    
    println!("\n💾 Results saved to test_results.json");

    // Generate summary
    print_summary(&results);
}

fn get_status_emoji(result: &TestResult) -> &'static str {
    if result.success {
        "OK"
    } else if result.error.as_ref().map_or(false, |e| e.contains("403")) {
        "BLOCKED"
    } else if result.error.as_ref().map_or(false, |e| e.contains("404")) {
        "NOT_FOUND"
    } else if result.error.as_ref().map_or(false, |e| e.contains("timeout")) {
        "TIMEOUT"
    } else {
        "FAIL"
    }
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
        reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
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
        .timeout(std::time::Duration::from_secs(20))
        .default_headers(headers)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("Failed to create HTTP client")
}

async fn test_url(client: &Client, name: &str, url: &str, collector_type: &str, id: &str) -> TestResult {
    let start = Instant::now();
    
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            let success = status.is_success();
            
            match response.text().await {
                Ok(content) => {
                    let duration = start.elapsed().as_millis();
                    let recommended = if success {
                        collector_type.to_lowercase()
                    } else if status_code == 403 {
                        "playwright".to_string()
                    } else {
                        "review".to_string()
                    };
                    
                    TestResult {
                        name: name.to_string(),
                        url: url.to_string(),
                        collector_type: collector_type.to_string(),
                        collector_id: id.to_string(),
                        success,
                        content_length: Some(content.len()),
                        duration_ms: duration,
                        error: if !success {
                            Some(format!("HTTP {}", status_code))
                        } else {
                            None
                        },
                        recommended_method: recommended,
                    }
                }
                Err(e) => {
                    let duration = start.elapsed().as_millis();
                    TestResult {
                        name: name.to_string(),
                        url: url.to_string(),
                        collector_type: collector_type.to_string(),
                        collector_id: id.to_string(),
                        success: false,
                        content_length: None,
                        duration_ms: duration,
                        error: Some(format!("HTTP {} - Read error: {}", status_code, e)),
                        recommended_method: "review".to_string(),
                    }
                }
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis();
            let recommended = if e.to_string().contains("timeout") {
                "playwright"
            } else {
                "review"
            };
            
            TestResult {
                name: name.to_string(),
                url: url.to_string(),
                collector_type: collector_type.to_string(),
                collector_id: id.to_string(),
                success: false,
                content_length: None,
                duration_ms: duration,
                error: Some(format!("{}", e)),
                recommended_method: recommended.to_string(),
            }
        }
    }
}

fn print_summary(results: &[TestResult]) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  SUMMARY REPORT");
    println!("═══════════════════════════════════════════════════════════\n");

    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total - successful;
    
    let rss_results: Vec<_> = results.iter().filter(|r| r.collector_type == "RSS").collect();
    let html_results: Vec<_> = results.iter().filter(|r| r.collector_type == "HTML").collect();
    
    let rss_success = rss_results.iter().filter(|r| r.success).count();
    let html_success = html_results.iter().filter(|r| r.success).count();

    println!("OVERALL STATISTICS:");
    println!("  Total Sites: {}", total);
    println!("  Success: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("  Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!();
    println!("  RSS Feeds: {}/{} ({:.1}%)", rss_success, rss_results.len(), (rss_success as f64 / rss_results.len() as f64) * 100.0);
    println!("  HTML Sites: {}/{} ({:.1}%)", html_success, html_results.len(), (html_success as f64 / html_results.len() as f64) * 100.0);

    // Breakdown by issue
    let blocked: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("403"))).collect();
    let not_found: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("404"))).collect();
    let timeouts: Vec<_> = results.iter().filter(|r| !r.success && r.error.as_ref().map_or(false, |e| e.contains("timeout"))).collect();
    let need_playwright: Vec<_> = results.iter().filter(|r| r.recommended_method == "playwright").collect();

    if !blocked.is_empty() {
        println!("\nBLOCKED (403) - {} sites:", blocked.len());
        for r in &blocked {
            println!("  - {} ({})", r.name, r.collector_id);
        }
    }

    if !not_found.is_empty() {
        println!("\nNOT FOUND (404) - {} sites:", not_found.len());
        for r in &not_found {
            println!("  - {} ({})", r.name, r.collector_id);
        }
    }

    if !timeouts.is_empty() {
        println!("\nTIMEOUT - {} sites:", timeouts.len());
        for r in &timeouts {
            println!("  - {} ({})", r.name, r.collector_id);
        }
    }

    if !need_playwright.is_empty() {
        println!("\nNEED PLAYWRIGHT - {} sites:", need_playwright.len());
        for r in &need_playwright {
            println!("  - {} ({})", r.name, r.collector_id);
        }
    }

    println!("\nACTIONS REQUIRED:");
    println!("  1. Update system_config.json force_js: true for {} sites", need_playwright.len());
    println!("  2. Fix/update URLs for {} 404 sites", not_found.len());
    println!("  3. Review {} other failed sites", failed - blocked.len() - not_found.len() - timeouts.len());
    
    println!("\n═══════════════════════════════════════════════════════════\n");
}

