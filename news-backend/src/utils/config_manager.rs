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
    #[allow(dead_code)]
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

    /// Sync collectors_config.json from system_config.json
    /// This reads all enabled collectors from all enabled sites in system_config.json
    /// and generates/updates collectors_config.json
    pub fn sync_from_system_config(system_config_path: &Path, collectors_config_path: &Path) -> Result<()> {
        use crate::utils::site_config_manager::SiteConfigManager;
        use std::collections::HashMap;

        eprintln!("🔄 [SYNC] Syncing collectors_config.json from system_config.json...");
        eprintln!("🔍 [DEBUG] system_config_path: {} (exists: {})", system_config_path.display(), system_config_path.exists());
        eprintln!("🔍 [DEBUG] collectors_config_path: {} (exists: {})", collectors_config_path.display(), collectors_config_path.exists());
        
        // Verify system_config.json exists
        if !system_config_path.exists() {
            return Err(anyhow::anyhow!(
                "system_config.json not found at: {}. Please ensure the file exists.",
                system_config_path.display()
            ));
        }
        
        // Load system_config.json
        let system_manager = SiteConfigManager::new(system_config_path);
        let system_config = system_manager.load()
            .context(format!("Failed to load system_config.json from: {}", system_config_path.display()))?;
        
        eprintln!("🔄 [SYNC] Loaded system_config.json with {} sites", system_config.sites.len());
        
        // Extract all enabled collectors from all enabled sites
        let mut collectors_map: HashMap<String, CollectorConfig> = HashMap::new();
        
        for (site_id, site) in &system_config.sites {
            if site.enabled {
                eprintln!("🔄 [SYNC] Processing site: {} ({} collectors)", site_id, site.collectors.len());
                for site_collector in &site.collectors {
                    eprintln!("🔄 [SYNC]   → Collector: {} (enabled: {}, type: {:?})", site_collector.id, site_collector.enabled, site_collector.collector_type);
                    let collector_id = site_collector.id.clone();
                    
                    // Convert from site_config_manager::CollectorConfig to config_manager::CollectorConfig
                    // Note: config_manager::CollectorConfig doesn't have destinations field
                    let config_collector = CollectorConfig {
                        id: site_collector.id.clone(),
                        name: site_collector.name.clone(),
                        enabled: site_collector.enabled,
                        api_key: site_collector.api_key.clone(),
                        collector_type: site_collector.collector_type.clone(),
                        feed_url: site_collector.feed_url.clone(),
                        base_url: site_collector.base_url.clone(),
                        selectors: site_collector.selectors.clone(),
                        config: site_collector.config.clone(),
                    };
                    
                    // Deduplicate by ID (if same collector appears in multiple sites, keep first found)
                    if !collectors_map.contains_key(&collector_id) {
                        collectors_map.insert(collector_id.clone(), config_collector);
                        eprintln!("🔄 [SYNC]     ✅ Added collector: {}", collector_id);
                    } else {
                        // If already exists, but new one is enabled and old one is not, update status
                        if let Some(existing) = collectors_map.get_mut(&collector_id) {
                            if site_collector.enabled && !existing.enabled {
                                existing.enabled = true;
                                eprintln!("🔄 [SYNC] Updated collector '{}' status to enabled", collector_id);
                            }
                        }
                    }
                }
            } else {
                eprintln!("🔄 [SYNC] Skipping disabled site: {}", site_id);
            }
        }
        
        // Convert HashMap to Vec
        let all_collectors: Vec<CollectorConfig> = collectors_map.into_values().collect();
        
        eprintln!("🔄 [SYNC] Extracted {} collectors (deduplicated)", all_collectors.len());
        eprintln!("🔄 [SYNC] Enabled collectors: {}", all_collectors.iter().filter(|c| c.enabled).count());
        
        // Create CollectorsConfig
        let collectors_config = CollectorsConfig {
            collectors: all_collectors,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Save to collectors_config.json
        let collectors_manager = ConfigManager::new(collectors_config_path);
        collectors_manager.save(&collectors_config)
            .context(format!("Failed to save collectors_config.json: {}", collectors_config_path.display()))?;
        
        eprintln!("✅ [SYNC] Successfully synced collectors_config.json from system_config.json");
        
        Ok(())
    }
}

















