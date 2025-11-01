// DeepSeek API Client
// Handles communication with DeepSeek API for content generation
use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

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
    ) -> Result<ArticleResponse> {
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert technology journalist writing for a major international news portal (style: Wired, The Verge, TechCrunch)."
                },
                {
                    "role": "user",
                    "content": compressed_prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4000, // Increased for news articles with social content
            "response_format": { "type": "json_object" }
        });
        
        let response = self.client
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
            return Err(anyhow::anyhow!("DeepSeek API error {}: {}", status, error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        // Check if response has choices
        if response_json["choices"].is_null() || response_json["choices"].as_array().map_or(0, |a| a.len()) == 0 {
            eprintln!("❌ No choices in DeepSeek API response");
            eprintln!("❌ Full response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            return Err(anyhow::anyhow!("No choices in DeepSeek API response"));
        }
        
        // Parse article from JSON response
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        if content.is_empty() {
            eprintln!("❌ Empty content in DeepSeek API response");
            eprintln!("❌ Full response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            return Err(anyhow::anyhow!("Empty content in DeepSeek API response"));
        }
        
        // Parse JSON response - handle both formats:
        // Format 1: { "title": "...", "article_text": "...", ... }
        // Format 2: { "title": "...", "article": { "opening_hook": "...", ... }, ... }
        let parsed_json: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON. Content length: {} chars", content.len()))?;
        
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
            eprintln!("⚠️  WARNING: API returned nested 'article' object instead of 'article_text' field");
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
                
                eprintln!("   ✅ Reconstructed article_text from {} nested fields ({} chars) - markdown formatting removed", 
                         parts_count, reconstructed.len());
                reconstructed
            } else {
                // Last resort: serialize the whole object as JSON string
                eprintln!("   ⚠️  Could not extract string fields, using full JSON serialization");
                serde_json::to_string_pretty(article_obj).unwrap_or_default()
            }
        } else {
            return Err(anyhow::anyhow!("Missing 'article_text' or 'article' field in response. Expected format: {{\"title\": \"...\", \"article_text\": \"...\"}}"));
        };
        
        // Extract optional fields
        let subtitle = parsed_json["subtitle"].as_str().map(|s| s.to_string()).unwrap_or_default();
        let image_categories = parsed_json["image_categories"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let x_post = parsed_json["x_post"].as_str().map(|s| s.to_string()).unwrap_or_default();
        let linkedin_post = parsed_json["linkedin_post"].as_str().map(|s| s.to_string()).unwrap_or_default();
        let shorts_script = parsed_json["shorts_script"].as_str().map(|s| s.to_string()).unwrap_or_default();
        
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
    ) -> Result<SocialResponse> {
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a science communication expert creating viral social media content."
                },
                {
                    "role": "user",
                    "content": compressed_prompt
                }
            ],
            "temperature": 0.8,
            "max_tokens": 2000,
            "response_format": { "type": "json_object" }
        });
        
        let response = self.client
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
            return Err(anyhow::anyhow!("DeepSeek API error {}: {}", status, error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        // Check if response has choices
        if response_json["choices"].is_null() || response_json["choices"].as_array().map_or(0, |a| a.len()) == 0 {
            eprintln!("❌ No choices in DeepSeek API response for social content");
            eprintln!("❌ Full response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            return Err(anyhow::anyhow!("No choices in DeepSeek API response for social content"));
        }
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        if content.is_empty() {
            eprintln!("❌ Empty content in DeepSeek API response for social content");
            eprintln!("❌ Full response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            return Err(anyhow::anyhow!("Empty content in DeepSeek API response for social content"));
        }
        
        // Parse JSON response
        let parsed: SocialContentJson = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("❌ Failed to parse social content JSON from DeepSeek response");
                eprintln!("❌ Parse error: {}", e);
                eprintln!("❌ Content (first 1000 chars): {}", &content[..content.len().min(1000)]);
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
