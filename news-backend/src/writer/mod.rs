// Writer Module - Content Generation with DeepSeek API
pub mod content_generator;
pub mod deepseek_client;
pub mod file_writer;
pub mod news_writer;
pub mod prompt_compressor;
pub mod prompts;

// Re-export commonly used types and functions
pub use content_generator::WriterService;
