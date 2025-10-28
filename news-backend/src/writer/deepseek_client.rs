// DeepSeek API Client
// Handles communication with DeepSeek API for content generation
use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone)]
pub struct ArticleResponse {
    pub title: String,
    pub article_text: String,
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
                    "content": "You are a PhD-level science journalist writing for Nature/Science magazine."
                },
                {
                    "role": "user",
                    "content": compressed_prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 3000,
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
        
        // Parse article from JSON response
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        // Parse JSON response
        #[derive(Debug, serde::Deserialize)]
        struct ArticleContentJson {
            title: String,
            article_text: String,
        }
        
        let parsed: ArticleContentJson = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse article JSON. Content: {}", &content[..content.len().min(500)]))?;
        
        Ok(ArticleResponse {
            title: parsed.title,
            article_text: parsed.article_text,
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
        
        // Log response for debugging
        if response_json["choices"].is_null() || response_json["choices"].as_array().map_or(0, |a| a.len()) == 0 {
            eprintln!("❌ No choices in DeepSeek response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            return Err(anyhow::anyhow!("No choices in DeepSeek response"));
        }
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        // Parse JSON response
        let parsed: SocialContentJson = serde_json::from_str(&content)
            .context("Failed to parse social content JSON")?;
        
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
