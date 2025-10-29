// Content Generator Service
// Main orchestration for generating content from filtered papers
use std::path::{Path, PathBuf};
use std::env;
use anyhow::{Result, Context};
use crate::filter::parser::parse_pdf;
use super::deepseek_client::*;
use super::prompts::*;
use super::prompt_compressor::*;
use super::file_writer::{save_article, save_title, save_linkedin, save_x, save_shorts_script, save_image_categories};

pub struct WriterService {
    deepseek_client: DeepSeekClient,
    prompt_compressor: PromptCompressor,
    output_base: PathBuf,
    site: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedContent {
    pub output_dir: PathBuf,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub compression_ratio: f32,
}

impl WriterService {
    pub fn new() -> Result<Self> {
        let api_key = env::var("DEEPSEEK_API_KEY")
            .context("DEEPSEEK_API_KEY environment variable not set")?;
        
        let base_url = env::var("DEEPSEEK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());
        
        let model = env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string());
        
        let output_base = PathBuf::from(
            env::var("WRITER_OUTPUT_DIR")
                .unwrap_or_else(|_| "G:/Hive-Hub/News-main/output".to_string())
        );
        
        let site = env::var("WRITER_DEFAULT_SITE")
            .unwrap_or_else(|_| "AIResearch".to_string());
        
        Ok(Self {
            deepseek_client: DeepSeekClient::new(api_key, base_url, model),
            prompt_compressor: PromptCompressor::new()?,
            output_base,
            site,
        })
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
        
        // Structure: output/<Site>/<código do artigo>/
        let output_dir = self.output_base.join(&self.site).join(&article_id);
        
        // 2.1. Criar pasta de output
        tokio::fs::create_dir_all(&output_dir).await?;
        println!("  📁 Saving to: {}", output_dir.display());
        
        // 3. PHASE 1: Generate article
        println!("  📝 Building article prompt for: {}", self.site);
        let article_prompt = build_article_prompt(&parsed.text, &[], &self.site);
        
        let estimated_tokens = article_prompt.len() / 4;
        println!("  🗜️  Compressing prompt (~{} tokens)...", estimated_tokens);
        
        let compressed_article = self.prompt_compressor.compress(&article_prompt)
            .context("Failed to compress article prompt")?;
        
        println!("  ✅ Compressed to {} tokens ({:.1}% savings)", 
                 compressed_article.compressed_tokens,
                 compressed_article.compression_ratio * 100.0);
        
        println!("  🤖 Sending to DeepSeek API...");
        let article_response = self.deepseek_client
            .generate_article(&compressed_article.compressed_text)
            .await
            .with_context(|| format!("Failed to generate article for {}", article_id))?;
        
        println!("  ✅ Article generated");
        
        // PHASE 3: Fetch Pixabay image (based on keywords)
        // Images are fetched from Pixabay API using article keywords
        // No longer extracting images from PDF - using Pixabay instead
        
        // 4. PHASE 2: Generate social content
        println!("  📱 Building social media prompts...");
        let social_prompt = build_social_script_prompt(
            &article_response.article_text,
            &parsed.title,
        );
        
        let estimated_social_tokens = social_prompt.len() / 4;
        println!("  🗜️  Compressing social prompt (~{} tokens)...", estimated_social_tokens);
        
        let compressed_social = self.prompt_compressor.compress(&social_prompt)
            .context("Failed to compress social prompt")?;
        
        println!("  ✅ Compressed to {} tokens ({:.1}% savings)", 
                 compressed_social.compressed_tokens,
                 compressed_social.compression_ratio * 100.0);
        
        println!("  🤖 Generating social content...");
        let social_response = self.deepseek_client
            .generate_social_content(&compressed_social.compressed_text)
            .await
            .context("Failed to generate social content")?;
        
        println!("  ✅ Social content generated");
        
        // PHASE 3: Save all content (no longer fetching from Pixabay - using local images instead)
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
