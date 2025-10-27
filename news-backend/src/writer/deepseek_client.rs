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
    pub article_text: String,
    pub recommended_figure: String,
    pub figure_reason: String,
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
            "max_tokens": 3000
        });
        
        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("DeepSeek API error {}: {}", status, error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        // Parse article and extract recommended figure
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let (article_clean, recommended_figure, figure_reason) = 
            Self::extract_figure_recommendation(&content);
        
        Ok(ArticleResponse {
            article_text: article_clean,
            recommended_figure,
            figure_reason,
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
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("DeepSeek API error {}: {}", status, error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
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
    
    fn extract_figure_recommendation(text: &str) -> (String, String, String) {
        // Look for "Recommended Figure:" section
        if let Some(start) = text.find("**Recommended Figure:**") {
            if let Some(reason_start) = text[start..].find("**Reason:**") {
                let figure_line = &text[start..start + reason_start];
                let figure = figure_line
                    .replace("**Recommended Figure:**", "")
                    .trim()
                    .to_string();
                
                // Extract reason (up to 500 chars)
                let reason = if let Some(reason_content) = text[start + reason_start..].lines().nth(0) {
                    reason_content
                        .replace("**Reason:**", "")
                        .trim()
                        .to_string()
                } else {
                    String::new()
                };
                
                // Get article without recommendation section
                let article_clean = text[..start].trim().to_string();
                
                return (article_clean, figure, reason);
            }
        }
        
        // Fallback: no recommendation found
        (text.to_string(), String::new(), String::new())
    }
}

#[derive(Deserialize)]
struct SocialContentJson {
    linkedin_post: String,
    x_post: String,
    shorts_script: String,
}
