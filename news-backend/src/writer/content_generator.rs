// Content Generator Service
// Main orchestration for generating content from filtered papers
use super::deepseek_client::*;
use super::file_writer::{
    save_article, save_image_categories, save_linkedin, save_shorts_script, save_subtitle,
    save_title, save_x,
};
use super::prompt_compressor::*;
use super::prompts::*;
use crate::filter::parser::parse_pdf;
use crate::utils::site_config_manager::SiteConfigManager;
use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct WriterService {
    deepseek_client: DeepSeekClient,
    prompt_compressor: Option<PromptCompressor>,
    output_base: PathBuf,
    site: String,
    site_id: String,
    use_compressor: bool,
    // Temperature per prompt channel
    temperature_article: f64,
    temperature_social: f64,
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
        let (
            api_key,
            base_url,
            model,
            site_name,
            use_compressor,
            temperature_article,
            temperature_social,
            prompt_article,
            prompt_social,
            prompt_blog,
        ) = if let Some(site_id) = site_id {
            match config_manager.get_site_config(site_id) {
                Ok(Some(site_config)) => {
                    let writer_config = &site_config.writer;
                    if writer_config.enabled {
                        let api_key = writer_config
                            .api_key
                            .clone()
                            .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
                            .context("API key not found in config or environment")?;

                        let base_url = writer_config.base_url.clone().unwrap_or_else(|| {
                            env::var("DEEPSEEK_BASE_URL")
                                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string())
                        });

                        let model = writer_config.model.clone();

                        // Load compressor setting from config
                        let use_compressor = writer_config.use_compressor.unwrap_or(false);

                        // Load temperature per prompt from config
                        let temperature_article = site_config.temperature_article.unwrap_or(0.7);
                        let temperature_social = site_config.temperature_social.unwrap_or(0.8);

                        // Load custom prompts if enabled
                        let prompt_article = if site_config.prompt_article_enabled.unwrap_or(false)
                        {
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

                        println!(
                            "✅ Loaded writer config from system_config.json for site: {}",
                            site_id
                        );
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
                        println!(
                            "   🗜️  Compressor: {}",
                            if use_compressor {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        );
                        println!(
                            "   🌡️  Temperature - Article: {:.2}, Social: {:.2}",
                            temperature_article, temperature_social
                        );

                        (
                            api_key,
                            base_url,
                            model,
                            site_config.name.clone(),
                            use_compressor,
                            temperature_article,
                            temperature_social,
                            prompt_article,
                            prompt_social,
                            prompt_blog,
                        )
                    } else {
                        anyhow::bail!("Writer is disabled for site: {}", site_id);
                    }
                }
                Ok(None) => {
                    println!(
                        "⚠️  Site {} not found in config, using environment variables",
                        site_id
                    );
                    let (api_key, base_url, model, site_name) =
                        Self::from_env(site_id.to_string())?;
                    (
                        api_key, base_url, model, site_name, false, 0.7, 0.8, None, None, None,
                    )
                }
                Err(e) => {
                    println!(
                        "⚠️  Failed to load config for site {}: {}, using environment variables",
                        site_id, e
                    );
                    let (api_key, base_url, model, site_name) =
                        Self::from_env(site_id.to_string())?;
                    (
                        api_key, base_url, model, site_name, false, 0.7, 0.8, None, None, None,
                    )
                }
            }
        } else {
            // No site_id provided, try to use default site or env vars
            let default_site_id =
                env::var("WRITER_DEFAULT_SITE").unwrap_or_else(|_| "airesearch".to_string());

            match config_manager.get_site_config(&default_site_id) {
                Ok(Some(site_config)) => {
                    let writer_config = &site_config.writer;
                    if writer_config.enabled {
                        let api_key = writer_config
                            .api_key
                            .clone()
                            .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
                            .context("API key not found in config or environment")?;

                        let base_url = writer_config.base_url.clone().unwrap_or_else(|| {
                            env::var("DEEPSEEK_BASE_URL")
                                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string())
                        });

                        let model = writer_config.model.clone();

                        // Load compressor setting from config
                        let use_compressor = writer_config.use_compressor.unwrap_or(false);

                        // Load temperature per prompt from config
                        let temperature_article = site_config.temperature_article.unwrap_or(0.7);
                        let temperature_social = site_config.temperature_social.unwrap_or(0.8);

                        // Load custom prompts if enabled
                        let prompt_article = if site_config.prompt_article_enabled.unwrap_or(false)
                        {
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

                        println!(
                            "✅ Loaded writer config from system_config.json for default site: {}",
                            default_site_id
                        );
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
                        println!(
                            "   🗜️  Compressor: {}",
                            if use_compressor {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        );
                        println!(
                            "   🌡️  Temperature - Article: {:.2}, Social: {:.2}",
                            temperature_article, temperature_social
                        );

                        (
                            api_key,
                            base_url,
                            model,
                            site_config.name.clone(),
                            use_compressor,
                            temperature_article,
                            temperature_social,
                            prompt_article,
                            prompt_social,
                            prompt_blog,
                        )
                    } else {
                        let (api_key, base_url, model, site_name) =
                            Self::from_env(site_config.name.clone())?;
                        (
                            api_key, base_url, model, site_name, false, 0.7, 0.8, None, None, None,
                        )
                    }
                }
                _ => {
                    println!(
                        "⚠️  Default site {} not found in config, using environment variables",
                        default_site_id
                    );
                    let (api_key, base_url, model, site_name) =
                        Self::from_env("AIResearch".to_string())?;
                    (
                        api_key, base_url, model, site_name, false, 0.7, 0.8, None, None, None,
                    )
                }
            }
        };

        let output_base = PathBuf::from(env::var("WRITER_OUTPUT_DIR").unwrap_or_else(|_| {
            crate::utils::path_resolver::resolve_workspace_path("output")
                .display()
                .to_string()
        }));

        let actual_site_id = match site_id {
            Some(id) => id.to_string(),
            None => env::var("WRITER_DEFAULT_SITE").unwrap_or_else(|_| "airesearch".to_string()),
        };

        Ok(Self {
            deepseek_client: DeepSeekClient::new(api_key, base_url, model),
            prompt_compressor: if use_compressor {
                Some(PromptCompressor::new()?)
            } else {
                None
            },
            output_base,
            site: site_name.clone(),
            site_id: actual_site_id,
            use_compressor,
            temperature_article,
            temperature_social,
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

        let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

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
                format!(
                    "{}\n\n## PAPER TEXT (YOUR ONLY SOURCE):\n{}",
                    custom_prompt, &parsed.text
                )
            }
        } else {
            // Try to use randomized prompt, fallback to default if it fails
            match load_random_article_prompt(&parsed.text) {
                Ok(random_prompt) => {
                    println!("  🎲 Using randomized prompt from article_randomizer");
                    random_prompt
                }
                Err(e) => {
                    println!("  ⚠️  Failed to load randomized prompt: {}", e);
                    println!("  📝 Falling back to default article prompt");
                    build_article_prompt(&parsed.text, &[], &self.site)
                }
            }
        };

        let (final_article_prompt, original_tokens, compressed_tokens, compression_ratio) = if self
            .use_compressor
        {
            if let Some(ref compressor) = self.prompt_compressor {
                let estimated_tokens = article_prompt.len() / 4;
                println!("  🗜️  Compressing prompt (~{} tokens)...", estimated_tokens);

                let compressed_article = compressor
                    .compress(&article_prompt)
                    .context("Failed to compress article prompt")?;

                println!(
                    "  ✅ Compressed to {} tokens ({:.1}% savings)",
                    compressed_article.compressed_tokens,
                    compressed_article.compression_ratio * 100.0
                );

                (
                    compressed_article.compressed_text,
                    compressed_article.original_tokens,
                    compressed_article.compressed_tokens,
                    compressed_article.compression_ratio,
                )
            } else {
                // Fallback if compressor is None despite use_compressor being true
                println!("  ⚠️  Compressor enabled but not initialized, using uncompressed prompt");
                let tokens = article_prompt.len() / 4;
                (article_prompt, tokens, tokens, 0.0)
            }
        } else {
            println!("  ⏭️  Prompt compression disabled");
            let tokens = article_prompt.len() / 4;
            (article_prompt, tokens, tokens, 0.0)
        };

        println!("  🤖 Sending to DeepSeek API...");
        let article_response = match self
            .deepseek_client
            .generate_article(&final_article_prompt, Some(self.temperature_article))
            .await
        {
            Ok(response) => {
                println!("  ✅ Article generated");
                response
            }
            Err(e) => {
                eprintln!("  ❌ Failed to generate article for {}: {}", article_id, e);
                eprintln!("  📄 PDF: {}", pdf_path.display());
                eprintln!(
                    "  📊 Prompt tokens: {} (compressed from {})",
                    compressed_tokens, original_tokens
                );
                return Err(e)
                    .with_context(|| format!("Failed to generate article for {}", article_id));
            }
        };

        // 4. PHASE 2: Generate social content
        // CRITICAL: Social content should be included in article response (from combined prompt)
        // Only use fallback if fields are completely missing (should not happen with updated prompts)
        let (social_response, social_original_tokens, social_compressed_tokens, social_compression_ratio) = 
            if !article_response.linkedin_post.is_empty() 
            && !article_response.x_post.is_empty() 
            && !article_response.shorts_script.is_empty() {
            println!("  ✅ Social content included in article response (no additional API call needed)");
            (
                SocialResponse {
                    linkedin_post: article_response.linkedin_post.clone(),
                    x_post: article_response.x_post.clone(),
                    shorts_script: article_response.shorts_script.clone(),
                },
                0, // No tokens used for social (already in article)
                0,
                0.0,
            )
        } else {
            // FALLBACK ONLY: Generate social content separately (should rarely happen)
            // This indicates the prompt may not have included social fields or DeepSeek didn't return them
            eprintln!("  ⚠️  WARNING: Social content not found in article response, using fallback generation");
            eprintln!("  ⚠️  This should not happen with updated prompts. Check prompt format.");
            println!("  📱 Building social media prompts (FALLBACK)...");
            eprintln!("  🔍 DEBUG: Article response fields: title={}, subtitle={}, article_text_len={}, linkedin={}, x={}, shorts={}", 
                !article_response.title.is_empty(),
                !article_response.subtitle.is_empty(),
                article_response.article_text.len(),
                !article_response.linkedin_post.is_empty(),
                !article_response.x_post.is_empty(),
                !article_response.shorts_script.is_empty());
            let social_prompt = if let Some(ref custom_prompt) = self.prompt_social {
            println!("  📱 Using custom social prompt from config");
            // Replace placeholders if present, otherwise prepend article text
            let mut prompt = custom_prompt.clone();
            prompt = prompt.replace("{{article_text}}", &article_response.article_text);
            prompt = prompt.replace("{{paper_title}}", &parsed.title);
            if !custom_prompt.contains("{{article_text}}")
                && !custom_prompt.contains("{{paper_title}}")
            {
                format!(
                    "{}\n\n## ARTICLE TEXT:\n{}\n\n## PAPER TITLE:\n{}",
                    custom_prompt, &article_response.article_text, &parsed.title
                )
            } else {
                prompt
            }
        } else {
            build_social_script_prompt(&article_response.article_text, &parsed.title)
        };

        let (
            final_social_prompt,
            social_original_tokens,
            social_compressed_tokens,
            social_compression_ratio,
        ) = if self.use_compressor {
            if let Some(ref compressor) = self.prompt_compressor {
                let estimated_social_tokens = social_prompt.len() / 4;
                println!(
                    "  🗜️  Compressing social prompt (~{} tokens)...",
                    estimated_social_tokens
                );

                let compressed_social = compressor
                    .compress(&social_prompt)
                    .context("Failed to compress social prompt")?;

                println!(
                    "  ✅ Compressed to {} tokens ({:.1}% savings)",
                    compressed_social.compressed_tokens,
                    compressed_social.compression_ratio * 100.0
                );

                // CRITICAL: ALWAYS add JSON format instructions back after compression
                // The compressor may have removed them, so we ALWAYS add them explicitly
                // This ensures DeepSeek returns the correct format even if compressor removed instructions
                let mut final_compressed_social = compressed_social.compressed_text.clone();
                
                // Remove any existing JSON instructions to avoid duplicates, then add fresh ones
                // This ensures we always have the correct, complete instructions
                let json_instruction_marker = "## CRITICAL: JSON OUTPUT REQUIRED";
                if let Some(pos) = final_compressed_social.rfind(json_instruction_marker) {
                    final_compressed_social.truncate(pos);
                    final_compressed_social = final_compressed_social.trim_end().to_string();
                }
                
                // ALWAYS append the complete JSON format instructions
                // CRITICAL: Make it absolutely clear this is for SOCIAL content, not article content
                final_compressed_social.push_str("\n\n## ⚠️⚠️⚠️ CRITICAL: THIS IS A SOCIAL MEDIA CONTENT REQUEST - NOT AN ARTICLE REQUEST ⚠️⚠️⚠️\n\n## JSON OUTPUT REQUIRED - FOLLOW THIS EXACT FORMAT:\n\nYou are generating SOCIAL MEDIA CONTENT ONLY. You MUST return a JSON object with EXACTLY these 3 fields (no more, no less):\n{\n  \"linkedin_post\": \"Your LinkedIn post text here (300 chars max)\",\n  \"x_post\": \"Your X/Twitter post text here (280 chars max)\",\n  \"shorts_script\": \"Your YouTube Shorts script here (2 minutes, ~300 words)\"\n}\n\n🚫🚫🚫 FORBIDDEN FIELDS - DO NOT INCLUDE THESE: 🚫🚫🚫\n- \"title\" (FORBIDDEN - this is for articles, not social content)\n- \"article_text\" (FORBIDDEN - this is for articles, not social content)\n- \"subtitle\" (FORBIDDEN - this is for articles, not social content)\n- \"image_categories\" (FORBIDDEN - this is for articles, not social content)\n- ANY other fields (FORBIDDEN)\n\n✅✅✅ REQUIRED FIELDS - ONLY THESE 3 ARE ALLOWED: ✅✅✅\n- \"linkedin_post\" (REQUIRED)\n- \"x_post\" (REQUIRED)\n- \"shorts_script\" (REQUIRED)\n\n⚠️ Return your response as valid JSON format with ONLY the 3 required fields above.\n⚠️ The word \"json\" must appear in your response format.\n⚠️ Remember: This is SOCIAL MEDIA content, NOT an article. Do NOT return article fields.");

                (
                    final_compressed_social,
                    compressed_social.original_tokens,
                    compressed_social.compressed_tokens,
                    compressed_social.compression_ratio,
                )
            } else {
                println!("  ⚠️  Compressor enabled but not initialized, using uncompressed prompt");
                let tokens = social_prompt.len() / 4;
                (social_prompt, tokens, tokens, 0.0)
            }
        } else {
            println!("  ⏭️  Social prompt compression disabled");
            // Even without compression, ensure JSON instructions are present
            let mut social_prompt_checked = social_prompt.clone();
            let lower_prompt = social_prompt_checked.to_lowercase();
            let has_json = lower_prompt.contains("json");
            let has_linkedin = lower_prompt.contains("linkedin_post");
            let has_x_post = lower_prompt.contains("x_post");
            let has_shorts = lower_prompt.contains("shorts_script");
            let has_forbidden_warning = lower_prompt.contains("forbidden") || lower_prompt.contains("do not include");
            
            if !has_json || !has_linkedin || !has_x_post || !has_shorts || !has_forbidden_warning {
                // Remove any existing incomplete JSON instructions first
                let json_instruction_marker = "## CRITICAL: JSON OUTPUT REQUIRED";
                if let Some(pos) = social_prompt_checked.rfind(json_instruction_marker) {
                    social_prompt_checked.truncate(pos);
                    social_prompt_checked = social_prompt_checked.trim_end().to_string();
                }
                
                // Add complete JSON format instructions
                // CRITICAL: Make it absolutely clear this is for SOCIAL content, not article content
                social_prompt_checked.push_str("\n\n## ⚠️⚠️⚠️ CRITICAL: THIS IS A SOCIAL MEDIA CONTENT REQUEST - NOT AN ARTICLE REQUEST ⚠️⚠️⚠️\n\n## JSON OUTPUT REQUIRED - FOLLOW THIS EXACT FORMAT:\n\nYou are generating SOCIAL MEDIA CONTENT ONLY. You MUST return a JSON object with EXACTLY these 3 fields (no more, no less):\n{\n  \"linkedin_post\": \"Your LinkedIn post text here (300 chars max)\",\n  \"x_post\": \"Your X/Twitter post text here (280 chars max)\",\n  \"shorts_script\": \"Your YouTube Shorts script here (2 minutes, ~300 words)\"\n}\n\n🚫🚫🚫 FORBIDDEN FIELDS - DO NOT INCLUDE THESE: 🚫🚫🚫\n- \"title\" (FORBIDDEN - this is for articles, not social content)\n- \"article_text\" (FORBIDDEN - this is for articles, not social content)\n- \"subtitle\" (FORBIDDEN - this is for articles, not social content)\n- \"image_categories\" (FORBIDDEN - this is for articles, not social content)\n- ANY other fields (FORBIDDEN)\n\n✅✅✅ REQUIRED FIELDS - ONLY THESE 3 ARE ALLOWED: ✅✅✅\n- \"linkedin_post\" (REQUIRED)\n- \"x_post\" (REQUIRED)\n- \"shorts_script\" (REQUIRED)\n\n⚠️ Return your response as valid JSON format with ONLY the 3 required fields above.\n⚠️ The word \"json\" must appear in your response format.\n⚠️ Remember: This is SOCIAL MEDIA content, NOT an article. Do NOT return article fields.");
            }
            
            let tokens = social_prompt_checked.len() / 4;
            (social_prompt_checked, tokens, tokens, 0.0)
        };

        // CRITICAL: ALWAYS ensure JSON format instructions are present before sending to DeepSeek
        // DeepSeek API requires the word "json" in the prompt when using response_format: json_object
        // This is a final safety check to guarantee correct format even if previous steps failed
        let mut final_social_prompt_with_json = final_social_prompt.clone();
        let lower_prompt = final_social_prompt_with_json.to_lowercase();
        
        // Check if JSON instructions are present and complete
        let has_json = lower_prompt.contains("json");
        let has_linkedin = lower_prompt.contains("linkedin_post");
        let has_x_post = lower_prompt.contains("x_post");
        let has_shorts = lower_prompt.contains("shorts_script");
        let has_forbidden_warning = lower_prompt.contains("forbidden") || lower_prompt.contains("do not include");
        
        // ALWAYS add instructions if any are missing or incomplete
        if !has_json || !has_linkedin || !has_x_post || !has_shorts || !has_forbidden_warning {
            // Remove any existing incomplete JSON instructions first
            let json_instruction_marker = "## CRITICAL: JSON OUTPUT REQUIRED";
            if let Some(pos) = final_social_prompt_with_json.rfind(json_instruction_marker) {
                final_social_prompt_with_json.truncate(pos);
                final_social_prompt_with_json = final_social_prompt_with_json.trim_end().to_string();
            }
            
            // Add complete, explicit JSON format instructions
            // CRITICAL: Make it absolutely clear this is for SOCIAL content, not article content
            final_social_prompt_with_json.push_str("\n\n## ⚠️⚠️⚠️ CRITICAL: THIS IS A SOCIAL MEDIA CONTENT REQUEST - NOT AN ARTICLE REQUEST ⚠️⚠️⚠️\n\n## JSON OUTPUT REQUIRED - FOLLOW THIS EXACT FORMAT:\n\nYou are generating SOCIAL MEDIA CONTENT ONLY. You MUST return a JSON object with EXACTLY these 3 fields (no more, no less):\n{\n  \"linkedin_post\": \"Your LinkedIn post text here (300 chars max)\",\n  \"x_post\": \"Your X/Twitter post text here (280 chars max)\",\n  \"shorts_script\": \"Your YouTube Shorts script here (2 minutes, ~300 words)\"\n}\n\n🚫🚫🚫 FORBIDDEN FIELDS - DO NOT INCLUDE THESE: 🚫🚫🚫\n- \"title\" (FORBIDDEN - this is for articles, not social content)\n- \"article_text\" (FORBIDDEN - this is for articles, not social content)\n- \"subtitle\" (FORBIDDEN - this is for articles, not social content)\n- \"image_categories\" (FORBIDDEN - this is for articles, not social content)\n- ANY other fields (FORBIDDEN)\n\n✅✅✅ REQUIRED FIELDS - ONLY THESE 3 ARE ALLOWED: ✅✅✅\n- \"linkedin_post\" (REQUIRED)\n- \"x_post\" (REQUIRED)\n- \"shorts_script\" (REQUIRED)\n\n⚠️ Return your response as valid JSON format with ONLY the 3 required fields above.\n⚠️ The word \"json\" must appear in your response format.\n⚠️ Remember: This is SOCIAL MEDIA content, NOT an article. Do NOT return article fields.");
        }

            println!("  🤖 Generating social content...");
            eprintln!("  🔍 DEBUG: Final social prompt length: {} chars", final_social_prompt_with_json.len());
            eprintln!("  🔍 DEBUG: Prompt contains 'json': {}", final_social_prompt_with_json.to_lowercase().contains("json"));
            eprintln!("  🔍 DEBUG: Prompt contains 'linkedin_post': {}", final_social_prompt_with_json.to_lowercase().contains("linkedin_post"));
            eprintln!("  🔍 DEBUG: Prompt contains 'x_post': {}", final_social_prompt_with_json.to_lowercase().contains("x_post"));
            eprintln!("  🔍 DEBUG: Prompt contains 'shorts_script': {}", final_social_prompt_with_json.to_lowercase().contains("shorts_script"));
            eprintln!("  🔍 DEBUG: Prompt contains 'FORBIDDEN': {}", final_social_prompt_with_json.to_lowercase().contains("forbidden"));
            eprintln!("  🔍 DEBUG: Last 500 chars of prompt: {}", &final_social_prompt_with_json[final_social_prompt_with_json.len().saturating_sub(500)..]);
            
            let response = match self
                .deepseek_client
                .generate_social_content(&final_social_prompt_with_json, Some(self.temperature_social))
                .await
            {
                Ok(response) => {
                    println!("  ✅ Social content generated");
                    response
                }
                Err(e) => {
                    eprintln!(
                        "  ❌ Failed to generate social content for {}: {}",
                        article_id, e
                    );
                    eprintln!("  📄 PDF: {}", pdf_path.display());
                    eprintln!("  🔍 DEBUG: This error occurred during social content generation fallback.");
                    eprintln!("  🔍 DEBUG: The article was generated successfully, but social content failed.");
                    eprintln!("  🔍 DEBUG: Check the prompt JSON instructions above to see if they were preserved.");
                    eprintln!("  ⚠️  WARNING: Continuing with empty social content to save article files.");
                    
                    // Return empty social content instead of failing completely
                    // This ensures article files are still saved even if social content fails
                    SocialResponse {
                        linkedin_post: String::new(),
                        x_post: String::new(),
                        shorts_script: String::new(),
                    }
                }
            };
            
            (
                response,
                social_original_tokens,
                social_compressed_tokens,
                social_compression_ratio,
            )
        };

        // PHASE 3: Save all content
        println!("  💾 Saving content to disk...");
        self.save_content(&output_dir, &article_response, &social_response)
            .await?;

        Ok(GeneratedContent {
            output_dir,
            original_tokens: original_tokens + social_original_tokens,
            compressed_tokens: compressed_tokens + social_compressed_tokens,
            compression_ratio: (compression_ratio + social_compression_ratio) / 2.0,
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
