// Writer Module - Content Generation with DeepSeek API
pub mod deepseek_client;
pub mod prompts;
pub mod prompt_compressor;
pub mod content_generator;
pub mod file_writer;
pub mod news_writer;

// Re-export commonly used types and functions
pub use content_generator::WriterService;
