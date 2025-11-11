use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Compact JSON formatting: put small arrays and objects on one line
#[allow(dead_code)]
fn compact_json_formatting(json: &str) -> String {
    let lines: Vec<&str> = json.lines().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        let indent_str = &line[..indent];

        // Skip empty lines
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Compact arrays that span multiple lines but are small
        if trimmed.starts_with('[') && !trimmed.ends_with(']') {
            let mut array_lines = vec![trimmed.to_string()];
            let mut depth = 1;
            let mut j = i + 1;

            // Collect all lines until array closes
            while j < lines.len() && depth > 0 {
                let next_line = lines[j].trim();
                for ch in next_line.chars() {
                    if ch == '[' {
                        depth += 1;
                    }
                    if ch == ']' {
                        depth -= 1;
                    }
                }
                array_lines.push(next_line.to_string());
                if depth == 0 {
                    j += 1;
                    break;
                }
                j += 1;
            }

            // Compact array if it's small enough (2-5 lines, < 120 chars total)
            let total_chars: usize = array_lines.iter().map(|s| s.len()).sum();
            if array_lines.len() >= 2 && array_lines.len() <= 5 && total_chars < 120 {
                let compact = array_lines.join(" ").replace(" ]", "]").replace("[ ", "[");
                result.push_str(&format!("{}{}\n", indent_str, compact));
                i = j;
                continue;
            }
        }
        // Compact objects that span multiple lines but are small
        else if trimmed.starts_with('{') && !trimmed.ends_with('}') {
            let mut obj_lines = vec![trimmed.to_string()];
            let mut depth = 1;
            let mut j = i + 1;

            // Collect all lines until object closes
            while j < lines.len() && depth > 0 {
                let next_line = lines[j].trim();
                for ch in next_line.chars() {
                    if ch == '{' {
                        depth += 1;
                    }
                    if ch == '}' {
                        depth -= 1;
                    }
                }
                obj_lines.push(next_line.to_string());
                if depth == 0 {
                    j += 1;
                    break;
                }
                j += 1;
            }

            // Compact object if it's small enough (2-10 lines, < 150 chars total)
            let total_chars: usize = obj_lines.iter().map(|s| s.len()).sum();
            if obj_lines.len() >= 2 && obj_lines.len() <= 10 && total_chars < 150 {
                let compact = obj_lines.join(" ").replace(" }", "}").replace("{ ", "{");
                result.push_str(&format!("{}{}\n", indent_str, compact));
                i = j;
                continue;
            }
        }
        // Already compact small objects like {"max_results": 5}
        else if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() < 100 {
            result.push_str(&format!("{}\n", line));
        }
        // Regular line
        else {
            result.push_str(&format!("{}\n", line));
        }

        i += 1;
    }

    result
}

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
    /// List of site IDs where content from this collector should be sent
    /// This is separate from enabled - it indicates the target destinations for generated content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destinations: Option<Vec<String>>,
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
    pub username: Option<String>,   // TikTok, Instagram, etc.
    pub config: serde_json::Value,  // Configurações específicas de cada rede
}

// Complete Site Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub id: String,             // "airesearch", "scienceai", etc.
    pub name: String,           // "AI Research", "Science AI", etc.
    pub domain: Option<String>, // "airesearch.news"
    pub enabled: bool,
    pub collectors: Vec<CollectorConfig>,
    pub writer: WriterConfig,
    pub education_sources: Vec<EducationSourceConfig>,
    pub social_media: Vec<SocialMediaConfig>,
    pub collection_frequency_minutes: Option<u32>, // Frequency in minutes (60 = 1 hour, 120 = 2 hours, etc.)
    pub writing_style: Option<String>,             // "scientific", "technical", "general", etc.
    // Prompt templates per channel
    pub prompt_article: Option<String>,
    pub prompt_social: Option<String>,
    pub prompt_blog: Option<String>,
    // Enable flags per channel
    pub prompt_article_enabled: Option<bool>,
    pub prompt_social_enabled: Option<bool>,
    pub prompt_blog_enabled: Option<bool>,
    // Temperature per prompt channel
    pub temperature_article: Option<f64>,
    pub temperature_social: Option<f64>,
    pub temperature_blog: Option<f64>,
}

// System Paths Configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathsConfig {
    /// Base directory for the project (default: current directory or env var)
    #[serde(default = "default_base_dir")]
    pub base_dir: String,
    /// Directory for downloaded files (relative to base_dir)
    #[serde(default = "default_downloads_dir")]
    pub downloads_dir: String,
    /// Directory for output files (relative to base_dir)
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    /// Path to articles registry file (relative to base_dir)
    #[serde(default = "default_registry_file")]
    pub registry_file: String,
}

fn default_base_dir() -> String {
    std::env::var("NEWS_BASE_DIR").unwrap_or_else(|_| {
        // Try to detect from current working directory
        std::env::current_dir()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string())
    })
}

fn default_downloads_dir() -> String {
    "downloads".to_string()
}

fn default_output_dir() -> String {
    "output".to_string()
}

fn default_registry_file() -> String {
    "articles_registry.json".to_string()
}

// Complete System Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    #[serde(default)]
    pub paths: PathsConfig,
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

        let content = fs::read_to_string(&self.config_path).context(format!(
            "Failed to read config file: {}",
            self.config_path.display()
        ))?;

        let config: SystemConfig =
            serde_json::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save system configuration to file
    pub fn save(&self, config: &SystemConfig) -> Result<()> {
        let mut config_to_save = config.clone();
        config_to_save.updated_at = chrono::Utc::now().to_rfc3339();

        // Serialize to pretty JSON (compact formatting temporarily disabled to ensure valid JSON)
        let content =
            serde_json::to_string_pretty(&config_to_save).context("Failed to serialize config")?;

        // TODO: Re-enable compact formatting after fixing JSON validity issues
        // let content = compact_json_formatting(&pretty_json);

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create config directory: {}",
                parent.display()
            ))?;
        }

        fs::write(&self.config_path, content).context(format!(
            "Failed to write config file: {}",
            self.config_path.display()
        ))?;

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

    /// Get paths configuration
    pub fn get_paths(&self) -> Result<PathsConfig> {
        let config = self.load()?;
        Ok(config.paths)
    }

    /// Get first enabled site with writer enabled
    #[allow(dead_code)]
    pub fn get_default_site_id(&self) -> Result<Option<String>> {
        let config = self.load()?;
        for site in config.sites.values() {
            if site.enabled && site.writer.enabled {
                return Ok(Some(site.id.clone()));
            }
        }
        Ok(None)
    }

    /// Get writer API key for a specific site
    #[allow(dead_code)]
    pub fn get_writer_api_key(&self, site_id: &str) -> Result<Option<String>> {
        let config = self.load()?;
        if let Some(site) = config.sites.get(site_id) {
            Ok(site.writer.api_key.clone())
        } else {
            Ok(None)
        }
    }

    /// Get first enabled site's writer API key
    #[allow(dead_code)]
    pub fn get_default_writer_api_key(&self) -> Result<Option<String>> {
        if let Some(site_id) = self.get_default_site_id()? {
            self.get_writer_api_key(&site_id)
        } else {
            Ok(None)
        }
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
    pub fn update_collector_status(
        &self,
        site_id: &str,
        collector_id: &str,
        enabled: bool,
    ) -> Result<()> {
        let mut config = self.load()?;
        if let Some(site) = config.sites.get_mut(site_id) {
            if let Some(collector) = site.collectors.iter_mut().find(|c| c.id == collector_id) {
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
    pub fn update_social_status(
        &self,
        site_id: &str,
        social_id: &str,
        enabled: bool,
    ) -> Result<()> {
        let mut config = self.load()?;
        if let Some(site) = config.sites.get_mut(site_id) {
            if let Some(social) = site.social_media.iter_mut().find(|s| s.id == social_id) {
                social.enabled = enabled;
            } else {
                anyhow::bail!("Social media not found: {}", social_id);
            }
        } else {
            anyhow::bail!("Site not found: {}", site_id);
        }
        self.save(&config)
    }

    /// Get the default article prompt template for a specific site
    pub fn get_default_article_prompt_template(site_id: &str, site_name: &str) -> String {
        // Get site-specific context (matching logic from prompts.rs)
        let site_context = match site_id.to_lowercase().as_str() {
            "airesearch" => r#"AIResearch is a cutting-edge AI news platform focusing on:
- Latest breakthroughs in artificial intelligence research
- Practical applications of ML/deep learning
- Industry news and expert analysis
- **News-style journalism**: Complex topics explained for general audience
- **Simple titles**: Focus on WHAT the discovery means, not HOW it works technically
- **Accessibility first**: Make readers understand WHY it matters
- **Clear explanations**: Use analogies and real-world comparisons
- Users who want technical details can read the original paper
- Emphasis on accuracy and scientific rigor WITH simple language"#
                .to_string(),

            "scienceai" => r#"Science AI is a cutting-edge AI news platform focusing on:
- Latest breakthroughs in artificial intelligence research
- Practical applications of ML/deep learning
- Industry news and expert analysis
- **News-style journalism**: Complex topics explained for general audience
- **Simple titles**: Focus on WHAT the discovery means, not HOW it works technically
- **Accessibility first**: Make readers understand WHY it matters
- **Clear explanations**: Use analogies and real-world comparisons
- Users who want technical details can read the original paper
- Emphasis on accuracy and scientific rigor WITH simple language"#
                .to_string(),

            _ => r#"General scientific publication:
- Clear, accurate, accessible communication
- Emphasis on evidence-based reporting
- Professional academic tone
- Broad scientific audience"#
                .to_string(),
        };

        format!(
            r#"CRITICAL INSTRUCTIONS (READ FIRST):
1. You are writing for {} in Nature/Science magazine editorial style (News & Views, Perspectives sections)
2. **NEVER FABRICATE**: Do not invent citations, references, authors, studies, or data that are not explicitly in the paper below
3. **ONLY USE PAPER CONTENT**: Reference only what exists in the provided paper text
4. NO AI clichés: "delve", "revolutionize", "game-changer", "unlock", "harness", "dive into", "shed light on"
5. NO emojis, NO excessive dashes (—), NO ellipses (...)

---

TARGET PUBLICATION:
{}

---

WRITING STYLE (Nature/Science Editorial - Simplified for General Audience):
- **Opening:** Establish significance immediately (why non-technical readers should care)
- **Voice:** Active, direct, authoritative yet conversational and accessible
- **Structure:** Inverted pyramid - key findings first, details follow
- **Precision:** Reference specific figures, data points, methodology FROM THE PAPER
- **Clarity:** Define technical terms on first use AND use plain language alternatives
- **Flow:** Smooth transitions between concepts - explain as if to an intelligent layperson
- **Accessibility:** Every technical concept should have a simple analogy or real-world comparison
- **Purpose:** Make complex science understandable so readers grasp the importance, then they can read the paper for details

CRITICAL RULES:
- ✅ "The researchers found X (as shown in Figure 2)..."
- ✅ "This approach builds on the methods described in the paper..."
- ✅ "The data shows a 23% increase..." (if paper states this)
- ❌ "Previous work by Zhang et al. (Nature, 2023)..." (unless paper cites this)
- ❌ "Experts suggest..." (unless paper includes expert quotes)
- ❌ "This could lead to cures for cancer..." (unless paper discusses this)

WHAT TO AVOID (AI writing patterns):
- "In a groundbreaking study that could revolutionize..."
- "Scientists have unlocked the secrets of..."
- "This research sheds new light on..."
- "Paradigm-shifting", "game-changing" (unless genuinely warranted)
- Inventing related research not mentioned in paper
- Speculating beyond what paper's data supports

IMAGE CATEGORY SELECTION (REQUIRED - CRITICAL RULES):

You MUST select exactly 3 categories from THIS EXACT LIST ONLY:
ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound

CRITICAL CONSTRAINTS:
- ❌ DO NOT create new categories (like "biology", "physics", "medical", etc.)
- ❌ DO NOT use synonyms or variations
- ✅ ONLY use the 13 categories listed above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must be lowercase, matching the list exactly

SELECTION GUIDELINES:
- ai: Artificial intelligence, machine learning, AI research
- coding: Programming, software development, code
- crypto: Cryptocurrency, blockchain, digital currency
- database: Data storage, data management, servers
- ethics: Ethical considerations, societal impact
- games: Gaming, game development, interactive tech
- hardware: Physical computing, electronics, processors
- legal: Legal issues, regulations, compliance
- network: Networking, connectivity, communication
- robotics: Robots, automation, mechanical AI
- science: General scientific research, experiments
- security: Cybersecurity, privacy, protection
- sound: Audio technology, sound processing

EXAMPLES (USE THESE EXACT NAMES):
✓ For neural networks → ["ai", "science", "network"]
✓ For robotics → ["robotics", "ai", "hardware"]
✓ For cybersecurity → ["security", "network", "crypto"]
✓ For data analysis → ["database", "ai", "coding"]
✓ For chip research → ["hardware", "science", "ai"]

Include this as "image_categories" array in your JSON response.

REQUIRED ARTICLE STRUCTURE:
1. **Opening Hook** (2-3 sentences: significance and context from paper - explain WHY non-technical readers should care)
2. **Key Finding** (what researchers discovered - explain in plain language, avoid jargon)
3. **Methodology** (how they did it - simplified explanation focusing on the approach, not technical details)
4. **Results Analysis** (what data shows - reference figures from paper, use simple language)
5. **Context** (why it matters - real-world implications for regular readers)
6. **Limitations** (what remains unknown - from paper's limitations section)

TITLE REQUIREMENTS (CRITICAL):
- **CRITICAL**: The generated title MUST be DIFFERENT from the original title in the paper. NEVER use the same title as the original source.
- **News-focused**: Explain what the breakthrough means to everyday readers
- **Simple language**: Avoid technical jargon, write for general audience
- **Active voice**: Make it engaging and accessible
- **Specific**: Include what was achieved or discovered
- **Hook**: Capture attention like a news headline

BAD TITLES (Too technical):
❌ "Graph Neural Networks for Spatiotemporal Dynamics"
❌ "Gradient-Based Optimization in High-Dimensional Space"
❌ "Multi-Agent Path Planning Algorithm for AUV Coordination"

GOOD TITLES (News-focused):
✓ "AI Can Now Predict Complex Data Relationships Without Violating Privacy"
✓ "New Method Helps Scientists Share Data Securely Without Losing Accuracy"
✓ "Robots Navigate Ocean Currents Using Real-Time Weather Data"

SUBTITLE REQUIREMENTS (CRITICAL):
- **SEO-optimized**: A compelling summary optimized for search engines
- **Maximum 2 lines**: Keep it concise and impactful
- **Add tension**: Should create curiosity, consequence, or reason why this matters
- **Clear value proposition**: Explain the significance in simple terms
- **No technical jargon**: Write for general audience understanding

GOOD SUBTITLES:
✓ "A new AI method can generate fake data that captures real-world patterns so accurately that researchers can use it for sensitive analysis—without ever touching the original information."
✓ "Scientists discovered the universe might be two billion years younger than previously thought by using more precise measurements."
✓ "Cancer cells have a hidden escape route that researchers just identified, opening new doors for treatment."

EXAMPLES OF GOOD OPENING LINES (News style):
✓ "A new AI method can generate fake data that captures real-world patterns so accurately that researchers can use it for sensitive analysis—without ever touching the original information."
✓ "Scientists discovered the universe might be two billion years younger than previously thought by using more precise measurements."
✓ "Cancer cells have a hidden escape route that researchers just identified, opening new doors for treatment."

## PAPER TEXT (YOUR ONLY SOURCE):
{{paper_text}}

## DELIVERABLE:
Write a 500-800 word article in Nature/Science editorial style. Use ONLY information from the paper above.

Format:
# [Compelling, Specific Title - Nature/Science Style]

[Article body - based ONLY on paper content...]

CRITICAL JSON FORMAT - YOU MUST FOLLOW THIS EXACT STRUCTURE:
{{
  "title": "Your title here",
  "subtitle": "SEO-optimized subtitle (max 2 lines) - compelling summary that adds tension or explains significance",
  "article_text": "Full article body text here - all content in one string field",
  "image_categories": ["category1", "category2", "category3"]
}}

⚠️ IMPORTANT RULES:
- "subtitle" MUST be a STRING field at the root level
- "subtitle" MUST be SEO-optimized, maximum 2 lines, and create curiosity or explain significance
- "article_text" MUST be a STRING field at the root level (NOT nested in an "article" object)
- "article_text" MUST contain the complete article text in one string
- DO NOT create nested objects like {{"article": {{"opening_hook": "...", "key_finding": "..."}}}}
- All article content goes directly into the "article_text" string field
- Return ONLY valid JSON - no markdown, no extra formatting

TITLE REQUIREMENTS (CRITICAL):
- **CRITICAL**: The generated title MUST be DIFFERENT from the original title in the paper. NEVER use the same title as the original source.
- MAXIMUM 8 WORDS (short, punchy, viral)
- STRONG HOOK to make users WANT to click and read
- Create curiosity, tension, or surprise
- Make readers NEED to know more

BAD TITLES (too long, no hook):
❌ "A New Approach to Machine Learning Optimization in Deep Neural Networks"
❌ "Understanding the Fundamentals of Quantum Computing Applications"

GOOD TITLES (short, hooky, irresistible):
✓ "AI Agents Fall Short at Scientific Discovery"
✓ "Scientists Find Hidden Pattern in Neural Networks"
✓ "This AI Breakthrough May Be Wrong"  
"#,
            site_name, site_context
        )
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
                    destinations: None,
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
                    destinations: None,
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
                    destinations: None,
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
            prompt_article: Some(Self::get_default_article_prompt_template("airesearch", "AI Research")),
            prompt_social: Some("Create a short, engaging social post highlighting the key insight and why it matters. Include 3-5 relevant hashtags.".to_string()),
            prompt_blog: Some("Write a blog-style article with context, methodology, results, and implications. Use headings and keep a professional tone.".to_string()),
            prompt_article_enabled: Some(true),
            prompt_social_enabled: Some(false),
            prompt_blog_enabled: Some(false),
            temperature_article: Some(0.7),
            temperature_social: Some(0.8),
            temperature_blog: Some(0.7),
        };

        // Default ScienceAI site - News sources (RSS and HTML)
        let scienceai = SiteConfig {
            id: "scienceai".to_string(),
            name: "Science AI".to_string(),
            domain: Some("scienceai.news".to_string()),
            enabled: false, // Disabled by default
            collectors: vec![
                // Article collectors (disabled - only for AIResearch)
                CollectorConfig {
                    id: "arxiv".to_string(),
                    name: "arXiv".to_string(),
                    enabled: false,
                    api_key: None,
                    collector_type: None,
                    feed_url: None,
                    base_url: None,
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({
                        "category": "cs.AI",
                        "max_results": 10,
                    }),
                },
                // RSS News Collectors
                CollectorConfig {
                    id: "rss_openai".to_string(),
                    name: "OpenAI Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://openai.com/blog/rss.xml".to_string()),
                    base_url: Some("https://openai.com".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_google_ai".to_string(),
                    name: "Google AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://blog.research.google/feeds/posts/default".to_string()),
                    base_url: Some("https://blog.research.google".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_nvidia".to_string(),
                    name: "NVIDIA News RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://nvidianews.nvidia.com/rss/all-news.xml".to_string()),
                    base_url: Some("https://nvidianews.nvidia.com".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_alibaba_damo".to_string(),
                    name: "Alibaba DAMO RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://damo.alibaba.com/news/rss".to_string()),
                    base_url: Some("https://damo.alibaba.com".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                // HTML News Collectors
                CollectorConfig {
                    id: "html_anthropic".to_string(),
                    name: "Anthropic News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://www.anthropic.com/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article",
                        "content": ".content, article, main",
                        "link": "a[href*='/news/']",
                        "title": "h1, h2"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_meta_ai".to_string(),
                    name: "Meta AI Blog".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://ai.meta.com/blog/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .blog-post",
                        "content": ".content, article, main",
                        "link": "a[href*='/blog/']",
                        "title": "h1, h2"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_deepseek".to_string(),
                    name: "DeepSeek News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://deepseek.com/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item",
                        "content": ".content, article, main",
                        "link": "a[href*='/news/']",
                        "title": "h1, h2"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_alibaba_damo".to_string(),
                    name: "Alibaba Alizila News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://www.alizila.com/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "a[href*='alizila.com'], article, .post-item, .article-item, .news-item",
                        "content": ".content, .post-content, .article-content, .story-content, article, main, .article-body",
                        "link": "a",
                        "title": "h1, h2, h3, .title, .post-title, .article-title, .story-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_xai".to_string(),
                    name: "X.ai News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://x.ai/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .blog-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/news/'], a[href*='x.ai/news']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_mistral_ai".to_string(),
                    name: "Mistral AI News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://mistral.ai/news/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .blog-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/news/'], a[href*='mistral.ai/news']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_cohere".to_string(),
                    name: "Cohere AI Blog".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://txt.cohere.com/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .blog-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='cohere.com'], a[href*='txt.cohere.com']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_perplexity".to_string(),
                    name: "Perplexity AI Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://blog.perplexity.ai/feed".to_string()),
                    base_url: Some("https://blog.perplexity.ai/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_huggingface".to_string(),
                    name: "Hugging Face Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://huggingface.co/blog/feed.xml".to_string()),
                    base_url: Some("https://huggingface.co/blog".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_stability_ai".to_string(),
                    name: "Stability AI News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://stability.ai/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .blog-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/news'], a[href*='stability.ai/news']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_elevenlabs".to_string(),
                    name: "ElevenLabs Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://blog.elevenlabs.io/feed".to_string()),
                    base_url: Some("https://blog.elevenlabs.io/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_character_ai".to_string(),
                    name: "Character.AI".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://beta.character.ai/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .blog-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='character.ai']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_inflection_ai".to_string(),
                    name: "Inflection AI (Pi)".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://inflection.ai/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .blog-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='inflection.ai']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_microsoft_ai".to_string(),
                    name: "Microsoft AI Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://blogs.microsoft.com/ai/feed/".to_string()),
                    base_url: Some("https://blogs.microsoft.com/ai/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_ibm_research".to_string(),
                    name: "IBM Research AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://research.ibm.com/blog/feed".to_string()),
                    base_url: Some("https://research.ibm.com/blog".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_apple_ml".to_string(),
                    name: "Apple Machine Learning Journal".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://machinelearning.apple.com/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .research-item, .publication-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='machinelearning.apple.com']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_intel_ai".to_string(),
                    name: "Intel AI Blog".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://www.intel.com/content/www/us/en/artificial-intelligence/posts.html".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .blog-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/artificial-intelligence/posts']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_amd_ai".to_string(),
                    name: "AMD AI / Machine Learning".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://community.amd.com/t5/ai-and-ml/bg-p/ai-ml".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .discussion-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/ai-and-ml']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_salesforce_ai".to_string(),
                    name: "Salesforce AI Blog RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://www.salesforce.com/news/feed/".to_string()),
                    base_url: Some("https://www.salesforce.com/news/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_stanford_hai".to_string(),
                    name: "Stanford HAI News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://hai.stanford.edu/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='hai.stanford.edu/news']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_berkeley_ai".to_string(),
                    name: "Berkeley AI Research News".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://bair.berkeley.edu/news".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='bair.berkeley.edu/news']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_deepmind_blog".to_string(),
                    name: "DeepMind Blog".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://deepmind.google/discover/blog/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .post-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='deepmind.google/discover/blog']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_techcrunch_ai".to_string(),
                    name: "TechCrunch AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://techcrunch.com/tag/artificial-intelligence/feed/".to_string()),
                    base_url: Some("https://techcrunch.com/tag/artificial-intelligence/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_venturebeat_ai".to_string(),
                    name: "VentureBeat AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://venturebeat.com/category/ai/feed/".to_string()),
                    base_url: Some("https://venturebeat.com/category/ai/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_the_verge_ai".to_string(),
                    name: "The Verge AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://www.theverge.com/rss/group/ai/index.xml".to_string()),
                    base_url: Some("https://www.theverge.com/tech/ai".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_wired_ai".to_string(),
                    name: "Wired AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://www.wired.com/feed/category/science/ai/latest/rss".to_string()),
                    base_url: Some("https://www.wired.com/category/science/ai/".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_mit_tech_review_ai".to_string(),
                    name: "MIT Technology Review AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://news.mit.edu/topic/artificial-intelligence-rss.xml".to_string()),
                    base_url: Some("https://news.mit.edu/topic/artificial-intelligence".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_nature_ai".to_string(),
                    name: "Nature AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://www.nature.com/subjects/artificial-intelligence.rss".to_string()),
                    base_url: Some("https://www.nature.com/subjects/artificial-intelligence".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "rss_science_ai".to_string(),
                    name: "Science AI RSS".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("rss".to_string()),
                    feed_url: Some("https://www.science.org/topic/artificial-intelligence/rss".to_string()),
                    base_url: Some("https://www.science.org/topic/artificial-intelligence".to_string()),
                    selectors: None,
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_hive_hub".to_string(),
                    name: "Hive Hub".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://hive-hub.com".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .post-item, .blog-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='hive-hub.com']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
                },
                CollectorConfig {
                    id: "html_menlo_ventures".to_string(),
                    name: "Menlo Ventures AI".to_string(),
                    enabled: true,
                    api_key: None,
                    collector_type: Some("html".to_string()),
                    feed_url: None,
                    base_url: Some("https://menlovc.com/focus-areas/ai/".to_string()),
                    selectors: Some(serde_json::json!({
                        "article": "article, .post, .news-item, .post-item, .blog-item, .perspective-item",
                        "content": ".content, .post-content, .article-content, article, main, .article-body",
                        "link": "a[href*='/focus-areas/ai'], a[href*='menlovc.com']",
                        "title": "h1, h2, h3, .title, .post-title, .article-title, .perspective-title"
                    })),
                    destinations: None,
                    config: serde_json::json!({"max_results": 5}),
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
            prompt_article: Some(Self::get_default_article_prompt_template("scienceai", "Science AI")),
            prompt_social: Some("Create a short social post summarizing the key takeaway for a tech audience with hashtags.".to_string()),
            prompt_blog: Some("Compose a deep-dive blog post with sections for background, approach, results, limitations, and future work.".to_string()),
            prompt_article_enabled: Some(true),
            prompt_social_enabled: Some(false),
            prompt_blog_enabled: Some(false),
            temperature_article: Some(0.7),
            temperature_social: Some(0.8),
            temperature_blog: Some(0.7),
        };

        sites.insert("airesearch".to_string(), airesearch);
        sites.insert("scienceai".to_string(), scienceai);

        SystemConfig {
            paths: PathsConfig {
                base_dir: std::env::var("NEWS_BASE_DIR").unwrap_or_else(|_| ".".to_string()),
                downloads_dir: "downloads".to_string(),
                output_dir: "output".to_string(),
                registry_file: "articles_registry.json".to_string(),
            },
            sites,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
