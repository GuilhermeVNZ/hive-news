// Writer Module - Content Generation with DeepSeek API
pub mod deepseek_client;
pub mod prompts;
pub mod prompt_compressor;
pub mod content_generator;
pub mod file_writer;

// Re-export commonly used types and functions
pub use deepseek_client::{DeepSeekClient, ArticleResponse, SocialResponse};
pub use content_generator::{WriterService, GeneratedContent};
