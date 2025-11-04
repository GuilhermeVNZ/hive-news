// News Writer Module
// Handles generating news articles from collected news JSON using DeepSeek API
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use crate::models::raw_document::ArticleMetadata;
use crate::utils::article_registry::RegistryManager;
use crate::utils::site_config_manager::SiteConfigManager;
use super::deepseek_client::DeepSeekClient;
use super::file_writer::{save_article, save_title, save_subtitle, save_linkedin, save_x, save_shorts_script, save_image_categories, save_source, save_slug};
use super::prompt_compressor::PromptCompressor;
use serde_json;

#[allow(dead_code)]
pub struct NewsWriterService {
    output_base: PathBuf,
    registry: RegistryManager,
}

#[allow(dead_code)]
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
        // Extract collection date from JSON path: downloads/raw/{date}/{id}.json
        let collection_date = Self::extract_collection_date_from_path(article_json_path);
        
        // Load article from JSON
        let article_content = tokio::fs::read_to_string(article_json_path).await
            .context("Failed to read article JSON file")?;
        
        // Try to parse and log errors for debugging
        let article: ArticleMetadata = match serde_json::from_str(&article_content) {
            Ok(article) => article,
            Err(e) => {
                eprintln!("Failed to parse article JSON: {}", e);
                eprintln!("JSON content (first 1000 chars): {}", &article_content[..article_content.len().min(1000)]);
                return Err(anyhow::anyhow!("Failed to parse article JSON: {}", e));
            }
        };
        
        // Check which sites should receive this article (from registry destinations)
        println!("  📋 Loading article JSON: {}", article_json_path.display());
        println!("  📅 Collection date: {}", collection_date);
        
        // Use original_title if available, otherwise fallback to title
        let article_title = article.original_title.as_ref()
            .unwrap_or(&article.title);
        println!("  📝 Title: {}", article_title);
        println!("  🔗 URL: {}", article.url);
        println!("  🆔 Article ID: {}", article.id);
        
        // Verificar se artigo está registrado no registry
        println!("  🔍 Checking registry for article {}...", article.id);
        let metadata = self.registry.get_metadata(&article.id);
        
        if metadata.is_none() {
            eprintln!("  ⚠️  WARNING: Article {} not found in registry", article.id);
            eprintln!("     This may indicate a timing issue or the article wasn't properly registered during collection.");
            eprintln!("     Attempting to register now...");
            
            // Tentar registrar o artigo agora (pode ser um problema de timing)
            if let Err(e) = self.registry.register_collected(
                article.id.clone(),
                article_title.clone(),
                article.url.clone(),
                article.url.clone(), // Web articles não têm PDF, usar URL como pdf_url
            ) {
                eprintln!("  ❌ Failed to register article: {}", e);
                return Err(anyhow::anyhow!("Article {} not found in registry and failed to register: {}", article.id, e));
            }
            eprintln!("  ✅ Article registered successfully");
        } else {
            println!("  ✅ Article found in registry");
            if let Some(meta) = &metadata {
                println!("     Status: {:?}", meta.status);
                if let Some(collected_at) = &meta.collected_at {
                    println!("     Collected at: {}", collected_at);
                }
            }
        }
        
        // Obter destinations do registry
        let destinations = metadata
            .as_ref()
            .and_then(|m| m.destinations.as_ref())
            .cloned()
            .unwrap_or_default();

        if destinations.is_empty() {
            eprintln!("  ⚠️  WARNING: No destinations configured for article {}", article.id);
            eprintln!("     This article will be skipped. Possible causes:");
            eprintln!("     1. Article was collected but destinations were not set");
            eprintln!("     2. No sites are configured for this source type");
            eprintln!("     3. Timing issue - destinations not set yet");
            
            // Tentar obter destinations baseado no source_type
            let source_type = article.source_type.as_deref().unwrap_or("unknown");
            eprintln!("     Source type: {}", source_type);
            eprintln!("     Attempting to set default destinations based on source type...");
            
            // Retornar erro mas com mais informações
            let article_title = article.original_title.as_ref()
                .unwrap_or(&article.title);
            return Err(anyhow::anyhow!(
                "No destinations configured for article {}. Source type: {}, Title: {}",
                article.id,
                source_type,
                article_title
            ));
        }

        println!("  🎯 Destinations found: {} site(s)", destinations.len());
        for (idx, dest) in destinations.iter().enumerate() {
            println!("     {}. {}", idx + 1, dest);
        }
        println!("");

        // Load site configurations
        println!("  ⚙️  Loading site configurations...");
        let config_path = Path::new("system_config.json");
        let config_manager = SiteConfigManager::new(config_path);
        println!("  ✅ Config loaded\n");
        
        let mut results = Vec::new();

        // Process article for each destination site
        println!("  🔄 Processing for {} destination(s)...\n", destinations.len());
        
        for (idx, site_id) in destinations.iter().enumerate() {
            println!("  ┌─ [DESTINATION {}/{}] {}", idx + 1, destinations.len(), site_id);
            
            match self.process_article_for_site(&article, site_id, &config_manager, &collection_date).await {
                Ok(result) => {
                    println!("  │  ✅ Successfully generated for {}", site_id);
                    results.push(result);
                    println!("  └─\n");
                }
                Err(e) => {
                    eprintln!("  │  ❌ Failed to process for {}: {}", site_id, e);
                    eprintln!("  └─\n");
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
        collection_date: &str,
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
        
        // Get temperature for blog prompt from config
        let temperature_blog = site_config.temperature_blog.unwrap_or(0.7);

        // Build prompt with article JSON
        let article_json = serde_json::to_string_pretty(article)?;
        let full_prompt = format!("{}\n\n## ARTICLE JSON:\n{}", blog_prompt, article_json);

        println!("    ┌─ [SITE {}] {}", site_id, site_config.name);
        println!("    │  📝 Generating content...");
        println!("    │  📄 Custom blog prompt: {}", site_config.prompt_blog_enabled.unwrap_or(false));
        println!("    │  🔑 API: {} ({})", base_url, model);
        println!("    │  🌡️  Temperature: {:.2}", temperature_blog);
        println!("    │  📊 Prompt size: {} characters", full_prompt.len());

        // Compress prompt if enabled
        // After compression, add source link to ensure it's not affected by compression
        let final_prompt = if site_config.writer.use_compressor.unwrap_or(false) {
            println!("    │  🗜️  Compressing prompt...");
            let compressor = PromptCompressor::new()
                .context("Failed to initialize prompt compressor")?;
            
            let compression_start = std::time::Instant::now();
            let compressed = compressor.compress(&full_prompt)
                .context("Failed to compress prompt")?;
            let compression_duration = compression_start.elapsed();
            
            println!("    │  ✅ Prompt compressed in {:?}: {} → {} tokens ({:.1}% reduction)", 
                compression_duration,
                compressed.original_tokens, 
                compressed.compressed_tokens,
                compressed.compression_ratio * 100.0);
            
            // Add source URL after compression to ensure it's preserved
            let source_link_section = format!(
                "\n\n## SOURCE LINK (FOR FACT VERIFICATION):\n{}\n\n**CRITICAL INSTRUCTION**: Before writing, use the source link above to verify:\n1. Company relationships (competitor, subsidiary, partner, owner)\n2. Ownership claims (does Company A actually own Product B?)\n3. Any ambiguous facts in the scraped content\n4. The primary subject of the article (what company/person is this article ABOUT?)\n\nIf the scraped content seems incorrect or ambiguous, the source link is the authoritative truth. DO NOT assume relationships or ownership based solely on how the scraped text is worded.",
                article.url
            );
            format!("{}{}", compressed.compressed_text, source_link_section)
        } else {
            println!("    │  ⏭️  Prompt compression disabled");
            // Add source link even without compression for consistency
            let source_link_section = format!(
                "\n\n## SOURCE LINK (FOR FACT VERIFICATION):\n{}\n\n**CRITICAL INSTRUCTION**: Before writing, use the source link above to verify:\n1. Company relationships (competitor, subsidiary, partner, owner)\n2. Ownership claims (does Company A actually own Product B?)\n3. Any ambiguous facts in the scraped content\n4. The primary subject of the article (what company/person is this article ABOUT?)\n\nIf the scraped content seems incorrect or ambiguous, the source link is the authoritative truth. DO NOT assume relationships or ownership based solely on how the scraped text is worded.",
                article.url
            );
            format!("{}{}", full_prompt, source_link_section)
        };

        // Create DeepSeek client for this site
        println!("    │  🔧 Creating DeepSeek client...");
        let deepseek_client = DeepSeekClient::new(api_key, base_url, model);

        // Generate article content (includes social media content in response)
        println!("    │  🤖 Calling DeepSeek API...");
        let api_start = std::time::Instant::now();
        let article_response = deepseek_client.generate_article(&final_prompt, Some(temperature_blog)).await
            .context("Failed to generate article content")?;
        let api_duration = api_start.elapsed();
        println!("    │  ✅ API response received in {:?}", api_duration);

        // Detect article source/category (used in folder name)
        println!("    │  🔍 Detecting source category...");
        let article_title = article.original_title.as_ref()
            .unwrap_or(&article.title);
        let source_category = Self::detect_source_category(&article.url, article_title);
        println!("    │  ✅ Source: {}", source_category);
        
        // Generate SEO-friendly slug from title (independent of folder name)
        println!("    │  🔗 Generating SEO-friendly slug...");
        let article_slug = Self::generate_seo_slug(&article_response.title, &site_id, &article.id).await?;
        println!("    │  ✅ Slug: {}", article_slug);
        
        // Create standardized folder name: DATA_SOURCE_ID
        // Format: YYYY-MM-DD_source_category_article_id
        let standardized_folder_name = format!("{}_{}_{}", collection_date, source_category, article.id);
        
        // Create output directory with standardized name
        println!("    │  📁 Creating output directory...");
        let site_output_dir = Self::get_site_output_dir(&site_id);
        let article_output_dir = site_output_dir.join(&standardized_folder_name);
        tokio::fs::create_dir_all(&article_output_dir).await
            .context("Failed to create article output directory")?;
        println!("    │  ✅ Directory created: {}", article_output_dir.display());
        println!("    │  📂 Folder format: {}_{}_{}", collection_date, source_category, article.id);

        // Save all content to separate files
        println!("    │  💾 Saving files...");
        println!("    │  │  📝 Saving title.txt...");
        save_title(&article_output_dir, &article_response.title).await?;
        println!("    │  │  📝 Saving subtitle.txt...");
        save_subtitle(&article_output_dir, &article_response.subtitle).await?;
        println!("    │  │  📄 Saving article.md...");
        save_article(&article_output_dir, &article_response.article_text).await?;
        println!("    │  │  🖼️  Saving image_categories.txt...");
        save_image_categories(&article_output_dir, &article_response.image_categories).await?;
        println!("    │  │  🏷️  Saving source.txt...");
        save_source(&article_output_dir, &source_category).await?;
        println!("    │  │  🔗 Saving slug.txt (SEO-friendly URL)...");
        save_slug(&article_output_dir, &article_slug).await?;
        println!("    │  │  🐦 Saving x.txt...");
        save_x(&article_output_dir, &article_response.x_post).await?;
        println!("    │  │  💼 Saving linkedin.txt...");
        save_linkedin(&article_output_dir, &article_response.linkedin_post).await?;
        println!("    │  │  🎬 Saving shorts_script.txt...");
        save_shorts_script(&article_output_dir, &article_response.shorts_script).await?;
        println!("    │  ✅ All files saved");

        // Update registry to mark as published
        // IMPORTANT: Only update output_dir if it matches the current site_id
        // This prevents overwriting output_dir when processing multiple destinations
        println!("    │  📝 Updating registry...");
        
        // Get current metadata to check existing output_dir
        let current_metadata = self.registry.get_metadata(&article.id);
        
        // Verify that output_dir corresponds to this site_id
        // Note: output_dir now uses standardized format (DATE_SOURCE_ID), but registry may still reference old format
        // We'll validate based on site_id match only, not exact path match
        let expected_output_dir_base = Self::get_site_output_dir(site_id);
        let output_dir_matches = article_output_dir.starts_with(&expected_output_dir_base);
        
        if !output_dir_matches {
            eprintln!("    │  ⚠️  WARNING: output_dir mismatch!");
            eprintln!("    │      Expected base: {}", expected_output_dir_base.display());
            eprintln!("    │      Got:          {}", article_output_dir.display());
            eprintln!("    │      Site ID:      {}", site_id);
            return Err(anyhow::anyhow!("Output directory does not match site ID. Expected path for site '{}' but got '{}'", site_id, article_output_dir.display()));
        }
        
        // Only register if this is the first destination or if output_dir matches
        // For multiple destinations, we should store each in separate directories
        if let Some(existing_meta) = current_metadata {
            if let Some(existing_output_dir) = &existing_meta.output_dir {
                // If output_dir already exists and it's different, this means we're processing multiple destinations
                // In this case, we should keep the original output_dir or create site-specific subdirectories
                if existing_output_dir != &article_output_dir {
                    eprintln!("    │  ⚠️  WARNING: Article already has output_dir: {}", existing_output_dir.display());
                    eprintln!("    │      New output_dir would be: {}", article_output_dir.display());
                    eprintln!("    │      This suggests multiple destinations are being processed.");
                    // Don't overwrite - each destination should have its own directory
                    // For now, we'll still update to the correct one for this site
                }
            }
        }
        
        self.registry.register_published(&article.id, article_output_dir.clone())
            .context("Failed to register article as published")?;
        println!("    │  ✅ Registry updated");

        println!("    └─ ✅ Content saved → {}\n", article_output_dir.display());

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

    /// Extracts collection date from JSON path
    /// Path format: downloads/raw/{YYYY-MM-DD}/{id}.json
    /// Returns date in format YYYY-MM-DD, or current date if extraction fails
    fn extract_collection_date_from_path(json_path: &Path) -> String {
        use regex::Regex;
        
        // Try to extract date from path components
        // Path: downloads/raw/2025-11-03/article_id.json
        let path_str = json_path.to_string_lossy();
        
        // Look for date pattern YYYY-MM-DD in path
        let date_pattern = Regex::new(r"\b(\d{4}-\d{2}-\d{2})\b").ok();
        
        if let Some(re) = date_pattern {
            if let Some(captures) = re.captures(&path_str) {
                if let Some(date_match) = captures.get(1) {
                    return date_match.as_str().to_string();
                }
            }
        }
        
        // Fallback: use parent directory name if it looks like a date
        if let Some(parent) = json_path.parent() {
            if let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) {
                // Check if parent name is a date (YYYY-MM-DD format)
                if parent_name.len() == 10 && parent_name.matches('-').count() == 2 {
                    // Validate it's actually a date by checking pattern
                    let date_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}$").ok();
                    if let Some(re) = date_pattern {
                        if re.is_match(parent_name) {
                            return parent_name.to_string();
                        }
                    }
                }
            }
        }
        
        // Final fallback: use current date
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    /// Generates a SEO-friendly slug from article title (same format as AIResearch)
    /// Formula: title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-')
    /// Ensures uniqueness by checking existing slugs and adding numeric suffix if needed
    async fn generate_seo_slug(title: &str, site_id: &str, article_id: &str) -> Result<String> {
        use regex::Regex;
        
        // Same formula as AIResearch: title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-')
        let mut slug = title.to_lowercase();
        
        // Remove special characters (keep only alphanumeric, spaces, hyphens)
        let re = Regex::new(r"[^\w\s-]").ok();
        if let Some(regex) = re {
            slug = regex.replace_all(&slug, "").to_string();
        }
        
        // Replace spaces with hyphens
        let re_spaces = Regex::new(r"\s+").ok();
        if let Some(regex) = re_spaces {
            slug = regex.replace_all(&slug, "-").to_string();
        }
        
        // Trim hyphens from start and end
        slug = slug.trim_matches('-').to_string();
        
        // Fallback if slug is empty (use article_id prefix)
        if slug.is_empty() {
            slug = format!("article-{}", article_id.chars().take(20).collect::<String>());
        }
        
        // Ensure uniqueness by checking existing slugs in output directory
        let site_output_dir = Self::get_site_output_dir(site_id);
        let unique_slug = Self::ensure_slug_uniqueness(&slug, &site_output_dir).await?;
        
        Ok(unique_slug)
    }

    /// Ensures slug uniqueness by checking existing slugs and adding numeric suffix if needed
    async fn ensure_slug_uniqueness(base_slug: &str, site_output_dir: &Path) -> Result<String> {
        use tokio::fs;
        
        // If directory doesn't exist yet, slug is unique
        if !site_output_dir.exists() {
            return Ok(base_slug.to_string());
        }
        
        // Read all article directories
        let mut entries = match fs::read_dir(site_output_dir).await {
            Ok(entries) => entries,
            Err(_) => return Ok(base_slug.to_string()),
        };
        
        let mut existing_slugs = std::collections::HashSet::new();
        
        // Check each article directory for slug.txt
        while let Some(entry) = entries.next_entry().await? {
            let article_dir = entry.path();
            if article_dir.is_dir() {
                let slug_file = article_dir.join("slug.txt");
                if let Ok(slug_content) = fs::read_to_string(&slug_file).await {
                    let existing_slug = slug_content.trim().to_string();
                    if !existing_slug.is_empty() {
                        existing_slugs.insert(existing_slug);
                    }
                }
            }
        }
        
        // If slug is unique, return as-is
        if !existing_slugs.contains(base_slug) {
            return Ok(base_slug.to_string());
        }
        
        // If not unique, add numeric suffix
        let mut counter = 2;
        loop {
            let candidate = format!("{}-{}", base_slug, counter);
            if !existing_slugs.contains(&candidate) {
                return Ok(candidate);
            }
            counter += 1;
            
            // Safety limit to prevent infinite loop
            if counter > 1000 {
                // Use timestamp as fallback for uniqueness
                let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
                return Ok(format!("{}-{}", base_slug, timestamp));
            }
        }
    }

    /// Detect source category from URL or title using a scoring system
    /// This avoids conflicts when multiple keywords are present by using context-aware scoring
    fn detect_source_category(url: &str, title: &str) -> String {
        let url_lower = url.to_lowercase();
        let title_lower = title.to_lowercase();
        
        // Use a scoring system: domain-specific matches get highest score
        // Then check for keywords with context awareness
        
        struct CategoryScore {
            name: &'static str,
            score: i32,
        }
        
        let mut scores = vec![
            CategoryScore { name: "openai", score: 0 },
            CategoryScore { name: "nvidia", score: 0 },
            CategoryScore { name: "google", score: 0 },
            CategoryScore { name: "meta", score: 0 },
            CategoryScore { name: "anthropic", score: 0 },
            CategoryScore { name: "alibaba", score: 0 },
            CategoryScore { name: "deepseek", score: 0 },
            CategoryScore { name: "x", score: 0 },
            CategoryScore { name: "mistral", score: 0 },
            CategoryScore { name: "microsoft", score: 0 },
            CategoryScore { name: "apple", score: 0 },
            CategoryScore { name: "berkeley", score: 0 },
            CategoryScore { name: "stanford", score: 0 },
            CategoryScore { name: "inflection", score: 0 },
            CategoryScore { name: "stability", score: 0 },
            CategoryScore { name: "intel", score: 0 },
            CategoryScore { name: "amd", score: 0 },
            CategoryScore { name: "cohere", score: 0 },
            CategoryScore { name: "deepmind", score: 0 },
            CategoryScore { name: "characterai", score: 0 },
            CategoryScore { name: "menlo", score: 0 },
            CategoryScore { name: "science", score: 0 },
            CategoryScore { name: "airesearch", score: 0 },
            CategoryScore { name: "huggingface", score: 0 },
            CategoryScore { name: "techcrunch", score: 0 },
            CategoryScore { name: "perplexity", score: 0 },
        ];
        
        // Domain-specific matches get highest priority (score 100)
        if url_lower.contains("openai.com") { scores.iter_mut().find(|s| s.name == "openai").unwrap().score = 100; }
        if url_lower.contains("nvidia.com") { scores.iter_mut().find(|s| s.name == "nvidia").unwrap().score = 100; }
        if url_lower.contains("google.com") || url_lower.contains("blog.research.google") || url_lower.contains("deepmind.google") { scores.iter_mut().find(|s| s.name == "google").unwrap().score = 100; }
        if url_lower.contains("about.fb.com") || url_lower.contains("facebook.com") || url_lower.contains("meta.com") { scores.iter_mut().find(|s| s.name == "meta").unwrap().score = 100; }
        if url_lower.contains("anthropic.com") { scores.iter_mut().find(|s| s.name == "anthropic").unwrap().score = 100; }
        if url_lower.contains("alibaba.com") || url_lower.contains("alizila.com") { scores.iter_mut().find(|s| s.name == "alibaba").unwrap().score = 100; }
        if url_lower.contains("deepseek.ai") { scores.iter_mut().find(|s| s.name == "deepseek").unwrap().score = 100; }
        if url_lower.contains("x.ai") || url_lower.contains("x.com") { scores.iter_mut().find(|s| s.name == "x").unwrap().score = 100; }
        if url_lower.contains("mistral.ai") { scores.iter_mut().find(|s| s.name == "mistral").unwrap().score = 100; }
        if url_lower.contains("microsoft.com") { scores.iter_mut().find(|s| s.name == "microsoft").unwrap().score = 100; }
        if url_lower.contains("machinelearning.apple.com") || url_lower.contains("apple.com") { scores.iter_mut().find(|s| s.name == "apple").unwrap().score = 100; }
        if url_lower.contains("bair.berkeley.edu") { scores.iter_mut().find(|s| s.name == "berkeley").unwrap().score = 100; }
        if url_lower.contains("hai.stanford.edu") { scores.iter_mut().find(|s| s.name == "stanford").unwrap().score = 100; }
        if url_lower.contains("inflection.ai") { scores.iter_mut().find(|s| s.name == "inflection").unwrap().score = 100; }
        if url_lower.contains("stability.ai") { scores.iter_mut().find(|s| s.name == "stability").unwrap().score = 100; }
        if url_lower.contains("intel.com") { scores.iter_mut().find(|s| s.name == "intel").unwrap().score = 100; }
        if url_lower.contains("amd.com") { scores.iter_mut().find(|s| s.name == "amd").unwrap().score = 100; }
        if url_lower.contains("txt.cohere.com") || url_lower.contains("cohere.com") { scores.iter_mut().find(|s| s.name == "cohere").unwrap().score = 100; }
        if url_lower.contains("deepmind.google") || url_lower.contains("deepmind.com") { scores.iter_mut().find(|s| s.name == "deepmind").unwrap().score = 100; scores.iter_mut().find(|s| s.name == "google").unwrap().score = 100; }
        if url_lower.contains("blog.character.ai") || url_lower.contains("character.ai") { scores.iter_mut().find(|s| s.name == "characterai").unwrap().score = 100; }
        if url_lower.contains("menlovc.com") { scores.iter_mut().find(|s| s.name == "menlo").unwrap().score = 100; }
        if url_lower.contains("science.org") { scores.iter_mut().find(|s| s.name == "science").unwrap().score = 100; }
        if url_lower.contains("airesearch.news") { scores.iter_mut().find(|s| s.name == "airesearch").unwrap().score = 100; }
        if url_lower.contains("huggingface.co") || url_lower.contains("huggingface.com") { scores.iter_mut().find(|s| s.name == "huggingface").unwrap().score = 100; }
        if url_lower.contains("techcrunch.com") { scores.iter_mut().find(|s| s.name == "techcrunch").unwrap().score = 100; }
        if url_lower.contains("perplexity.ai") { scores.iter_mut().find(|s| s.name == "perplexity").unwrap().score = 100; }
        
        // If no domain match, check for keywords with context awareness
        // Priority scoring: if keyword appears early in URL path or in title as subject, higher score
        
        // URL keyword matches (score 50-80 based on position)
        if url_lower.contains("alibaba") {
            let alibaba_score = scores.iter_mut().find(|s| s.name == "alibaba").unwrap();
            // Higher score if "alibaba-" appears in URL path (indicates article ABOUT alibaba)
            if url_lower.contains("alibaba-") || url_lower.contains("/alibaba") {
                alibaba_score.score = std::cmp::max(alibaba_score.score, 80);
            } else {
                alibaba_score.score = std::cmp::max(alibaba_score.score, 50);
            }
        }
        
        if url_lower.contains("deepseek") {
            let deepseek_score = scores.iter_mut().find(|s| s.name == "deepseek").unwrap();
            // Lower score if URL also contains "alibaba" (likely article about alibaba mentioning deepseek)
            if url_lower.contains("alibaba") {
                deepseek_score.score = std::cmp::max(deepseek_score.score, 30);
            } else {
                deepseek_score.score = std::cmp::max(deepseek_score.score, 50);
            }
        }
        
        // Title keyword matches (score 40-60, lower than URL but still significant)
        if title_lower.contains("alibaba") || title_lower.contains("alibaba's") {
            let score = if title_lower.starts_with("alibaba") { 60 } else { 40 };
            scores.iter_mut().find(|s| s.name == "alibaba").unwrap().score = std::cmp::max(
                scores.iter().find(|s| s.name == "alibaba").unwrap().score,
                score
            );
        }
        
        if title_lower.contains("deepseek") {
            // Lower score if title mentions alibaba first
            let score = if title_lower.contains("alibaba") { 20 } else { 40 };
            scores.iter_mut().find(|s| s.name == "deepseek").unwrap().score = std::cmp::max(
                scores.iter().find(|s| s.name == "deepseek").unwrap().score,
                score
            );
        }
        
        // Other keyword checks (fallback scoring for keyword matches)
        if url_lower.contains("openai") || title_lower.contains("openai") { scores.iter_mut().find(|s| s.name == "openai").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "openai").unwrap().score, 50); }
        if url_lower.contains("nvidia") || title_lower.contains("nvidia") { scores.iter_mut().find(|s| s.name == "nvidia").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "nvidia").unwrap().score, 50); }
        if url_lower.contains("google") || title_lower.contains("google") { scores.iter_mut().find(|s| s.name == "google").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "google").unwrap().score, 50); }
        if url_lower.contains("meta") || title_lower.contains("meta") || title_lower.contains("facebook") { scores.iter_mut().find(|s| s.name == "meta").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "meta").unwrap().score, 50); }
        if url_lower.contains("anthropic") || title_lower.contains("anthropic") || title_lower.contains("claude") { scores.iter_mut().find(|s| s.name == "anthropic").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "anthropic").unwrap().score, 50); }
        if url_lower.contains("x.ai") || title_lower.contains("grok") { scores.iter_mut().find(|s| s.name == "x").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "x").unwrap().score, 50); }
        if url_lower.contains("mistral") || title_lower.contains("mistral") { scores.iter_mut().find(|s| s.name == "mistral").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "mistral").unwrap().score, 50); }
        if url_lower.contains("microsoft") || title_lower.contains("microsoft") { scores.iter_mut().find(|s| s.name == "microsoft").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "microsoft").unwrap().score, 50); }
        if url_lower.contains("apple") || title_lower.contains("apple") { scores.iter_mut().find(|s| s.name == "apple").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "apple").unwrap().score, 50); }
        if url_lower.contains("berkeley") || title_lower.contains("berkeley") { scores.iter_mut().find(|s| s.name == "berkeley").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "berkeley").unwrap().score, 50); }
        if url_lower.contains("stanford") || title_lower.contains("stanford") { scores.iter_mut().find(|s| s.name == "stanford").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "stanford").unwrap().score, 50); }
        if url_lower.contains("inflection") || title_lower.contains("inflection") { scores.iter_mut().find(|s| s.name == "inflection").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "inflection").unwrap().score, 50); }
        if url_lower.contains("stability") || title_lower.contains("stability") { scores.iter_mut().find(|s| s.name == "stability").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "stability").unwrap().score, 50); }
        if url_lower.contains("intel") || title_lower.contains("intel") { scores.iter_mut().find(|s| s.name == "intel").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "intel").unwrap().score, 50); }
        if url_lower.contains("amd") || title_lower.contains("amd") { scores.iter_mut().find(|s| s.name == "amd").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "amd").unwrap().score, 50); }
        if url_lower.contains("cohere") || title_lower.contains("cohere") { scores.iter_mut().find(|s| s.name == "cohere").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "cohere").unwrap().score, 50); }
        if url_lower.contains("deepmind") || title_lower.contains("deepmind") { scores.iter_mut().find(|s| s.name == "deepmind").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "deepmind").unwrap().score, 50); }
        if url_lower.contains("character.ai") || title_lower.contains("character.ai") { scores.iter_mut().find(|s| s.name == "characterai").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "characterai").unwrap().score, 50); }
        if url_lower.contains("menlo") || title_lower.contains("menlo") { scores.iter_mut().find(|s| s.name == "menlo").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "menlo").unwrap().score, 50); }
        if url_lower.contains("science.org") || title_lower.contains("science") { scores.iter_mut().find(|s| s.name == "science").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "science").unwrap().score, 50); }
        if url_lower.contains("airesearch") || title_lower.contains("airesearch") { scores.iter_mut().find(|s| s.name == "airesearch").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "airesearch").unwrap().score, 50); }
        if url_lower.contains("huggingface") || title_lower.contains("huggingface") || title_lower.contains("hugging face") { scores.iter_mut().find(|s| s.name == "huggingface").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "huggingface").unwrap().score, 50); }
        if url_lower.contains("techcrunch") || title_lower.contains("techcrunch") { scores.iter_mut().find(|s| s.name == "techcrunch").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "techcrunch").unwrap().score, 50); }
        if url_lower.contains("perplexity") || title_lower.contains("perplexity") { scores.iter_mut().find(|s| s.name == "perplexity").unwrap().score = std::cmp::max(scores.iter().find(|s| s.name == "perplexity").unwrap().score, 50); }
        
        // Return category with highest score
        if let Some(winner) = scores.iter().max_by_key(|s| s.score) {
            if winner.score > 0 {
                return winner.name.to_string();
            }
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
            
            println!("  📄 Processing cleanup for: {}", article_id);

            // Get metadata from registry
            let metadata = self.registry.get_metadata(article_id);
            
            if let Some(meta) = metadata {
                // Check if article has output_dir (was published)
                if let Some(output_dir) = &meta.output_dir {
                    // Verify all required files exist
                    let mut all_files_exist = true;
                    let mut missing_files = Vec::new();
                    for file_name in &required_files {
                        let file_path = output_dir.join(file_name);
                        if !file_path.exists() {
                            all_files_exist = false;
                            missing_files.push(file_name);
                        }
                    }

                    if all_files_exist {
                        stats.verified += 1;
                    } else {
                        eprintln!("  ⚠️  Article {} has {} missing files: {:?}", article_id, missing_files.len(), missing_files);
                    }

                    // Read title and subtitle from output files for registry update
                    let title_file = output_dir.join("title.txt");
                    let subtitle_file = output_dir.join("subtitle.txt");

                    // Try to read and update registry with more information
                    let mut should_update = false;
                    if let Ok(title) = tokio::fs::read_to_string(&title_file).await {
                        if let Ok(_subtitle) = tokio::fs::read_to_string(&subtitle_file).await {
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

                    // Remove JSON from raw directory if article was published
                    // Even if some files are missing, we remove the JSON since processing started
                    if json_path.exists() {
                        let path_to_remove = json_path.clone();
                        if let Err(e) = tokio::fs::remove_file(&path_to_remove).await {
                            eprintln!("  ⚠️  Failed to remove {}: {}", path_to_remove.display(), e);
                        } else {
                            stats.removed += 1;
                            println!("  🗑️  Removed: {}", path_to_remove.display());
                        }
                    } else {
                        // Try to find JSON in any date subfolder
                        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
                        let raw_dir = base_dir.join("raw");
                        
                        if raw_dir.exists() {
                            let mut found_json: Option<PathBuf> = None;
                            let mut date_entries = tokio::fs::read_dir(&raw_dir).await?;
                            
                            while let Some(date_entry) = date_entries.next_entry().await? {
                                let date_dir = date_entry.path();
                                if !date_dir.is_dir() {
                                    continue;
                                }
                                
                                let potential_json = date_dir.join(format!("{}.json", article_id));
                                if potential_json.exists() {
                                    found_json = Some(potential_json);
                                    println!("  🔍 Found JSON in date folder: {}", found_json.as_ref().unwrap().display());
                                    break;
                                }
                            }
                            
                            if let Some(found_path) = found_json {
                                if let Err(e) = tokio::fs::remove_file(&found_path).await {
                                    eprintln!("  ⚠️  Failed to remove {}: {}", found_path.display(), e);
                                } else {
                                    stats.removed += 1;
                                    println!("  🗑️  Removed: {}", found_path.display());
                                }
                            } else {
                                println!("  ⚠️  JSON not found for {} (searched in all date folders)", article_id);
                            }
                        } else {
                            println!("  ⚠️  Raw directory does not exist: {}", raw_dir.display());
                        }
                    }
                } else {
                    eprintln!("  ⚠️  Article {} has no output_dir (not published?)", article_id);
                    // Still try to remove JSON if it exists
                    if json_path.exists() {
                        let path_to_remove = json_path.clone();
                        if let Err(e) = tokio::fs::remove_file(&path_to_remove).await {
                            eprintln!("  ⚠️  Failed to remove {}: {}", path_to_remove.display(), e);
                        } else {
                            stats.removed += 1;
                            println!("  🗑️  Removed (unpublished): {}", path_to_remove.display());
                        }
                    }
                }
            } else {
                eprintln!("  ⚠️  Article {} not found in registry", article_id);
                // Still try to remove JSON if it exists
                if json_path.exists() {
                    let path_to_remove = json_path.clone();
                    if let Err(e) = tokio::fs::remove_file(&path_to_remove).await {
                        eprintln!("  ⚠️  Failed to remove {}: {}", path_to_remove.display(), e);
                    } else {
                        stats.removed += 1;
                        println!("  🗑️  Removed (not in registry): {}", path_to_remove.display());
                    }
                }
            }
        }

        // Always save registry after cleanup (even if no content updates, registry may have changed)
        // This ensures the registry reflects the current state after cleanup
        self.registry.save()?;
        if stats.removed > 0 {
            println!("  💾 Registry saved after cleanup ({} files removed)", stats.removed);
        }

        Ok(stats)
    }

    /// Default blog prompt (fallback if custom prompt not enabled)
    pub fn default_blog_prompt() -> String {
        r#"You are an expert technology journalist writing for a major international news portal (style: Wired, The Verge, TechCrunch).

You will receive:
- Raw cleaned content extracted from a website by a scraper (title, body text, date if available).
- A SOURCE LINK to the original article for fact verification.
- Your task is to transform this into a polished news article in **native, natural English**, undetectable as AI-generated.

**CRITICAL FACT-CHECKING RULES:**
1. **ALWAYS verify company relationships**: If content mentions Company A and Company B, verify their relationship (competitor, subsidiary, partner, etc.) using the source link provided.
2. **DO NOT assume ownership**: Just because an article mentions "Company A's Product B" does NOT mean Company A owns Product B. Verify actual ownership/relationship.
3. **Check the source link**: The provided source link is the authoritative source. Use it to verify any claims about relationships, ownership, or facts that seem ambiguous in the scraped content.
4. **When in doubt, be conservative**: If you cannot verify a fact from the source, state it as uncertainty ("may be", "appears to be", "reportedly") rather than stating it as fact.

### 🔹 OUTPUT STRUCTURE (must follow exactly this format):

Title:
- **CRITICAL**: The generated title MUST be DIFFERENT from the original title in the article JSON. NEVER use the same title as the original source.
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

### 🔹 IMAGE CATEGORIES (CRITICAL - READ CAREFULLY)

**⚠️ MANDATORY: You MUST select exactly 3 categories from THIS EXACT LIST ONLY (NO EXCEPTIONS):**
ai, coding, crypto, database, ethics, games, hardware, legal, network, robotics, science, security, sound

**ABSOLUTE CONSTRAINTS (VIOLATING THESE WILL CAUSE ERRORS):**
- ❌ DO NOT create new categories like "infrastructure", "partnership", "computing", "technology", etc.
- ❌ DO NOT use synonyms, variations, or related words
- ❌ DO NOT translate or pluralize (keep exactly as listed)
- ✅ ONLY use these exact 13 lowercase strings from the list above
- ✅ Order by priority: most relevant first, second choice, third choice
- ✅ Must match the list EXACTLY (case-sensitive, spelling-sensitive)

**VALIDATION: Before returning JSON, verify your image_categories array contains ONLY items from this list:**
["ai", "coding", "crypto", "database", "ethics", "games", "hardware", "legal", "network", "robotics", "science", "security", "sound"]

**EXAMPLES OF CORRECT USAGE:**
- For cloud/infrastructure articles → ["database", "network", "hardware"]
- For AI partnerships → ["ai", "network", "hardware"]
- For computing research → ["hardware", "ai", "science"]

**EXAMPLES OF INCORRECT USAGE (DO NOT USE THESE):**
- ❌ ["infrastructure", "partnership", "computing"] ← WRONG
- ❌ ["technology", "innovation", "business"] ← WRONG
- ❌ ["cloud", "server", "enterprise"] ← WRONG

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

