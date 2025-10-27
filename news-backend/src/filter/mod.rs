pub mod authors;
pub mod cache;
pub mod categorizer;
pub mod experiments;
pub mod fake_detector;
pub mod parser;
pub mod pipeline;
pub mod scorer;
pub mod source_detector;
pub mod validator;

// Re-export principais tipos
pub use parser::ParsedPdf;
pub use pipeline::{FilterStats, run_filter_pipeline};
pub use scorer::FilterResult;
pub use source_detector::{SourceType, detect_source_type};
