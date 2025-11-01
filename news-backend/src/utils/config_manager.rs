use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    /// Type of collector: "api", "rss", "html"
    #[serde(default)]
    pub collector_type: Option<String>,
    /// RSS feed URL (for RSS collectors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_url: Option<String>,
    /// Base URL for HTML scraping (for HTML collectors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// CSS selectors for HTML scraping (for HTML collectors)
    /// Format: {"article": "article", "title": "h1", "content": ".content", ...}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selectors: Option<serde_json::Value>,
    /// Additional configuration (for backwards compatibility)
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorsConfig {
    pub collectors: Vec<CollectorConfig>,
    pub updated_at: String,
}

pub struct ConfigManager {
    config_path: std::path::PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// Load collectors configuration from file
    pub fn load(&self) -> Result<CollectorsConfig> {
        if !self.config_path.exists() {
            // Create default configuration
            let default_config = CollectorsConfig {
                collectors: vec![
                    CollectorConfig {
                        id: "arxiv".to_string(),
                        name: "arXiv".to_string(),
                        enabled: true,
                        api_key: None,
                        collector_type: Some("api".to_string()),
                        feed_url: None,
                        base_url: None,
                        selectors: None,
                        config: serde_json::json!({
                            "category": "cs.AI",
                            "max_results": 10,
                        }),
                    },
                    CollectorConfig {
                        id: "pmc".to_string(),
                        name: "PubMed Central".to_string(),
                        enabled: false,
                        api_key: None,
                        collector_type: Some("api".to_string()),
                        feed_url: None,
                        base_url: None,
                        selectors: None,
                        config: serde_json::json!({}),
                    },
                    CollectorConfig {
                        id: "semantic_scholar".to_string(),
                        name: "Semantic Scholar".to_string(),
                        enabled: false,
                        api_key: None,
                        collector_type: Some("api".to_string()),
                        feed_url: None,
                        base_url: None,
                        selectors: None,
                        config: serde_json::json!({}),
                    },
                    // Example RSS collector
                    CollectorConfig {
                        id: "openai_rss".to_string(),
                        name: "OpenAI Blog RSS".to_string(),
                        enabled: false,
                        api_key: None,
                        collector_type: Some("rss".to_string()),
                        feed_url: Some("https://openai.com/blog/rss.xml".to_string()),
                        base_url: Some("https://openai.com".to_string()),
                        selectors: None,
                        config: serde_json::json!({
                            "max_results": 10,
                        }),
                    },
                    // Example HTML collector
                    CollectorConfig {
                        id: "meta_ai_html".to_string(),
                        name: "Meta AI Blog HTML".to_string(),
                        enabled: false,
                        api_key: None,
                        collector_type: Some("html".to_string()),
                        feed_url: None,
                        base_url: Some("https://ai.meta.com/blog/".to_string()),
                        selectors: Some(serde_json::json!({
                            "article": "article",
                            "title": "h2 a",
                            "content": "article",
                        })),
                        config: serde_json::json!({
                            "max_results": 10,
                        }),
                    },
                ],
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            
            // Save default config
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .context(format!("Failed to read config file: {}", self.config_path.display()))?;
        
        let config: CollectorsConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save collectors configuration to file
    pub fn save(&self, config: &CollectorsConfig) -> Result<()> {
        let mut config_to_save = config.clone();
        config_to_save.updated_at = chrono::Utc::now().to_rfc3339();
        
        let content = serde_json::to_string_pretty(&config_to_save)
            .context("Failed to serialize config")?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        fs::write(&self.config_path, content)
            .context(format!("Failed to write config file: {}", self.config_path.display()))?;
        
        Ok(())
    }

    /// Get enabled collectors
    pub fn get_enabled_collectors(&self) -> Result<Vec<CollectorConfig>> {
        let config = self.load()?;
        Ok(config.collectors.into_iter()
            .filter(|c| c.enabled)
            .collect())
    }

    /// Update collector status (enabled/disabled)
    pub fn update_collector_status(&self, collector_id: &str, enabled: bool) -> Result<()> {
        let mut config = self.load()?;
        
        if let Some(collector) = config.collectors.iter_mut()
            .find(|c| c.id == collector_id) {
            collector.enabled = enabled;
        } else {
            anyhow::bail!("Collector not found: {}", collector_id);
        }
        
        self.save(&config)
    }

    /// Update collector configuration
    pub fn update_collector_config(&self, collector_id: &str, updates: CollectorConfig) -> Result<()> {
        let mut config = self.load()?;
        
        if let Some(collector) = config.collectors.iter_mut()
            .find(|c| c.id == collector_id) {
            collector.enabled = updates.enabled;
            collector.api_key = updates.api_key;
            collector.collector_type = updates.collector_type;
            collector.feed_url = updates.feed_url;
            collector.base_url = updates.base_url;
            collector.selectors = updates.selectors;
            collector.config = updates.config;
        } else {
            config.collectors.push(updates);
        }
        
        self.save(&config)
    }
}

















