// Test Script for Pixabay Image Search
// Tests the Pixabay integration with re-ranking on existing articles

use std::path::Path;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🖼️  Testing Pixabay Integration with Re-Ranking");
    println!("=========================================\n");

    // 1. Read existing article
    let article_path = Path::new("G:/Hive-Hub/News-main/output/AIResearch/2510.21652");
    let title = std::fs::read_to_string(article_path.join("title.txt"))?;
    let article_text = std::fs::read_to_string(article_path.join("article.md"))?;

    println!("📄 Article: {}", title.trim());
    println!("   Path: {}\n", article_path.display());

    // 2. Extract keywords from BOTH title and article content for better context
    let stop_words = ["with", "for", "and", "the", "from", "this", "that", "with", "have", "been", "from"];
    
    // Extract from title
    let title_keywords: Vec<&str> = title
        .split_whitespace()
        .filter(|w| w.len() > 3 && !stop_words.contains(w))
        .take(2)
        .collect();
    
    // Extract key scientific terms from article (first 200 chars)
    let article_snippet = &article_text.chars().take(200).collect::<String>();
    let important_terms = ["ai", "artificial intelligence", "machine learning", "research", 
                           "scientific", "data", "algorithm", "neural", "model", "science",
                           "study", "discovery", "analysis", "technology"];
    
    let mut keywords = title_keywords;
    
    // Add relevant scientific terms found in article
    for term in &important_terms {
        if article_snippet.to_lowercase().contains(term) {
            keywords.push(term);
            break; // Add only first match
        }
    }
    
    // Ensure we have at least 3 keywords
    while keywords.len() < 3 {
        keywords.push("research");
    }
    
    println!("🔑 Keywords: {:?}\n", keywords);

    // Build search queries with fallback strategy
    // Use more generic scientific terms that work well on Pixabay
    let keywords_lower: Vec<String> = keywords.iter().map(|w| w.to_lowercase()).collect();
    
    let queries_to_try = vec![
        // Try specific combination of keywords
        format!("artificial+intelligence+{}", keywords_lower.iter().take(1).next().unwrap_or(&"research".to_string())),
        // More targeted scientific terms
        "neural+network+research".to_string(),
        "machine+learning+algorithm".to_string(),
        // Generic fallback with varying page to avoid same images
        "science+laboratory".to_string(),
        "research+technology".to_string(),
    ];
    
    println!("🔍 Will try queries: {:?}\n", queries_to_try);

    // 4. Search Pixabay with fallback strategy
    let api_key = "9559858-e44f8dc87bdb1c7d1859152e8";
    let client = reqwest::Client::new();
    
    let mut pixabay_response: Option<PixabayResponse> = None;
    
    for search_query in queries_to_try {
        println!("🌐 Trying query: {}", search_query);
        
        // Use page 2 or 3 to get different results than page 1
        let page = if search_query == "science+laboratory" { 2 } else { 1 };
        let url = format!(
            "https://pixabay.com/api/?key={}&q={}&image_type=photo&orientation=horizontal&per_page=10&page={}&safesearch=true",
            api_key,
            urlencoding::encode(&search_query),
            page
        );
        
        let response = client.get(&url).send().await?;
        
        let response_data: PixabayResponse = response.json().await?;
        
        if !response_data.hits.is_empty() {
            println!("✅ Found {} images on Pixabay with query: {}\n", response_data.total_hits, search_query);
            pixabay_response = Some(response_data);
            break;
        } else {
            println!("   No results for this query, trying next...\n");
        }
    }
    
    if pixabay_response.is_none() {
        println!("⚠️  No images found with any query");
        return Ok(());
    }
    
    let pixabay_response = pixabay_response.unwrap();

    // 5. Show top 3 results
    println!("📊 Top 3 results:");
    for (i, hit) in pixabay_response.hits.iter().take(3).enumerate() {
        println!("   {}. ID: {} | Tags: {}", i + 1, hit.id, hit.tags);
    }
    
    // 6. Download first image
    let best_hit = &pixabay_response.hits[0];
    println!("\n⬇️  Downloading best image (ID: {})...", best_hit.id);
    
    let image_response = client.get(&best_hit.large_image_url).send().await?;
    let bytes = image_response.bytes().await?;
    
    let filename = format!("featured_{}.jpg", "2510_21652");
    let file_path = article_path.join(&filename);
    
    std::fs::write(&file_path, bytes)?;
    
    println!("✅ Image saved: {}", file_path.display());
    
    let metadata = std::fs::metadata(&file_path)?;
    println!("📊 File size: {} bytes\n", metadata.len());

    Ok(())
}
