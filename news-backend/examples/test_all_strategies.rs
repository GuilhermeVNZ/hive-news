/// INTELLIGENT Multi-Strategy Collector Test
/// Tests RSS fallback, HTML, and Playwright need for ALL sites
/// Run with: cargo run --example test_all_strategies
///
/// For each site, tries:
/// 1. RSS feed (if HTML) - check common RSS paths
/// 2. Direct HTTP with anti-bot headers
/// 3. Recommends Playwright if blocked/timeout
///
/// Generates optimized system_config.json with best strategy for each site

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SiteConfig {
    name: String,
    original_url: String,
    original_type: String,
    original_id: String,
    
    // Test results
    rss_available: Option<RssFeedInfo>,
    html_works: bool,
    html_status: Option<u16>,
    html_content_size: Option<usize>,
    
    // Recommendation
    recommended_type: String, // "rss", "html", or "playwright"
    recommended_url: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RssFeedInfo {
    url: String,
    works: bool,
    content_size: Option<usize>,
}

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  INTELLIGENT MULTI-STRATEGY COLLECTOR TEST");
    println!("  Testing RSS fallbacks, HTML, and Playwright requirements");
    println!("═══════════════════════════════════════════════════════════\n");

    let client = build_client();
    let mut configs = Vec::new();

    // RSS sites (already optimal, just verify)
    println!("📡 Verifying RSS Collectors (15 sources)...\n");
    let rss_sites = vec![
        ("OpenAI Blog RSS", "https://openai.com", "https://openai.com/blog/rss.xml", "rss_openai"),
        ("Google AI RSS", "https://blog.research.google", "https://blog.research.google/feeds/posts/default", "rss_google_ai"),
        ("NVIDIA News RSS", "https://nvidianews.nvidia.com", "https://nvidianews.nvidia.com/rss.xml", "rss_nvidia"),
        ("Alibaba DAMO RSS", "https://damo.alibaba.com", "https://damo.alibaba.com/news/rss", "rss_alibaba_damo"),
        ("Hugging Face Blog RSS", "https://huggingface.co", "https://huggingface.co/blog/feed.xml", "rss_huggingface"),
        ("ElevenLabs Blog RSS", "https://blog.elevenlabs.io", "https://blog.elevenlabs.io/feed", "rss_elevenlabs"),
        ("Microsoft AI Blog RSS", "https://blogs.microsoft.com/ai", "https://blogs.microsoft.com/ai/feed/", "rss_microsoft_ai"),
        ("IBM Research AI RSS", "https://research.ibm.com", "https://research.ibm.com/blog/feed", "rss_ibm_research"),
        ("Salesforce AI Blog RSS", "https://www.salesforce.com", "https://www.salesforce.com/news/feed/", "rss_salesforce_ai"),
        ("TechCrunch AI RSS", "https://techcrunch.com", "https://techcrunch.com/tag/artificial-intelligence/feed/", "rss_techcrunch_ai"),
        ("VentureBeat AI RSS", "https://venturebeat.com", "https://venturebeat.com/category/ai/feed/", "rss_venturebeat_ai"),
        ("The Verge AI RSS", "https://www.theverge.com", "https://www.theverge.com/rss/group/ai/index.xml", "rss_the_verge_ai"),
        ("Wired AI RSS", "https://www.wired.com", "https://www.wired.com/feed/category/science/ai/latest/rss", "rss_wired_ai"),
        ("MIT Tech Review AI RSS", "https://news.mit.edu", "https://news.mit.edu/topic/artificial-intelligence-rss.xml", "rss_mit_tech_review_ai"),
        ("Nature AI RSS", "https://www.nature.com", "https://www.nature.com/subjects/artificial-intelligence.rss", "rss_nature_ai"),
    ];

    for (idx, (name, base_url, rss_url, id)) in rss_sites.iter().enumerate() {
        print!("  [{:2}/15] {}...", idx + 1, name);
        let config = test_rss_site(&client, name, base_url, rss_url, id).await;
        println!(" {}", get_recommendation_summary(&config));
        configs.push(config);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    // HTML sites - test for RSS fallback AND HTML viability
    println!("\n🌐 Testing HTML Collectors with RSS Detection (62 sources)...\n");
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
        ("Apple ML Highlights", "https://machinelearning.apple.com/highlights", "html_apple_ml"),
        ("Apple ML Research", "https://machinelearning.apple.com/research", "html_apple_ml_research"),
        ("Intel AI", "https://www.intel.com/content/www/us/en/customer-spotlight/overview.html", "html_intel_ai"),
        ("AMD AI", "https://www.amd.com/en/developer/resources/technical-articles.html", "html_amd_ai"),
        ("Stanford HAI", "https://hai.stanford.edu/news", "html_stanford_hai"),
        ("Berkeley AI", "https://bair.berkeley.edu/blog/archive/", "html_berkeley_ai"),
        ("DeepMind Blog", "https://deepmind.google/discover/blog/", "html_deepmind_blog"),
        ("Science", "https://www.science.org/news", "html_science"),
        ("Hive Hub", "https://hive-hub.ai", "html_hive_hub"),
        ("Menlo Ventures", "https://menlovc.com/focus-areas/ai/", "html_menlo_ventures"),
        ("AIResearch News", "https://www.airesearch.news/", "html_airesearch_news"),
        ("The Gradient", "https://thegradient.pub/", "html_the_gradient"),
        ("Robohub", "https://robohub.org/category/news/", "html_robohub"),
        ("IEEE Tech", "https://www.ieee.org/advancing-technology", "html_ieee"),
        ("Robot Report", "https://www.therobotreport.com/category/news/", "html_robot_report"),
        ("Robotics Biz Review", "https://roboticsbusinessreview.com/news", "html_robotics_business_review"),
        ("Quantum Computing Report", "https://quantumcomputingreport.com/news/", "html_quantum_computing_report"),
        ("IBM Quantum Blog", "https://research.ibm.com/blog", "html_ibm_quantum"),
        ("Quanta Magazine", "https://www.quantamagazine.org/computer-science/", "html_quanta_quantum"),
        ("Rigetti", "https://www.globenewswire.com/newsroom", "html_rigetti"),
        ("IonQ", "https://ionq.com/news", "html_ionq"),
        ("D-Wave", "https://www.dwavequantum.com/company/newsroom/", "html_dwave"),
        ("Quantinuum", "https://www.quantinuum.com/news/news", "html_quantinuum"),
        ("Pasqal", "https://www.pasqal.com/newsroom/", "html_pasqal"),
        ("Boston Dynamics", "https://bostondynamics.com/blog/", "html_boston_dynamics"),
        ("Robotics Online", "https://www.automate.org/robotics/news", "html_robotics_org"),
        ("ABB Robotics", "https://global.abb/group/en/media", "html_abb_robotics"),
        ("KUKA Robotics", "https://www.kuka.com/en-us/press/media-news", "html_kuka"),
        ("Anthropic Research", "https://www.anthropic.com/research", "html_anthropic_research"),
        ("Adept AI", "https://www.adept.ai/blog", "html_adept"),
        ("AssemblyAI", "https://www.assemblyai.com/blog", "html_assemblyai"),
        ("Replicate", "https://replicate.com/blog", "html_replicate"),
        ("LangChain", "https://blog.langchain.com/", "html_langchain"),
        ("Pinecone", "https://www.pinecone.io/blog/", "html_pinecone"),
        ("Weaviate", "https://weaviate.io/blog", "html_weaviate"),
        ("Together AI", "https://www.together.ai/blog", "html_together"),
        ("Anyscale", "https://www.anyscale.com/press", "html_anyscale"),
        ("Modal", "https://modal.com/blog", "html_modal"),
        ("Cursor", "https://www.cursor.com/blog", "html_cursor"),
        ("Continual AI", "https://www.continual.ai/blog", "html_continual"),
        ("Fast.ai", "https://www.fast.ai/", "html_fastai"),
        ("EleutherAI", "https://www.eleuther.ai/", "html_eleuther"),
        ("Xanadu Quantum", "https://www.xanadu.ai/blog/", "html_xanadu"),
        ("Infleqtion", "https://infleqtion.com/blog/", "html_infleqtion"),
        ("ColdQuanta", "https://www.coldquanta.com/news", "html_coldquanta"),
        ("QCI", "https://www.quantumcomputinginc.com/news", "html_qci"),
        ("Universal Robots", "https://www.universal-robots.com/blog/", "html_universal_robots"),
        ("OMRON", "https://automation.omron.com/en/us/news/", "html_omron"),
        ("Yaskawa", "https://www.yaskawa.com/about-us/media-center/news", "html_yaskawa"),
        ("Agility Robotics", "https://www.agilityrobotics.com/about/press", "html_agility"),
        ("Unitree", "https://www.unitree.com/news", "html_unitree"),
    ];

    for (idx, (name, url, id)) in html_sites.iter().enumerate() {
        print!("  [{:2}/62] {}...", idx + 1, name);
        let config = test_html_site_with_rss_detection(&client, name, url, id).await;
        println!(" {}", get_recommendation_summary(&config));
        configs.push(config);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    // Save results
    let json_results = serde_json::to_string_pretty(&configs).unwrap();
    fs::write("intelligent_test_results.json", json_results).expect("Failed to write results");
    println!("\n💾 Results saved to intelligent_test_results.json");

    // Generate summary and recommendations
    print_intelligent_summary(&configs);
}

fn get_recommendation_summary(config: &SiteConfig) -> String {
    match config.recommended_type.as_str() {
        "rss" => format!("✓ RSS ({})", if config.rss_available.as_ref().map_or(false, |r| r.works) { "verified" } else { "found" }),
        "html" => "✓ HTML works".to_string(),
        "playwright" => "⚠ Need Playwright".to_string(),
        "broken" => "✗ BROKEN (404/error)".to_string(),
        _ => "? Review needed".to_string(),
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
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("Failed to create HTTP client")
}

async fn test_rss_site(client: &Client, name: &str, _base_url: &str, rss_url: &str, id: &str) -> SiteConfig {
    // Test if RSS feed works
    let rss_result = test_url_simple(client, rss_url).await;
    
    SiteConfig {
        name: name.to_string(),
        original_url: rss_url.to_string(),
        original_type: "rss".to_string(),
        original_id: id.to_string(),
        rss_available: Some(RssFeedInfo {
            url: rss_url.to_string(),
            works: rss_result.0,
            content_size: rss_result.1,
        }),
        html_works: false,
        html_status: None,
        html_content_size: None,
        recommended_type: if rss_result.0 { "rss".to_string() } else { "broken".to_string() },
        recommended_url: rss_url.to_string(),
        reason: if rss_result.0 {
            "RSS feed working".to_string()
        } else {
            format!("RSS feed broken ({})", rss_result.2.unwrap_or("unknown error".to_string()))
        },
    }
}

async fn test_html_site_with_rss_detection(client: &Client, name: &str, url: &str, id: &str) -> SiteConfig {
    // Step 1: Try to find RSS feed
    let base_domain = extract_base_domain(url);
    let rss_candidates = generate_rss_candidates(&base_domain, url);
    
    let mut rss_info: Option<RssFeedInfo> = None;
    for rss_url in &rss_candidates {
        let result = test_url_simple(client, rss_url).await;
        if result.0 {
            rss_info = Some(RssFeedInfo {
                url: rss_url.clone(),
                works: true,
                content_size: result.1,
            });
            break;
        }
    }
    
    // Step 2: Test HTML page
    let html_result = test_url_detailed(client, url).await;
    
    // Decide recommendation
    let (recommended_type, recommended_url, reason) = if let Some(ref rss) = rss_info {
        ("rss".to_string(), rss.url.clone(), format!("Found working RSS feed at {}", rss.url))
    } else if html_result.0 {
        ("html".to_string(), url.to_string(), "HTML page accessible".to_string())
    } else if html_result.2 == Some(403) {
        ("playwright".to_string(), url.to_string(), "Blocked (403) - needs Playwright".to_string())
    } else if html_result.2 == Some(404) {
        ("broken".to_string(), url.to_string(), "Not found (404)".to_string())
    } else {
        ("playwright".to_string(), url.to_string(), format!("Failed with: {}", html_result.3.unwrap_or("unknown".to_string())))
    };
    
    SiteConfig {
        name: name.to_string(),
        original_url: url.to_string(),
        original_type: "html".to_string(),
        original_id: id.to_string(),
        rss_available: rss_info,
        html_works: html_result.0,
        html_status: html_result.2,
        html_content_size: html_result.1,
        recommended_type,
        recommended_url,
        reason,
    }
}

fn extract_base_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(domain) = parsed.domain() {
            return format!("{}://{}", parsed.scheme(), domain);
        }
    }
    url.to_string()
}

fn generate_rss_candidates(base_domain: &str, _original_url: &str) -> Vec<String> {
    vec![
        format!("{}/feed", base_domain),
        format!("{}/rss", base_domain),
        format!("{}/feed.xml", base_domain),
        format!("{}/rss.xml", base_domain),
        format!("{}/blog/feed", base_domain),
        format!("{}/blog/rss", base_domain),
        format!("{}/news/feed", base_domain),
        format!("{}/news/rss", base_domain),
        format!("{}/feed/", base_domain),
        format!("{}/rss/", base_domain),
    ]
}

async fn test_url_simple(client: &Client, url: &str) -> (bool, Option<usize>, Option<String>) {
    match client.get(url).send().await {
        Ok(response) => {
            let success = response.status().is_success();
            match response.text().await {
                Ok(content) => (success, Some(content.len()), None),
                Err(e) => (false, None, Some(e.to_string())),
            }
        }
        Err(e) => (false, None, Some(e.to_string())),
    }
}

async fn test_url_detailed(client: &Client, url: &str) -> (bool, Option<usize>, Option<u16>, Option<String>) {
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            let success = status.is_success();
            match response.text().await {
                Ok(content) => (success, Some(content.len()), Some(status_code), None),
                Err(e) => (false, None, Some(status_code), Some(e.to_string())),
            }
        }
        Err(e) => (false, None, None, Some(e.to_string())),
    }
}

fn print_intelligent_summary(configs: &[SiteConfig]) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  INTELLIGENT SUMMARY & RECOMMENDATIONS");
    println!("═══════════════════════════════════════════════════════════\n");

    let total = configs.len();
    let rss_recommended: Vec<_> = configs.iter().filter(|c| c.recommended_type == "rss").collect();
    let html_recommended: Vec<_> = configs.iter().filter(|c| c.recommended_type == "html").collect();
    let playwright_needed: Vec<_> = configs.iter().filter(|c| c.recommended_type == "playwright").collect();
    let broken: Vec<_> = configs.iter().filter(|c| c.recommended_type == "broken").collect();
    
    // Sites that found RSS when they were HTML
    let html_to_rss: Vec<_> = configs.iter()
        .filter(|c| c.original_type == "html" && c.recommended_type == "rss")
        .collect();

    println!("OVERALL STATISTICS:");
    println!("  Total Sites Tested: {}", total);
    println!("  ✓ RSS (best): {} ({:.1}%)", rss_recommended.len(), (rss_recommended.len() as f64 / total as f64) * 100.0);
    println!("  ✓ HTML (good): {} ({:.1}%)", html_recommended.len(), (html_recommended.len() as f64 / total as f64) * 100.0);
    println!("  ⚠ Playwright needed: {} ({:.1}%)", playwright_needed.len(), (playwright_needed.len() as f64 / total as f64) * 100.0);
    println!("  ✗ Broken/404: {} ({:.1}%)", broken.len(), (broken.len() as f64 / total as f64) * 100.0);

    if !html_to_rss.is_empty() {
        println!("\n🎉 FOUND RSS FEEDS FOR {} HTML SITES:", html_to_rss.len());
        for config in &html_to_rss {
            let rss_url = config.rss_available.as_ref().unwrap().url.clone();
            println!("  • {} -> {}", config.name, rss_url);
        }
    }

    if !playwright_needed.is_empty() {
        println!("\n⚠ NEED PLAYWRIGHT ({} sites):", playwright_needed.len());
        for config in &playwright_needed {
            println!("  • {} ({})", config.name, config.original_id);
            println!("    Reason: {}", config.reason);
        }
    }

    if !broken.is_empty() {
        println!("\n✗ BROKEN URLs ({} sites):", broken.len());
        for config in &broken {
            println!("  • {} ({})", config.name, config.original_id);
            println!("    URL: {}", config.original_url);
        }
    }

    println!("\n📋 NEXT STEPS:");
    println!("  1. Update {} HTML collectors to use RSS feeds", html_to_rss.len());
    println!("  2. Add force_js: true for {} Playwright sites", playwright_needed.len());
    println!("  3. Fix/remove {} broken URLs", broken.len());
    println!("  4. Run: cargo run --example generate_optimized_config");
    
    println!("\n═══════════════════════════════════════════════════════════\n");
}

