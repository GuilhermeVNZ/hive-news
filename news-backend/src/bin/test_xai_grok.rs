/// Test x.ai collector to verify it finds the Grok 4.1 news
/// Run with: cargo run --bin test-xai-grok
use std::process::Command;
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════");
    println!("  Testing x.ai Collector for Grok 4.1");
    println!("═══════════════════════════════════════\n");

    let base_url = "https://x.ai/news";

    println!("🔍 Testing access to: {}", base_url);
    println!("📋 Checking if Playwright scraper is available...\n");

    // Verificar se scraper.js existe
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let scraper_js = current_dir.join("js").join("scraper.js");

    if !scraper_js.exists() {
        println!("❌ ERROR: Playwright scraper not found at: {}", scraper_js.display());
        println!("\n💡 Please ensure:");
        println!("   1. The js/scraper.js file exists");
        println!("   2. You're running from the news-backend directory");
        println!("   3. Playwright is installed: npm install playwright");
        return;
    }

    println!("✅ Playwright scraper found: {}", scraper_js.display());
    println!("📡 Fetching HTML with Playwright...\n");

    let start = Instant::now();

    // Usar Playwright para renderizar a página
    let output = Command::new("node")
        .arg(scraper_js.as_os_str())
        .arg(base_url)
        .current_dir(&current_dir)
        .output();

    match output {
        Ok(output) => {
            let duration = start.elapsed();

            if output.status.success() {
                let html = String::from_utf8_lossy(&output.stdout).to_string();
                println!("✅ Successfully fetched HTML in {:?}", duration);
                println!("📊 HTML length: {} bytes\n", html.len());

                if html.len() < 1000 {
                    println!("⚠️  WARNING: HTML is very small ({}) bytes", html.len());
                    println!("   This might indicate:");
                    println!("   1. JavaScript rendering failed");
                    println!("   2. Page blocked the request");
                    println!("   3. Page structure changed\n");
                }

                // Verificar se encontra "Grok" e "4.1" no HTML
                let html_lower = html.to_lowercase();
                let has_grok = html_lower.contains("grok");
                let has_41 = html_lower.contains("4.1") || html_lower.contains("4_1") || html_lower.contains("4-1");
                let has_grok_41 = has_grok && has_41;

                println!("🔍 Searching for Grok 4.1 in HTML...");
                println!("   Contains 'grok': {}", if has_grok { "✅ YES" } else { "❌ NO" });
                println!("   Contains '4.1': {}", if has_41 { "✅ YES" } else { "❌ NO" });
                println!("   Contains 'grok 4.1': {}", if has_grok_41 { "✅ YES" } else { "❌ NO" });
                println!();

                // Procurar por links de artigos
                let article_links: Vec<&str> = html
                    .lines()
                    .filter(|line| {
                        let line_lower = line.to_lowercase();
                        (line_lower.contains("href") && line_lower.contains("/news/")) ||
                        (line_lower.contains("href") && line_lower.contains("x.ai/news"))
                    })
                    .take(10)
                    .collect();

                println!("📰 Found {} potential article links:\n", article_links.len());
                for (i, link) in article_links.iter().enumerate() {
                    let preview = if link.len() > 120 {
                        format!("{}...", &link[..120])
                    } else {
                        link.to_string()
                    };
                    println!("  {}. {}", i + 1, preview);
                    
                    let link_lower = link.to_lowercase();
                    if link_lower.contains("grok") && (link_lower.contains("4.1") || link_lower.contains("4_1") || link_lower.contains("4-1")) {
                        println!("     🎯 GROK 4.1 FOUND IN THIS LINK!");
                    }
                }

                // Salvar HTML para debug
                let temp_html_file = current_dir.join(format!("temp_xai_test_{}.html", chrono::Utc::now().timestamp()));
                if let Ok(_) = std::fs::write(&temp_html_file, &html) {
                    println!("\n💾 HTML saved to: {}", temp_html_file.display());
                    println!("   You can inspect this file to debug selectors\n");
                }

                println!("\n═══════════════════════════════════════════");
                if has_grok_41 {
                    println!("✅ SUCCESS: Grok 4.1 content found in HTML!");
                    println!("   The collector should be able to find it.");
                    println!("   If not, check selectors in system_config.json");
                } else {
                    println!("❌ FAILURE: Grok 4.1 NOT found in HTML!");
                    println!("\n💡 Possible reasons:");
                    println!("   1. The article was posted after the HTML was fetched");
                    println!("   2. The page structure changed");
                    println!("   3. JavaScript rendering incomplete (check Playwright logs)");
                    println!("\n🔧 Next steps:");
                    println!("   1. Inspect the saved HTML file manually");
                    println!("   2. Check x.ai/news in a browser");
                    println!("   3. Verify Playwright is rendering correctly");
                    println!("   4. Update selectors in system_config.json if needed");
                }
                println!("═══════════════════════════════════════════\n");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("❌ Playwright failed after {:?}", duration);
                println!("   Exit code: {:?}", output.status.code());
                println!("   Error: {}", stderr);
                println!("\n💡 Possible fixes:");
                println!("   1. Install Playwright: npm install playwright");
                println!("   2. Install browsers: npx playwright install");
                println!("   3. Check if Node.js is available: node --version");
                println!();
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("❌ Failed to execute Playwright after {:?}", duration);
            println!("   Error: {}", e);
            println!("\n💡 Possible fixes:");
            println!("   1. Install Node.js: https://nodejs.org/");
            println!("   2. Install Playwright: npm install playwright");
            println!("   3. Check file permissions on js/scraper.js");
            println!();
        }
    }
}