use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use std::collections::HashMap;

// Collector Configuration
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

// Writer Configuration (IA para gerar artigos)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterConfig {
    pub provider: String, // "deepseek", "openai", "anthropic"
    pub model: String,    // "deepseek-chat", "gpt-4", "claude-3", etc.
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub enabled: bool,
    pub use_compressor: Option<bool>,
}

// Education API Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationSourceConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub config: serde_json::Value, // params específicos (language, category, etc.)
}

// Social Media Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaConfig {
    pub id: String, // "youtube", "tiktok", "x", "instagram", "facebook"
    pub name: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub channel_id: Option<String>, // YouTube
    pub username: Option<String>,  // TikTok, Instagram, etc.
    pub config: serde_json::Value, // Configurações específicas de cada rede
}

// Complete Site Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub id: String,           // "airesearch", "scienceai", etc.
    pub name: String,          // "AI Research", "Science AI", etc.
    pub domain: Option<String>, // "airesearch.news"
    pub enabled: bool,
    pub collectors: Vec<CollectorConfig>,
    pub writer: WriterConfig,
    pub education_sources: Vec<EducationSourceConfig>,
    pub social_media: Vec<SocialMediaConfig>,
    pub collection_frequency_minutes: Option<u32>, // Frequency in minutes (60 = 1 hour, 120 = 2 hours, etc.)
    pub writing_style: Option<String>, // "scientific", "technical", "general", etc.
    // Prompt templates per channel
    pub prompt_article: Option<String>,
    pub prompt_social: Option<String>,
    pub prompt_blog: Option<String>,
    // Enable flags per channel
    pub prompt_article_enabled: Option<bool>,
    pub prompt_social_enabled: Option<bool>,
    pub prompt_blog_enabled: Option<bool>,
}

// Complete System Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub sites: HashMap<String, SiteConfig>,
    pub updated_at: String,
}

pub struct SiteConfigManager {
    config_path: std::path::PathBuf,
}

impl SiteConfigManager {
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// Load system configuration from file
    pub fn load(&self) -> Result<SystemConfig> {
        if !self.config_path.exists() {
            // Create default configuration
            let default_config = Self::create_default_config();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .context(format!("Failed to read config file: {}", self.config_path.display()))?;
        
        let config: SystemConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save system configuration to file
    pub fn save(&self, config: &SystemConfig) -> Result<()> {
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

    /// Get configuration for a specific site
    pub fn get_site_config(&self, site_id: &str) -> Result<Option<SiteConfig>> {
        let config = self.load()?;
        Ok(config.sites.get(site_id).cloned())
    }

    /// Update configuration for a specific site
    pub fn update_site_config(&self, site_id: &str, site_config: SiteConfig) -> Result<()> {
        let mut config = self.load()?;
        config.sites.insert(site_id.to_string(), site_config);
        self.save(&config)
    }

    /// Get all sites
    pub fn get_all_sites(&self) -> Result<Vec<SiteConfig>> {
        let config = self.load()?;
        Ok(config.sites.values().cloned().collect())
    }

    /// Update writer configuration for a site
    #[allow(dead_code)]
    pub fn update_writer_config(&self, site_id: &str, writer: WriterConfig) -> Result<()> {
        let mut config = self.load()?;
        if let Some(site) = config.sites.get_mut(site_id) {
            site.writer = writer;
        } else {
            anyhow::bail!("Site not found: {}", site_id);
        }
        self.save(&config)
    }

    /// Update collector status for a site
    pub fn update_collector_status(&self, site_id: &str, collector_id: &str, enabled: bool) -> Result<()> {
        let mut config = self.load()?;
        if let Some(site) = config.sites.get_mut(site_id) {
            if let Some(collector) = site.collectors.iter_mut()
                .find(|c| c.id == collector_id) {
                collector.enabled = enabled;
            } else {
                anyhow::bail!("Collector not found: {}", collector_id);
            }
        } else {
            anyhow::bail!("Site not found: {}", site_id);
        }
        self.save(&config)
    }

    /// Update social media status for a site
    pub fn update_social_status(&self, site_id: &str, social_id: &str, enabled: bool) -> Result<()> {
        let mut config = self.load()?;
        if let Some(site) = config.sites.get_mut(site_id) {
            if let Some(social) = site.social_media.iter_mut()
                .find(|s| s.id == social_id) {
                social.enabled = enabled;
            } else {
                anyhow::bail!("Social media not found: {}", social_id);
            }
        } else {
            anyhow::bail!("Site not found: {}", site_id);
        }
        self.save(&config)
    }

    /// Create default configuration
    fn create_default_config() -> SystemConfig {
        let mut sites = HashMap::new();

        // Default AIResearch site
        let airesearch = SiteConfig {
            id: "airesearch".to_string(),
            name: "AI Research".to_string(),
            domain: Some("airesearch.news".to_string()),
            enabled: true,
            collectors: vec![
                CollectorConfig {
                    id: "arxiv".to_string(),
                    name: "arXiv".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: None,
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
                    collector_type: None,
                    feed_url: None,
                    base_url: None,
                    selectors: None,
                    config: serde_json::json!({
                        "filter_technology_only": true,
                        "search_terms": [
                            "artificial intelligence",
                            "machine learning",
                            "deep learning",
                            "neural network",
                            "computer vision",
                            "natural language processing",
                            "data science",
                            "computer science",
                            "software engineering",
                            "programming",
                            "algorithm",
                            "transformer",
                            "reinforcement learning"
                        ],
                        "time_range_days": 30,
                    }),
                },
                CollectorConfig {
                    id: "semantic_scholar".to_string(),
                    name: "Semantic Scholar".to_string(),
                    enabled: false,
                    api_key: None,
                    collector_type: None,
                    feed_url: None,
                    base_url: None,
                    selectors: None,
                    config: serde_json::json!({
                        "filter_technology_only": true,
                        "fields_of_study": [
                            "Computer Science",
                            "Artificial Intelligence",
                            "Machine Learning",
                            "Computer Vision",
                            "Natural Language Processing",
                            "Data Science",
                            "Software Engineering"
                        ],
                        "query": "computer science artificial intelligence machine learning",
                        "max_results": 10,
                    }),
                },
            ],
            writer: WriterConfig {
                provider: "deepseek".to_string(),
                model: "deepseek-chat".to_string(),
                api_key: None,
                base_url: Some("https://api.deepseek.com".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(4000),
                enabled: true,
                use_compressor: Some(false),
            },
            education_sources: vec![
                EducationSourceConfig {
                    id: "edx".to_string(),
                    name: "edX".to_string(),
                    enabled: false,
                    api_key: None,
                    config: serde_json::json!({
                        "language": "en",
                        "categories": ["computer-science", "artificial-intelligence", "data-science", "programming"],
                        "filter_technology_only": true,
                    }),
                },
                EducationSourceConfig {
                    id: "mit_ocw".to_string(),
                    name: "MIT OpenCourseWare".to_string(),
                    enabled: false,
                    api_key: None,
                    config: serde_json::json!({
                        "language": "en",
                        "departments": ["Electrical-Engineering-and-Computer-Science", "Computer-Science"],
                        "filter_technology_only": true,
                    }),
                },
                EducationSourceConfig {
                    id: "class_central".to_string(),
                    name: "Class Central".to_string(),
                    enabled: false,
                    api_key: None,
                    config: serde_json::json!({
                        "language": "en",
                        "categories": ["computer-science", "ai", "programming"],
                        "filter_technology_only": true,
                    }),
                },
            ],
            social_media: vec![
                SocialMediaConfig {
                    id: "youtube".to_string(),
                    name: "YouTube".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
                SocialMediaConfig {
                    id: "tiktok".to_string(),
                    name: "TikTok".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
                SocialMediaConfig {
                    id: "x".to_string(),
                    name: "X (Twitter)".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
                SocialMediaConfig {
                    id: "instagram".to_string(),
                    name: "Instagram".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
                SocialMediaConfig {
                    id: "facebook".to_string(),
                    name: "Facebook".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
            ],
            collection_frequency_minutes: Some(60), // Default: 60 minutes (1 hour)
            writing_style: Some("scientific".to_string()), // Default: scientific
            prompt_article: Some("You are an expert writer. Generate a concise, factual summary of the article focusing on key findings and significance.".to_string()),
            prompt_social: Some("Create a short, engaging social post highlighting the key insight and why it matters. Include 3-5 relevant hashtags.".to_string()),
            prompt_blog: Some("Write a blog-style article with context, methodology, results, and implications. Use headings and keep a professional tone.".to_string()),
            prompt_article_enabled: Some(true),
            prompt_social_enabled: Some(false),
            prompt_blog_enabled: Some(false),
        };

        // Default ScienceAI site (future)
        let scienceai = SiteConfig {
            id: "scienceai".to_string(),
            name: "Science AI".to_string(),
            domain: Some("scienceai.news".to_string()),
            enabled: false, // Disabled by default
            collectors: vec![
                CollectorConfig {
                    id: "arxiv".to_string(),
                    name: "arXiv".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: None,
                    feed_url: None,
                    base_url: None,
                    selectors: None,
                    config: serde_json::json!({
                        "category": "cs.AI",
                        "max_results": 10,
                    }),
                },
            ],
            writer: WriterConfig {
                provider: "deepseek".to_string(),
                model: "deepseek-chat".to_string(),
                api_key: None,
                base_url: Some("https://api.deepseek.com".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(4000),
                enabled: true,
                use_compressor: Some(false),
            },
            education_sources: vec![],
            social_media: vec![
                SocialMediaConfig {
                    id: "youtube".to_string(),
                    name: "YouTube".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
                SocialMediaConfig {
                    id: "x".to_string(),
                    name: "X (Twitter)".to_string(),
                    enabled: false,
                    api_key: None,
                    api_secret: None,
                    access_token: None,
                    refresh_token: None,
                    channel_id: None,
                    username: None,
                    config: serde_json::json!({}),
                },
            ],
            collection_frequency_minutes: Some(120), // Default: 120 minutes (2 hours)
            writing_style: Some("technical".to_string()), // Default: technical
            prompt_article: Some("Write a technical summary emphasizing methodology, results, and implications for practitioners.".to_string()),
            prompt_social: Some("Create a short social post summarizing the key takeaway for a tech audience with hashtags.".to_string()),
            prompt_blog: Some("Compose a deep-dive blog post with sections for background, approach, results, limitations, and future work.".to_string()),
            prompt_article_enabled: Some(true),
            prompt_social_enabled: Some(false),
            prompt_blog_enabled: Some(false),
        };

        sites.insert("airesearch".to_string(), airesearch);
        sites.insert("scienceai".to_string(), scienceai);

        SystemConfig {
            sites,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

