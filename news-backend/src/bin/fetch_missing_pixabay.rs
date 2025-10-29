// Script to fetch missing Pixabay images for articles
// Fetches images for articles that don't have featured images yet

use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PixabayResponse {
    total: u32,
    #[serde(rename = "totalHits")]
    total_hits: u32,
    hits: Vec<Hit>,
}

#[derive(Debug, Deserialize)]
struct Hit {
    id: u32,
    #[serde(rename = "pageURL")]
    page_url: String,
    #[serde(rename = "largeImageURL")]
    large_image_url: String,
    tags: String,
    // Allow unknown fields
    #[serde(flatten)]
    _extra: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🖼️  Fetching Missing Pixabay Images");
    println!("=====================================\n");

    let api_key = "9559858-e44f8dc87bdb1c7d1859152e8";
    let client = reqwest::Client::new();
    
    // Articles to check
    let articles = vec!["2510.21638", "2510.21652"];
    
    for article_id in articles {
        let article_path = PathBuf::from(format!("G:/Hive-Hub/News-main/output/AIResearch/{}", article_id));
        let featured_path = article_path.join(format!("featured_{}.jpg", article_id.replace(".", "_")));
        
        if featured_path.exists() {
            println!("⏭️  Skipping {} - already has featured image", article_id);
            continue;
        }
        
        println!("📄 Processing: {}", article_id);
        
        // Read article content
        let title = std::fs::read_to_string(article_path.join("title.txt"))
            .with_context(|| format!("Failed to read title for {}", article_id))?;
        let article_text = std::fs::read_to_string(article_path.join("article.md"))
            .with_context(|| format!("Failed to read article for {}", article_id))?;
        
        println!("   Title: {}", title.trim());
        
        // Extract keywords
        let stop_words = ["with", "for", "and", "the", "from", "this", "that", "have", "been"];
        let keywords: Vec<&str> = title
            .split_whitespace()
            .filter(|w| w.len() > 3 && !stop_words.contains(w))
            .take(2)
            .collect();
        
        let queries_to_try = vec![
            format!("artificial+intelligence+{}", keywords.get(0).unwrap_or(&"research")),
            "machine+learning".to_string(),
            "neural+network".to_string(),
            "technology+research".to_string(),
            "ai+science".to_string(),
        ];
        
        // Calculate page number based on article ID to avoid duplicates
        let page = calculate_page_number(article_id);
        println!("   📄 Using page: {}", page);
        
        // Search Pixabay
        let mut images = None;
        
        for (query_idx, query) in queries_to_try.iter().enumerate() {
            println!("   🔍 Trying: {}", query);
            
            // Varia a página para cada query
            let query_page = page + (query_idx as u32);
            
            let url = format!(
                "https://pixabay.com/api/?key={}&q={}&image_type=photo&orientation=horizontal&per_page=20&page={}&safesearch=true",
                api_key,
                urlencoding::encode(query),
                query_page
            );
            
            match client.get(&url).send().await {
                Ok(response) => {
                    match response.json::<PixabayResponse>().await {
                        Ok(data) if !data.hits.is_empty() => {
                            println!("   ✅ Found {} images", data.total_hits);
                            images = Some(data.hits);
                            break;
                        }
                        Ok(data) => {
                            println!("   ⚠️  No images on page {}, trying next query/page...", query_page);
                            // Try next query or reduce page number if too high
                            if query_page > 10 {
                                // Try with page 1 if page too high
                                let page_1_url = format!(
                                    "https://pixabay.com/api/?key={}&q={}&image_type=photo&orientation=horizontal&per_page=10&page=1&safesearch=true",
                                    api_key,
                                    urlencoding::encode(query)
                                );
                                if let Ok(p1_response) = client.get(&page_1_url).send().await {
                                    if let Ok(p1_data) = p1_response.json::<PixabayResponse>().await {
                                        if !p1_data.hits.is_empty() {
                                            println!("   ✅ Found {} images on page 1", p1_data.total_hits);
                                            images = Some(p1_data.hits);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
        
        if let Some(hits) = images {
            // Use article ID to select different image to avoid duplicates
            let after_dot = article_id.split('.').nth(1).unwrap_or("0");
            let num: u32 = after_dot.parse().unwrap_or(0);
            let image_index = (num as usize) % hits.len().max(1);
            let best_hit = &hits[image_index];
            println!("   ⬇️  Downloading image ID: {} (index {}/{})...", best_hit.id, image_index + 1, hits.len());
            
            let image_response = client.get(&best_hit.large_image_url).send().await?;
            let bytes = image_response.bytes().await?;
            
            std::fs::write(&featured_path, bytes.as_ref())?;
            println!("   ✅ Saved: featured_{}.jpg", article_id.replace(".", "_"));
        } else {
            println!("   ⚠️  No images found");
        }
        
        println!();
    }
    
    println!("✅ Done!");
    
    Ok(())
}

fn calculate_page_number(article_id: &str) -> u32 {
    // Extrai o número do ID (ex: "2510.21638" -> números após o ponto)
    if let Some(dot_pos) = article_id.find('.') {
        let after_dot = &article_id[dot_pos + 1..];
        // Usa os últimos dígitos para gerar um número de página (1-20)
        if let Ok(num) = after_dot.parse::<u32>() {
            // Converte para página 1-20 (0 não é válido para Pixabay)
            let page = (num % 20) + 1;
            return page;
        }
    }
    
    // Fallback: usa hash do ID para garantir consistência
    let hash = article_id.chars().map(|c| c as u32).sum::<u32>();
    ((hash % 19) + 1) as u32
}


