//! # Compression Core
//!
//! Statistical compression for LLM prompts using intelligent filtering.
//!
//! ## Overview
//!
//! This library reduces token usage in LLM prompts by using statistical analysis
//! to identify and filter less important content while preserving semantic meaning.
//!
//! ## Architecture
//!
//! The compression pipeline:
//! 1. **Tokenize**: Convert input to tokens using pluggable tokenizer
//! 2. **Analyze**: Apply statistical filtering to identify important content
//! 3. **Filter**: Remove less important tokens/segments
//! 4. **Validate**: Ensure compression preserves semantic quality
//!
//! ## Example
//!
//! ```rust,ignore
//! use compression_prompt::{Compressor, CompressorConfig};
//!
//! let config = CompressorConfig::default();
//! let compressor = Compressor::new(config);
//! let result = compressor.compress(input, &tokenizer)?;
//!
//! println!("Saved {} tokens ({:.1}% compression)",
//!     result.original_tokens - result.compressed_tokens,
//!     (1.0 - result.compression_ratio) * 100.0
//! );
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod compressor;
#[cfg(feature = "image")]
pub mod image_renderer;
pub mod quality_metrics;
pub mod statistical_filter;

pub use compressor::{CompressionResult, Compressor, CompressorConfig, OutputFormat};
#[cfg(feature = "image")]
pub use image_renderer::{ImageRenderer, ImageRendererConfig};
pub use statistical_filter::{StatisticalFilter, StatisticalFilterConfig};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
