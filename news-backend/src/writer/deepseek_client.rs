// DeepSeek API Client
// Handles communication with DeepSeek API for content generation
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};

pub struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone)]
pub struct ArticleResponse {
    pub title: String,
    pub subtitle: String,
    pub article_text: String,
    pub image_categories: Vec<String>,
    pub x_post: String,
    pub linkedin_post: String,
    pub shorts_script: String,
}

#[derive(Debug, Clone)]
pub struct SocialResponse {
    pub linkedin_post: String,
    pub x_post: String,
    pub shorts_script: String,
}

impl DeepSeekClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            base_url,
            model,
        }
    }

    pub async fn generate_article(
        &self,
        compressed_prompt: &str,
        temperature: Option<f64>,
    ) -> Result<ArticleResponse> {
        // CRITICAL: Log what is actually being sent to DeepSeek API
        println!("  📤 [DeepSeekClient] Preparing to send prompt to DeepSeek API...");
        println!("  📊 [DeepSeekClient] Prompt length: {} characters", compressed_prompt.len());
        
        // Log preview of what will be sent
        let api_prompt_preview = if compressed_prompt.len() > 1500 {
            format!("{}... [truncated]", &compressed_prompt[..1500])
        } else {
            compressed_prompt.to_string()
        };
        println!("  📄 [DeepSeekClient] Prompt preview (first 1500 chars) that will be sent:");
        println!("     {}", api_prompt_preview);
        
        let temp = temperature.unwrap_or(0.7);
        
        // CRITICAL: Add instructions about minimum article length in system message
        // This ensures they are not removed by prompt compression
        let system_message = r#"You are an expert technology journalist writing for a major international news portal (style: Wired, The Verge, TechCrunch).

CRITICAL REQUIREMENTS FOR article_text FIELD:
- MINIMUM 5 PARAGRAPHS (this is the absolute minimum, not a maximum)
- MINIMUM 600 words (aim for 700-900 words)
- Each paragraph must be substantial (3-5 sentences)
- Paragraphs must be separated by \n\n (two line breaks)
- DO NOT write a single short paragraph - this is NEVER acceptable
- Cover all major sections: Introduction, Methodology, Results, Implications, Limitations
- Use substantial detail from the paper provided"#;
        
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_message
                },
                {
                    "role": "user",
                    "content": compressed_prompt
                }
            ],
            "temperature": temp,
            "max_tokens": 4000, // Increased for news articles with social content
            "response_format": { "type": "json_object" }
        });
        
        // Log request details (without exposing API key)
        println!("  🔗 [DeepSeekClient] API endpoint: {}/chat/completions", self.base_url);
        println!("  🤖 [DeepSeekClient] Model: {}", self.model);
        println!("  🌡️  [DeepSeekClient] Temperature: {}", temp);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("❌ DeepSeek API error status: {}", status);
            eprintln!("❌ Error body: {}", error_text);
            return Err(anyhow::anyhow!(
                "DeepSeek API error {}: {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Check if response has choices
        if response_json["choices"].is_null()
            || response_json["choices"].as_array().map_or(0, |a| a.len()) == 0
        {
            eprintln!("❌ No choices in DeepSeek API response");
            eprintln!(
                "❌ Full response: {}",
                serde_json::to_string_pretty(&response_json).unwrap_or_default()
            );
            return Err(anyhow::anyhow!("No choices in DeepSeek API response"));
        }

        // Parse article from JSON response
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if content.is_empty() {
            eprintln!("❌ Empty content in DeepSeek API response");
            eprintln!(
                "❌ Full response: {}",
                serde_json::to_string_pretty(&response_json).unwrap_or_default()
            );
            return Err(anyhow::anyhow!("Empty content in DeepSeek API response"));
        }

        // Parse JSON response - handle both formats:
        // Format 1: { "title": "...", "article_text": "...", ... }
        // Format 2: { "title": "...", "article": { "opening_hook": "...", ... }, ... }
        let parsed_json: serde_json::Value = serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to parse JSON. Content length: {} chars",
                content.len()
            )
        })?;

        // Extract title
        let title = parsed_json["title"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' field in response"))?
            .to_string();

        // Extract article_text - prefer correct format, fallback to nested structure
        let article_text = if let Some(article_text) = parsed_json["article_text"].as_str() {
            // ✅ Format 1 (CORRECT): article_text is directly in root
            article_text.to_string()
        } else if let Some(article_obj) = parsed_json.get("article") {
            // ⚠️ Format 2 (UNEXPECTED): article is an object with nested fields
            // Warn about incorrect format but try to extract all content
            eprintln!(
                "⚠️  WARNING: API returned nested 'article' object instead of 'article_text' field"
            );
            eprintln!("   This may cause information loss. Consider updating the prompt.");

            // Extract ALL string fields from the nested article object
            // Reconstruct as natural text without markdown formatting or labels
            let mut parts: Vec<(usize, String)> = Vec::new(); // (order_priority, text)

            if let Some(obj) = article_obj.as_object() {
                // Define field order (lower number = appears first)
                let mut field_order = HashMap::new();
                field_order.insert("opening_hook", 1);
                field_order.insert("introduction", 1);
                field_order.insert("body", 2);
                field_order.insert("text", 2);
                field_order.insert("article_text", 2);
                field_order.insert("article_body", 2);
                field_order.insert("key_finding", 3);
                field_order.insert("key_findings", 3);
                field_order.insert("methodology", 4);
                field_order.insert("methods", 4);
                field_order.insert("results", 5);
                field_order.insert("result", 5);
                field_order.insert("analysis", 5);
                field_order.insert("results_analysis", 5);
                field_order.insert("implications", 6);
                field_order.insert("conclusion", 7);
                field_order.insert("discussion", 7);
                field_order.insert("limitations", 8);

                // Collect all text fields with their order
                for (key, value) in obj.iter() {
                    if let Some(text) = value.as_str() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            let order = field_order.get(key.as_str()).copied().unwrap_or(99);
                            parts.push((order, trimmed.to_string()));
                        }
                    }
                }

                // Sort by order, then remove duplicates
                parts.sort_by_key(|(order, _)| *order);

                // Remove duplicates (compare normalized text)
                let mut unique_parts = Vec::new();
                for (order, text) in parts {
                    let normalized = text.to_lowercase().trim().to_string();
                    let is_duplicate = unique_parts.iter().any(|(_, existing): &(_, String)| {
                        existing.to_lowercase().trim() == normalized
                    });
                    if !is_duplicate {
                        unique_parts.push((order, text));
                    }
                }

                // Reconstruct: join parts naturally without markdown formatting
                let parts_count = unique_parts.len();
                let reconstructed = unique_parts
                    .into_iter()
                    .map(|(_, text)| text)
                    .collect::<Vec<_>>()
                    .join("\n\n");

                eprintln!(
                    "   ✅ Reconstructed article_text from {} nested fields ({} chars) - markdown formatting removed",
                    parts_count,
                    reconstructed.len()
                );
                reconstructed
            } else {
                // Last resort: serialize the whole object as JSON string
                eprintln!("   ⚠️  Could not extract string fields, using full JSON serialization");
                serde_json::to_string_pretty(article_obj).unwrap_or_default()
            }
        } else {
            return Err(anyhow::anyhow!(
                "Missing 'article_text' or 'article' field in response. Expected format: {{\"title\": \"...\", \"article_text\": \"...\"}}"
            ));
        };

        // Extract optional fields
        let subtitle = parsed_json["subtitle"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let raw_image_categories = parsed_json["image_categories"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string().to_lowercase()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        
        // Validate and fix image categories
        let image_categories = validate_and_fix_image_categories(&raw_image_categories);
        
        let x_post = parsed_json["x_post"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let linkedin_post = parsed_json["linkedin_post"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let shorts_script = parsed_json["shorts_script"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default();

        Ok(ArticleResponse {
            title,
            subtitle,
            article_text,
            image_categories,
            x_post,
            linkedin_post,
            shorts_script,
        })
    }

    pub async fn generate_social_content(
        &self,
        compressed_prompt: &str,
        temperature: Option<f64>,
    ) -> Result<SocialResponse> {
        let temp = temperature.unwrap_or(0.8);
        // CRITICAL: Ensure prompt contains JSON instructions before sending
        // Add explicit system message to reinforce JSON format requirements
        // Make it absolutely clear this is SOCIAL content, NOT an article
        let system_message = "You are a science communication expert creating viral SOCIAL MEDIA CONTENT ONLY. This is NOT an article request. You MUST return your response as a JSON object with EXACTLY these 3 fields: \"linkedin_post\", \"x_post\", and \"shorts_script\". Do NOT include \"title\", \"article_text\", \"subtitle\", \"image_categories\", or any other fields. This is for SOCIAL MEDIA posts, NOT an article.";
        
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_message
                },
                {
                    "role": "user",
                    "content": compressed_prompt
                }
            ],
            "temperature": temp,
            "max_tokens": 2000,
            "response_format": { "type": "json_object" }
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("❌ DeepSeek API error {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "DeepSeek API error {}: {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Check if response has choices
        if response_json["choices"].is_null()
            || response_json["choices"].as_array().map_or(0, |a| a.len()) == 0
        {
            eprintln!("❌ No choices in DeepSeek API response for social content");
            eprintln!(
                "❌ Full response: {}",
                serde_json::to_string_pretty(&response_json).unwrap_or_default()
            );
            return Err(anyhow::anyhow!(
                "No choices in DeepSeek API response for social content"
            ));
        }

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if content.is_empty() {
            eprintln!("❌ Empty content in DeepSeek API response for social content");
            eprintln!(
                "❌ Full response: {}",
                serde_json::to_string_pretty(&response_json).unwrap_or_default()
            );
            return Err(anyhow::anyhow!(
                "Empty content in DeepSeek API response for social content"
            ));
        }

        // Parse JSON response
        // First, try to parse as SocialContentJson
        let parsed: SocialContentJson = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                // Check if DeepSeek returned article format instead of social format
                let json_value: serde_json::Value = serde_json::from_str(&content)
                    .context("Failed to parse JSON at all")?;
                
                // Check if it's the wrong format (has article fields instead of social fields)
                let has_title = json_value.get("title").is_some();
                let has_article_text = json_value.get("article_text").is_some();
                let has_image_categories = json_value.get("image_categories").is_some();
                
                if has_title || has_article_text || has_image_categories {
                    eprintln!("❌ CRITICAL ERROR: DeepSeek returned ARTICLE format instead of SOCIAL format!");
                    eprintln!("❌ Received fields: {:?}", json_value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                    eprintln!("❌ Expected fields: linkedin_post, x_post, shorts_script");
                    eprintln!("❌ This indicates the prompt JSON instructions were not followed.");
                    eprintln!("❌ Full response (first 2000 chars): {}", &content[..content.len().min(2000)]);
                    return Err(anyhow::anyhow!(
                        "DeepSeek returned wrong format: got article fields (title/article_text/image_categories) instead of social fields (linkedin_post/x_post/shorts_script). This suggests the JSON format instructions in the prompt were ignored or removed by the compressor."
                    ));
                }
                
                // If it's not the article format, it's a different parsing error
                eprintln!("❌ Failed to parse social content JSON from DeepSeek response");
                eprintln!("❌ Parse error: {}", e);
                eprintln!(
                    "❌ Content (first 2000 chars): {}",
                    &content[..content.len().min(2000)]
                );
                return Err(e).context("Failed to parse social content JSON");
            }
        };

        Ok(SocialResponse {
            linkedin_post: parsed.linkedin_post,
            x_post: parsed.x_post,
            shorts_script: parsed.shorts_script,
        })
    }
}

#[derive(Deserialize)]
struct SocialContentJson {
    linkedin_post: String,
    x_post: String,
    shorts_script: String,
}

/// Validates and fixes image categories to ensure they match the allowed list
/// Returns exactly 3 valid categories, mapping invalid ones to valid equivalents
fn validate_and_fix_image_categories(raw_categories: &[String]) -> Vec<String> {
    // Allowed categories (exact list from prompts)
    let allowed: HashSet<&str> = [
        "ai", "coding", "crypto", "data", "ethics", "games", "hardware", 
        "legal", "network", "quantum_computing", "robotics", "science", 
        "security", "sound"
    ].iter().cloned().collect();
    
    // Mapping of invalid categories to valid ones
    let category_mapping: HashMap<&str, &str> = [
        // Physics/quantum related
        ("physics", "science"),
        ("quantum", "quantum_computing"),
        ("quantum_mechanics", "quantum_computing"),
        ("quantum_physics", "quantum_computing"),
        // AI/ML related
        ("machine_learning", "ai"),
        ("ml", "ai"),
        ("deep_learning", "ai"),
        ("neural_networks", "ai"),
        ("neural_network", "ai"),
        ("generative_ai", "ai"),
        ("computer_vision", "ai"),
        ("cv", "ai"),
        ("natural_language_processing", "ai"),
        ("nlp", "ai"),
        ("reinforcement_learning", "ai"),
        ("rl", "ai"),
        ("artificial_intelligence", "ai"),
        // Programming/software
        ("programming", "coding"),
        ("software", "coding"),
        ("development", "coding"),
        ("code", "coding"),
        // Data related
        ("databases", "data"),
        ("database", "data"),
        ("data_storage", "data"),
        ("data_analysis", "data"),
        ("big_data", "data"),
        // Security related
        ("cybersecurity", "security"),
        ("cyber_security", "security"),
        ("privacy", "security"),
        // Hardware related
        ("electronics", "hardware"),
        ("processors", "hardware"),
        ("chips", "hardware"),
        ("computing", "hardware"),
        // Network related
        ("connectivity", "network"),
        ("networking", "network"),
        ("communication", "network"),
        // Other
        ("biology", "science"),
        ("chemistry", "science"),
        ("medical", "science"),
        ("research", "science"),
    ].iter().cloned().collect();
    
    let mut valid_categories = Vec::new();
    let mut used = HashSet::new();
    
    // First pass: collect valid categories and map invalid ones
    for cat in raw_categories.iter() {
        let normalized = cat.trim().to_lowercase();
        
        if normalized.is_empty() {
            continue;
        }
        
        // Check if it's already valid
        if allowed.contains(normalized.as_str()) {
            if !used.contains(&normalized) {
                valid_categories.push(normalized.clone());
                used.insert(normalized);
            }
        } else if let Some(&mapped) = category_mapping.get(normalized.as_str()) {
            // Map invalid category to valid one
            let mapped_str = mapped.to_string();
            if !used.contains(&mapped_str) {
                eprintln!("  ⚠️  Mapped invalid category '{}' to '{}'", cat, mapped);
                valid_categories.push(mapped_str.clone());
                used.insert(mapped_str);
            }
        } else {
            // Try partial matching (e.g., "quantum_computing" contains "quantum")
            let mut found_match = false;
            for &allowed_cat in allowed.iter() {
                if normalized.contains(allowed_cat) || allowed_cat.contains(normalized.as_str()) {
                    let allowed_cat_str = allowed_cat.to_string();
                    if !used.contains(&allowed_cat_str) {
                        eprintln!("  ⚠️  Mapped '{}' to '{}' (partial match)", cat, allowed_cat);
                        valid_categories.push(allowed_cat_str.clone());
                        used.insert(allowed_cat_str);
                        found_match = true;
                        break;
                    }
                }
            }
            
            if !found_match {
                eprintln!("  ⚠️  Invalid category '{}' - will be replaced with fallback", cat);
            }
        }
        
        // Stop if we have 3 valid categories
        if valid_categories.len() >= 3 {
            break;
        }
    }
    
    // Fill up to 3 categories with fallbacks if needed
    let fallbacks = ["science", "ai", "hardware"];
    for fallback in fallbacks.iter() {
        if valid_categories.len() >= 3 {
            break;
        }
        let fallback_str = fallback.to_string();
        if !used.contains(&fallback_str) {
            eprintln!("  ⚠️  Using fallback category '{}' to complete list", fallback);
            valid_categories.push(fallback_str.clone());
            used.insert(fallback_str);
        }
    }
    
    // Ensure we have exactly 3 (or less if we couldn't find enough)
    valid_categories.truncate(3);
    
    if valid_categories.len() != raw_categories.len() || 
       raw_categories.iter().any(|c| !allowed.contains(c.trim().to_lowercase().as_str())) {
        eprintln!("  ⚠️  Category validation: {:?} -> {:?}", raw_categories, valid_categories);
    }
    
    valid_categories
}
