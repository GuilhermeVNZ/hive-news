pub mod authors;
pub mod cache;
pub mod categorizer;
pub mod experiments;
pub mod fake_detector;
pub mod news_filter;
pub mod parser;
pub mod pipeline;
pub mod scorer;
pub mod source_detector;
pub mod validator;

// Re-export principais tipos
pub use news_filter::NewsFilter;
