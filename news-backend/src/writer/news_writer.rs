// News Writer Module
// Handles generating news articles from collected news JSON using DeepSeek API
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use crate::models::raw_document::ArticleMetadata;
use crate::utils::article_registry::RegistryManager;
use crate::utils::site_config_manager::SiteConfigManager;
use super::deepseek_client::{DeepSeekClient, ArticleResponse, SocialResponse};
use super::file_writer::{save_article, save_title, save_subtitle, save_linkedin, save_x, save_shorts_script, save_image_categories, save_source};
use super::prompt_compressor::PromptCompressor;
use serde_json;

pub struct NewsWriterService {
    output_base: PathBuf,
    registry: RegistryManager,
}

#[derive(Debug, Clone)]
pub struct NewsWriterResult {
    pub output_dir: PathBuf,
    pub site_id: String,
    pub site_name: String,
}

#[derive(Debug, Default)]
pub struct CleanupStats {
    pub verified: usize,
    pub updated: usize,
    pub removed: usize,
}

impl NewsWriterService {
    /// Creates a new NewsWriterService
    pub fn new(
        output_base: PathBuf,
        registry_path: PathBuf,
    ) -> Result<Self> {
        let registry = RegistryManager::new(&registry_path)
            .context("Failed to create registry manager")?;

        Ok(Self {
            output_base,
            registry,
        })
    }

    /// Processes a news article JSON file and generates content for configured sites
    pub async fn process_news_article(
        &self,
        article_json_path: &Path,
    ) -> Result<Vec<NewsWriterResult>> {
        // Load article from JSON
        let article_content = tokio::fs::read_to_string(article_json_path).await
            .context("Failed to read article JSON file")?;
        
        let article: ArticleMetadata = serde_json::from_str(&article_content)
            .context("Failed to parse article JSON")?;

        // Check which sites should receive this article (from registry destinations)
        let metadata = self.registry.get_metadata(&article.id);
        let destinations = metadata
            .as_ref()
            .and_then(|m| m.destinations.as_ref())
            .cloned()
            .unwrap_or_default();

        if destinations.is_empty() {
            return Err(anyhow::anyhow!("No destinations configured for article {}", article.id));
        }

        // Load site configurations
        let config_path = Path::new("system_config.json");
        let config_manager = SiteConfigManager::new(config_path);
        
        let mut results = Vec::new();

        // Process article for each destination site
        for site_id in destinations {
            match self.process_article_for_site(&article, &site_id, &config_manager).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to process article for site {}: {}", site_id, e);
                    // Continue with other sites even if one fails
                }
            }
        }

        Ok(results)
    }

    /// Processes an article for a specific site
    async fn process_article_for_site(
        &self,
        article: &ArticleMetadata,
        site_id: &str,
        config_manager: &SiteConfigManager,
    ) -> Result<NewsWriterResult> {
        // Get site configuration
        let site_config = config_manager.get_site_config(site_id)?
            .ok_or_else(|| anyhow::anyhow!("Site {} not found in configuration", site_id))?;

        // Check if blog writer is enabled for this site
        if !site_config.writer.enabled {
            return Err(anyhow::anyhow!("Writer is disabled for site {}", site_id));
        }

        // Get blog prompt (use custom if enabled, otherwise use default)
        let blog_prompt = if site_config.prompt_blog_enabled.unwrap_or(false) {
            site_config.prompt_blog.clone()
                .unwrap_or_else(|| Self::default_blog_prompt())
        } else {
            Self::default_blog_prompt()
        };

        // Get API configuration from site config
        let api_key = site_config.writer.api_key.clone()
            .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
            .context("API key not found in config or environment")?;
        
        let base_url = site_config.writer.base_url.clone()
            .unwrap_or_else(|| std::env::var("DEEPSEEK_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string()));
        
        let model = site_config.writer.model.clone();

        // Build prompt with article JSON
        let article_json = serde_json::to_string_pretty(article)?;
        let full_prompt = format!("{}\n\n## ARTICLE JSON:\n{}", blog_prompt, article_json);

        println!("  📝 Generating content for site: {} ({})", site_id, site_config.name);
        println!("     Using custom blog prompt: {}", site_config.prompt_blog_enabled.unwrap_or(false));

        // Compress prompt if enabled
        let final_prompt = if site_config.writer.use_compressor.unwrap_or(false) {
            println!("     Compressing prompt...");
            let compressor = PromptCompressor::new()
                .context("Failed to initialize prompt compressor")?;
            
            let compressed = compressor.compress(&full_prompt)
                .context("Failed to compress prompt")?;
            
            println!("     ✅ Prompt compressed: {} → {} tokens ({:.1}% reduction)", 
                compressed.original_tokens, 
                compressed.compressed_tokens,
                compressed.compression_ratio * 100.0);
            
            compressed.compressed_text
        } else {
            full_prompt
        };

        // Create DeepSeek client for this site
        let deepseek_client = DeepSeekClient::new(api_key, base_url, model);

        // Generate article content (includes social media content in response)
        let article_response = deepseek_client.generate_article(&final_prompt).await
            .context("Failed to generate article content")?;

        // Create output directory
        let site_output_dir = Self::get_site_output_dir(&site_id);
        let article_output_dir = site_output_dir.join(&article.id);
        tokio::fs::create_dir_all(&article_output_dir).await
            .context("Failed to create article output directory")?;

        // Detect article source/category
        let source_category = Self::detect_source_category(&article.url, &article.title);

        // Save all content to separate files
        save_title(&article_output_dir, &article_response.title).await?;
        save_subtitle(&article_output_dir, &article_response.subtitle).await?;
        save_article(&article_output_dir, &article_response.article_text).await?;
        save_image_categories(&article_output_dir, &article_response.image_categories).await?;
        save_source(&article_output_dir, &source_category).await?;
        save_x(&article_output_dir, &article_response.x_post).await?;
        save_linkedin(&article_output_dir, &article_response.linkedin_post).await?;
        save_shorts_script(&article_output_dir, &article_response.shorts_script).await?;

        // Update registry to mark as published
        self.registry.register_published(&article.id, article_output_dir.clone())
            .context("Failed to register article as published")?;

        println!("  ✅ Content saved → {}", article_output_dir.display());

        Ok(NewsWriterResult {
            output_dir: article_output_dir,
            site_id: site_id.to_string(),
            site_name: site_config.name.clone(),
        })
    }

    /// Gets output directory for a site
    fn get_site_output_dir(site_id: &str) -> PathBuf {
        let site_name = match site_id.to_lowercase().as_str() {
            "airesearch" => "AIResearch",
            "scienceai" => "ScienceAI",
            _ => site_id,
        };
        Path::new("G:/Hive-Hub/News-main/output").join(site_name)
    }

    /// Detect source category from URL or title
    fn detect_source_category(url: &str, title: &str) -> String {
        let url_lower = url.to_lowercase();
        let title_lower = title.to_lowercase();

        // Check URL first (most reliable)
        if url_lower.contains("openai.com") || url_lower.contains("openai") {
            return "openai".to_string();
        }
        if url_lower.contains("nvidia.com") || url_lower.contains("nvidia") {
            return "nvidia".to_string();
        }
        if url_lower.contains("google.com") || url_lower.contains("blog.research.google") || url_lower.contains("google ai") {
            return "google".to_string();
        }
        if url_lower.contains("about.fb.com") || url_lower.contains("facebook.com") || url_lower.contains("meta.com") {
            return "meta".to_string();
        }
        if url_lower.contains("anthropic.com") || url_lower.contains("anthropic") || url_lower.contains("claude") {
            return "anthropic".to_string();
        }
        if url_lower.contains("deepseek.ai") || url_lower.contains("deepseek") {
            return "deepseek".to_string();
        }
        if url_lower.contains("x.ai") || url_lower.contains("x.com") {
            return "x".to_string();
        }
        if url_lower.contains("mistral.ai") || url_lower.contains("mistral") {
            return "mistral".to_string();
        }
        if url_lower.contains("alibaba.com") || url_lower.contains("alizila.com") || url_lower.contains("alibaba") {
            return "alibaba".to_string();
        }
        if url_lower.contains("microsoft.com") || url_lower.contains("microsoft") {
            return "microsoft".to_string();
        }

        // Fallback: check title for keywords
        if title_lower.contains("openai") {
            return "openai".to_string();
        }
        if title_lower.contains("nvidia") {
            return "nvidia".to_string();
        }
        if title_lower.contains("google") || title_lower.contains("alphabet") {
            return "google".to_string();
        }
        if title_lower.contains("meta") || title_lower.contains("facebook") {
            return "meta".to_string();
        }
        if title_lower.contains("anthropic") || title_lower.contains("claude") {
            return "anthropic".to_string();
        }
        if title_lower.contains("deepseek") {
            return "deepseek".to_string();
        }
        if title_lower.contains("grok") || title_lower.contains("x.ai") {
            return "x".to_string();
        }
        if title_lower.contains("mistral") {
            return "mistral".to_string();
        }
        if title_lower.contains("alibaba") {
            return "alibaba".to_string();
        }
        if title_lower.contains("microsoft") {
            return "microsoft".to_string();
        }

        // Default: unknown
        "unknown".to_string()
    }

    /// Cleanup after processing articles: verify output files, update registry, remove raw JSONs
    pub async fn cleanup_processed_articles(
        &self,
        article_json_paths: &[PathBuf],
    ) -> Result<CleanupStats> {
        let mut stats = CleanupStats::default();
        let required_files = vec![
            "title.txt",
            "subtitle.txt",
            "article.md",
            "image_categories.txt",
            "source.txt",
            "x.txt",
            "linkedin.txt",
            "shorts_script.txt",
        ];

        for json_path in article_json_paths {
            // Load article ID from JSON path
            let article_id = json_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Get metadata from registry
            let metadata = self.registry.get_metadata(article_id);
            
            if let Some(meta) = metadata {
                // Check if article has output_dir (was published)
                if let Some(output_dir) = &meta.output_dir {
                    // Verify all required files exist
                    let mut all_files_exist = true;
                    for file_name in &required_files {
                        let file_path = output_dir.join(file_name);
                        if !file_path.exists() {
                            all_files_exist = false;
                            eprintln!("  ⚠️  Missing file for {}: {}", article_id, file_name);
                            break;
                        }
                    }

                    if all_files_exist {
                        stats.verified += 1;

                        // Read title and subtitle from output files for registry update
                        let title_file = output_dir.join("title.txt");
                        let subtitle_file = output_dir.join("subtitle.txt");

                        // Try to read and update registry with more information
                        let mut should_update = false;
                        if let Ok(title) = tokio::fs::read_to_string(&title_file).await {
                            if let Ok(subtitle) = tokio::fs::read_to_string(&subtitle_file).await {
                                // Check if registry needs update
                                if let Some(current_meta) = self.registry.get_metadata(article_id) {
                                    if current_meta.title.is_empty() || 
                                       current_meta.title != title.trim() {
                                        // We can't directly update, but mark as updated
                                        should_update = true;
                                    }
                                }
                            }
                        }

                        if should_update {
                            stats.updated += 1;
                        }

                        // Remove JSON from raw directory if all files are verified
                        if json_path.exists() {
                            if let Err(e) = tokio::fs::remove_file(json_path).await {
                                eprintln!("  ⚠️  Failed to remove {}: {}", json_path.display(), e);
                            } else {
                                stats.removed += 1;
                                println!("  🗑️  Removed: {}", json_path.display());
                            }
                        }
                    }
                } else {
                    eprintln!("  ⚠️  Article {} has no output_dir (not published?)", article_id);
                }
            } else {
                eprintln!("  ⚠️  Article {} not found in registry", article_id);
            }
        }

        // Save registry after updates
        if stats.updated > 0 {
            self.registry.save()?;
        }

        Ok(stats)
    }

    /// Default blog prompt (fallback if custom prompt not enabled)
    fn default_blog_prompt() -> String {
        r#"You are an expert technology journalist writing for a major international news portal (style: Wired, The Verge, TechCrunch).

You will receive:
- Raw cleaned content extracted from a website by a scraper (title, body text, date if available).
- Your task is to transform this into a polished news article in **native, natural English**, undetectable as AI-generated.

### 🔹 OUTPUT STRUCTURE (must follow exactly this format):

Title:
- A strong SEO-friendly headline.
- Must contain a clear keyword (AI, model, GPU, language model, etc).
- Must include a "hook" that makes the reader curious.
- Max 60 characters.

Subtitle:
- A compelling summary.
- Max 2 lines.
- Should add tension, consequence, or reason why this matters.

Article:
- 6 to 8 paragraphs.
- Clear journalistic tone, informative but engaging.
- Write like a human: vary sentence length, avoid robotic structure, add light narrative context.
- Make complex ideas simple.
- Never praise a company in a commercial tone. If the scraped text is promotional, rewrite neutrally, e.g.:
  "Grok just launched version 4.5, which claims to improve reasoning by 20%" instead of "Grok proudly revolutionizes AI with its innovative 4.5 model".

### 🔹 LANGUAGE & STYLE RULES

✔ Write in **native-level English**, clear, fluent, and natural.  
✔ Use active voice unless passive is necessary.  
✔ Keep paragraphs short for online reading (2–4 sentences).  
✔ Add context: "This follows previous updates from…", "The move comes as…", "Industry analysts suggest…"  
✔ No filler phrases like "In the ever-changing world of technology…"  
✔ No moralizing or opinions — just informative, sharp writing.

### 🔹 IMAGE CATEGORIES

You must select exactly 3 categories from this exact list ONLY:
ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound

CRITICAL CONSTRAINTS:
- ❌ DO NOT create new categories
- ❌ DO NOT use synonyms or variations
- ✅ ONLY use the 13 categories listed above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must be lowercase, matching the list exactly

### 🔹 SOCIAL MEDIA CONTENT

You must also generate:
1. X (Twitter) post - 280 characters max, engaging hook
2. LinkedIn post - Professional tone, 300 characters max
3. TikTok Shorts script - 2 minutes (~300 words), max 5 seconds per take/frase

TikTok Script Format:
- Each take/frase should be exactly 5 seconds or less
- Include visual directions when needed
- Conversational, engaging, hook-driven

### 🔹 OUTPUT FORMAT (JSON):

{
  "title": "...",                           // Max 60 characters, SEO-friendly, hook
  "subtitle": "...",                        // Max 2 lines, compelling summary
  "article_text": "...",                    // 6-8 paragraphs, journalistic tone
  "image_categories": [                     // Top 3 categories from exact list
    "category1", "category2", "category3"
  ],
  "x_post": "...",                          // Twitter/X post, 280 chars max
  "linkedin_post": "...",                   // LinkedIn post, 300 chars max
  "shorts_script": "..."                    // TikTok 2min script, 5sec per take
}"#.to_string()
    }
}

