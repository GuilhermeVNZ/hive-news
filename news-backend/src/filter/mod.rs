pub mod source_detector;
pub mod parser;
pub mod cache;
pub mod experiments;
pub mod fake_detector;
pub mod validator;
pub mod authors;
pub mod categorizer;
pub mod scorer;
pub mod pipeline;

// Re-export principais tipos
pub use source_detector::{SourceType, detect_source_type};
pub use pipeline::{run_filter_pipeline, FilterStats};
pub use parser::ParsedPdf;
pub use scorer::FilterResult;


