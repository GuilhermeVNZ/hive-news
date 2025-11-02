// Content Generator Service
// Main orchestration for generating content from filtered papers
use std::path::{Path, PathBuf};
use std::env;
use anyhow::{Result, Context};
use crate::filter::parser::parse_pdf;
use crate::utils::site_config_manager::SiteConfigManager;
use super::deepseek_client::*;
use super::prompts::*;
use super::prompt_compressor::*;
use super::file_writer::{save_article, save_title, save_subtitle, save_linkedin, save_x, save_shorts_script, save_image_categories};

#[allow(dead_code)]
pub struct WriterService {
    deepseek_client: DeepSeekClient,
    prompt_compressor: PromptCompressor,
    output_base: PathBuf,
    site: String,
    site_id: String,
    // Custom prompts from config (if enabled)
    prompt_article: Option<String>,
    prompt_social: Option<String>,
    prompt_blog: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeneratedContent {
    pub output_dir: PathBuf,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub compression_ratio: f32,
}

#[allow(dead_code)]
impl WriterService {
    /// Creates a new WriterService, reading configuration from system_config.json
    /// Falls back to environment variables if config file is not available
    pub fn new() -> Result<Self> {
        Self::new_with_site(None)
    }

    /// Creates a new WriterService for a specific site
    /// Reads writer configuration from system_config.json for that site
    pub fn new_with_site(site_id: Option<&str>) -> Result<Self> {
        let config_path = Path::new("system_config.json");
        let config_manager = SiteConfigManager::new(config_path);
        
        // Try to load config from JSON file
        let (api_key, base_url, model, site_name, prompt_article, prompt_social, prompt_blog) = if let Some(site_id) = site_id {
            match config_manager.get_site_config(site_id) {
                Ok(Some(site_config)) => {
                    let writer_config = &site_config.writer;
                    if writer_config.enabled {
                        let api_key = writer_config.api_key.clone()
                            .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
                            .context("API key not found in config or environment")?;
                        
                        let base_url = writer_config.base_url.clone()
                            .unwrap_or_else(|| env::var("DEEPSEEK_BASE_URL")
                                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string()));
                        
                        let model = writer_config.model.clone();
                        
                        // Load custom prompts if enabled
                        let prompt_article = if site_config.prompt_article_enabled.unwrap_or(false) {
                            site_config.prompt_article.clone()
                        } else {
                            None
                        };
                        let prompt_social = if site_config.prompt_social_enabled.unwrap_or(false) {
                            site_config.prompt_social.clone()
                        } else {
                            None
                        };
                        let prompt_blog = if site_config.prompt_blog_enabled.unwrap_or(false) {
                            site_config.prompt_blog.clone()
                        } else {
                            None
                        };
                        
                        println!("✅ Loaded writer config from system_config.json for site: {}", site_id);
                        println!("   Provider: {}, Model: {}", writer_config.provider, model);
                        if prompt_article.is_some() {
                            println!("   📝 Using custom article prompt");
                        }
                        if prompt_social.is_some() {
                            println!("   📱 Using custom social prompt");
                        }
                        if prompt_blog.is_some() {
                            println!("   📄 Using custom blog prompt");
                        }
                        
                        (api_key, base_url, model, site_config.name.clone(), prompt_article, prompt_social, prompt_blog)
                    } else {
                        anyhow::bail!("Writer is disabled for site: {}", site_id);
                    }
                }
                Ok(None) => {
                    println!("⚠️  Site {} not found in config, using environment variables", site_id);
                    let (api_key, base_url, model, site_name) = Self::from_env(site_id.to_string())?;
                    (api_key, base_url, model, site_name, None, None, None)
                }
                Err(e) => {
                    println!("⚠️  Failed to load config for site {}: {}, using environment variables", site_id, e);
                    let (api_key, base_url, model, site_name) = Self::from_env(site_id.to_string())?;
                    (api_key, base_url, model, site_name, None, None, None)
                }
            }
        } else {
            // No site_id provided, try to use default site or env vars
            let default_site_id = env::var("WRITER_DEFAULT_SITE")
                .unwrap_or_else(|_| "airesearch".to_string());
            
            match config_manager.get_site_config(&default_site_id) {
                Ok(Some(site_config)) => {
                    let writer_config = &site_config.writer;
                    if writer_config.enabled {
                        let api_key = writer_config.api_key.clone()
                            .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
                            .context("API key not found in config or environment")?;
                        
                        let base_url = writer_config.base_url.clone()
                            .unwrap_or_else(|| env::var("DEEPSEEK_BASE_URL")
                                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string()));
                        
                        let model = writer_config.model.clone();
                        
                        // Load custom prompts if enabled
                        let prompt_article = if site_config.prompt_article_enabled.unwrap_or(false) {
                            site_config.prompt_article.clone()
                        } else {
                            None
                        };
                        let prompt_social = if site_config.prompt_social_enabled.unwrap_or(false) {
                            site_config.prompt_social.clone()
                        } else {
                            None
                        };
                        let prompt_blog = if site_config.prompt_blog_enabled.unwrap_or(false) {
                            site_config.prompt_blog.clone()
                        } else {
                            None
                        };
                        
                        println!("✅ Loaded writer config from system_config.json for default site: {}", default_site_id);
                        println!("   Provider: {}, Model: {}", writer_config.provider, model);
                        if prompt_article.is_some() {
                            println!("   📝 Using custom article prompt");
                        }
                        if prompt_social.is_some() {
                            println!("   📱 Using custom social prompt");
                        }
                        if prompt_blog.is_some() {
                            println!("   📄 Using custom blog prompt");
                        }
                        
                        (api_key, base_url, model, site_config.name.clone(), prompt_article, prompt_social, prompt_blog)
                    } else {
                        let (api_key, base_url, model, site_name) = Self::from_env(site_config.name.clone())?;
                        (api_key, base_url, model, site_name, None, None, None)
                    }
                }
                _ => {
                    println!("⚠️  Default site {} not found in config, using environment variables", default_site_id);
                    let (api_key, base_url, model, site_name) = Self::from_env("AIResearch".to_string())?;
                    (api_key, base_url, model, site_name, None, None, None)
                }
            }
        };
        
        let output_base = PathBuf::from(
            env::var("WRITER_OUTPUT_DIR")
                .unwrap_or_else(|_| "G:/Hive-Hub/News-main/output".to_string())
        );
        
        let actual_site_id = match site_id {
            Some(id) => id.to_string(),
            None => env::var("WRITER_DEFAULT_SITE")
                .unwrap_or_else(|_| "airesearch".to_string()),
        };
        
        Ok(Self {
            deepseek_client: DeepSeekClient::new(api_key, base_url, model),
            prompt_compressor: PromptCompressor::new()?,
            output_base,
            site: site_name.clone(),
            site_id: actual_site_id,
            prompt_article,
            prompt_social,
            prompt_blog,
        })
    }

    /// Helper to load config from environment variables (fallback)
    fn from_env(site_name: String) -> Result<(String, String, String, String)> {
        let api_key = env::var("DEEPSEEK_API_KEY")
            .context("DEEPSEEK_API_KEY environment variable not set")?;
        
        let base_url = env::var("DEEPSEEK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());
        
        let model = env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string());
        
        Ok((api_key, base_url, model, site_name))
    }
    
    /// Retorna o site atual (ex: AIResearch, Nature, Science)
    pub fn get_site(&self) -> &str {
        &self.site
    }
    
    /// Retorna o diretório base de output
    pub fn get_output_base(&self) -> &Path {
        &self.output_base
    }
    
    pub async fn process_pdf(&self, pdf_path: &Path) -> Result<GeneratedContent> {
        // 1. Extract text from PDF
        println!("  📄 Parsing PDF...");
        let parsed = parse_pdf(pdf_path)?;
        
        // 2. Extract article ID (sem criar pasta ainda)
        let article_id = extract_article_id(pdf_path);
        
        // Map site_id to correct output directory name
        let output_dir_name = match self.site_id.to_lowercase().as_str() {
            "airesearch" => "AIResearch",
            "scienceai" => "ScienceAI",
            _ => self.site.as_str(), // Fallback to site name if not mapped
        };
        
        // Structure: output/<SiteID>/<código do artigo>/
        let output_dir = self.output_base.join(output_dir_name).join(&article_id);
        
        // 2.1. Criar pasta de output
        tokio::fs::create_dir_all(&output_dir).await?;
        println!("  📁 Saving to: {}", output_dir.display());
        
        // 3. PHASE 1: Generate article
        println!("  📝 Building article prompt for: {}", self.site);
        let article_prompt = if let Some(ref custom_prompt) = self.prompt_article {
            println!("  📝 Using custom article prompt from config");
            // Replace {{paper_text}} placeholder if present, otherwise prepend paper text
            if custom_prompt.contains("{{paper_text}}") {
                custom_prompt.replace("{{paper_text}}", &parsed.text)
            } else {
                format!("{}\n\n## PAPER TEXT (YOUR ONLY SOURCE):\n{}", custom_prompt, &parsed.text)
            }
        } else {
            build_article_prompt(&parsed.text, &[], &self.site)
        };
        
        let estimated_tokens = article_prompt.len() / 4;
        println!("  🗜️  Compressing prompt (~{} tokens)...", estimated_tokens);
        
        let compressed_article = self.prompt_compressor.compress(&article_prompt)
            .context("Failed to compress article prompt")?;
        
        println!("  ✅ Compressed to {} tokens ({:.1}% savings)", 
                 compressed_article.compressed_tokens,
                 compressed_article.compression_ratio * 100.0);
        
        println!("  🤖 Sending to DeepSeek API...");
        let article_response = match self.deepseek_client
            .generate_article(&compressed_article.compressed_text)
            .await
        {
            Ok(response) => {
                println!("  ✅ Article generated");
                response
            }
            Err(e) => {
                eprintln!("  ❌ Failed to generate article for {}: {}", article_id, e);
                eprintln!("  📄 PDF: {}", pdf_path.display());
                eprintln!("  📊 Prompt tokens: {} (compressed from {})", 
                         compressed_article.compressed_tokens,
                         compressed_article.original_tokens);
                return Err(e).with_context(|| format!("Failed to generate article for {}", article_id));
            }
        };
        
        // 4. PHASE 2: Generate social content
        println!("  📱 Building social media prompts...");
        let social_prompt = if let Some(ref custom_prompt) = self.prompt_social {
            println!("  📱 Using custom social prompt from config");
            // Replace placeholders if present, otherwise prepend article text
            let mut prompt = custom_prompt.clone();
            prompt = prompt.replace("{{article_text}}", &article_response.article_text);
            prompt = prompt.replace("{{paper_title}}", &parsed.title);
            if !custom_prompt.contains("{{article_text}}") && !custom_prompt.contains("{{paper_title}}") {
                format!("{}\n\n## ARTICLE TEXT:\n{}\n\n## PAPER TITLE:\n{}", 
                    custom_prompt, &article_response.article_text, &parsed.title)
            } else {
                prompt
            }
        } else {
            build_social_script_prompt(
                &article_response.article_text,
                &parsed.title,
            )
        };
        
        let estimated_social_tokens = social_prompt.len() / 4;
        println!("  🗜️  Compressing social prompt (~{} tokens)...", estimated_social_tokens);
        
        let compressed_social = self.prompt_compressor.compress(&social_prompt)
            .context("Failed to compress social prompt")?;
        
        println!("  ✅ Compressed to {} tokens ({:.1}% savings)", 
                 compressed_social.compressed_tokens,
                 compressed_social.compression_ratio * 100.0);
        
        println!("  🤖 Generating social content...");
        let social_response = match self.deepseek_client
            .generate_social_content(&compressed_social.compressed_text)
            .await
        {
            Ok(response) => {
                println!("  ✅ Social content generated");
                response
            }
            Err(e) => {
                eprintln!("  ❌ Failed to generate social content for {}: {}", article_id, e);
                eprintln!("  📄 PDF: {}", pdf_path.display());
                return Err(e).context("Failed to generate social content");
            }
        };
        
        // PHASE 3: Save all content
        println!("  💾 Saving content to disk...");
        self.save_content(
            &output_dir,
            &article_response,
            &social_response,
        ).await?;
        
        Ok(GeneratedContent {
            output_dir,
            original_tokens: compressed_article.original_tokens + compressed_social.original_tokens,
            compressed_tokens: compressed_article.compressed_tokens + compressed_social.compressed_tokens,
            compression_ratio: (compressed_article.compression_ratio + compressed_social.compression_ratio) / 2.0,
        })
    }
    
    async fn save_content(
        &self,
        output_dir: &Path,
        article: &ArticleResponse,
        social: &SocialResponse,
    ) -> Result<()> {
        // Save title (short hook for frontend)
        save_title(output_dir, &article.title).await?;
        
        // Save subtitle (SEO-optimized, max 2 lines)
        if !article.subtitle.is_empty() {
            save_subtitle(output_dir, &article.subtitle).await?;
        }
        
        // Save article
        save_article(output_dir, &article.article_text).await?;
        
        // Save social posts
        save_linkedin(output_dir, &social.linkedin_post).await?;
        save_x(output_dir, &social.x_post).await?;
        
        // Save video script
        save_shorts_script(output_dir, &social.shorts_script).await?;
        
        // Save image categories (for future image selection)
        if !article.image_categories.is_empty() {
            println!("  📑 Image categories: {:?}", article.image_categories);
            save_image_categories(output_dir, &article.image_categories).await?;
        }
        
        // NO LONGER SAVING metadata.json - not needed
        
        Ok(())
    }
}

fn extract_article_id(pdf_path: &Path) -> String {
    pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}
