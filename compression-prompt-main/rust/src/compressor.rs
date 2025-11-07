//! Main compression pipeline and result structures.

use crate::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Output format for compression result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Plain text output (default).
    Text,
    /// PNG image output (1024x1024 monospace).
    Image,
}

/// Compression errors.
#[derive(Error, Debug)]
pub enum CompressionError {
    /// Compression would increase token count.
    #[error("Compression ratio {0:.2} < 1.0, would increase tokens")]
    NegativeGain(f32),

    /// Input too short to compress.
    #[error("Input too short ({0} tokens), minimum is {1}")]
    InputTooShort(usize, usize),
}

/// Configuration for the compressor.
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    /// Target compression ratio (default: 0.5 = 50% of original size).
    pub target_ratio: f32,

    /// Minimum input tokens to attempt compression (default: 100).
    pub min_input_tokens: usize,

    /// Minimum input bytes to attempt compression (default: 1024).
    pub min_input_bytes: usize,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            min_input_tokens: 100,
            min_input_bytes: 1024,
        }
    }
}

/// Result of compression operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// The compressed text (always included).
    pub compressed: String,

    /// Optional image output (PNG bytes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Vec<u8>>,

    /// Output format used.
    pub format: OutputFormat,

    /// Original token count.
    pub original_tokens: usize,

    /// Compressed token count.
    pub compressed_tokens: usize,

    /// Compression ratio (compressed/original).
    pub compression_ratio: f32,

    /// Number of tokens removed.
    pub tokens_removed: usize,
}

/// Main compressor.
#[derive(Debug)]
pub struct Compressor {
    /// Compression configuration.
    pub config: CompressorConfig,
    /// Statistical filter instance
    filter: StatisticalFilter,
}

impl Compressor {
    /// Create a new compressor with given configuration.
    pub fn new(config: CompressorConfig) -> Self {
        let filter_config = StatisticalFilterConfig {
            compression_ratio: config.target_ratio,
            ..Default::default()
        };
        let filter = StatisticalFilter::new(filter_config);
        Self { config, filter }
    }

    /// Create a new compressor with custom statistical filter configuration.
    pub fn with_filter_config(
        config: CompressorConfig,
        filter_config: StatisticalFilterConfig,
    ) -> Self {
        let filter = StatisticalFilter::new(filter_config);
        Self { config, filter }
    }

    /// Compress input text using statistical filtering.
    ///
    /// Returns an error if compression would be counterproductive.
    pub fn compress(&self, input: &str) -> Result<CompressionResult, CompressionError> {
        self.compress_with_format(input, OutputFormat::Text)
    }

    /// Compress input text with specified output format.
    ///
    /// # Arguments
    ///
    /// * `input` - The text to compress
    /// * `format` - Output format (Text or Image)
    ///
    /// # Returns
    ///
    /// CompressionResult with compressed text and optional image data.
    ///
    /// # Errors
    ///
    /// Returns `CompressionError` if:
    /// - Input is too short (< min_input_bytes or < min_input_tokens)
    /// - Compression would increase size (ratio >= 1.0)
    pub fn compress_with_format(
        &self,
        input: &str,
        format: OutputFormat,
    ) -> Result<CompressionResult, CompressionError> {
        // Step 1: Check input size (bytes)
        let input_bytes = input.len();
        if input_bytes < self.config.min_input_bytes {
            return Err(CompressionError::InputTooShort(
                input_bytes,
                self.config.min_input_bytes,
            ));
        }

        // Step 2: Estimate tokens (using char count / 4 as rough estimate)
        let original_tokens = input.chars().count() / 4;
        if original_tokens < self.config.min_input_tokens {
            return Err(CompressionError::InputTooShort(
                original_tokens,
                self.config.min_input_tokens,
            ));
        }

        // Step 3: Apply statistical filtering
        let compressed = self.filter.compress(input);

        // Step 4: Validate compression ratio
        let compressed_tokens = compressed.chars().count() / 4;
        let compression_ratio = compressed_tokens as f32 / original_tokens as f32;

        if compression_ratio >= 1.0 {
            return Err(CompressionError::NegativeGain(compression_ratio));
        }

        let tokens_removed = original_tokens.saturating_sub(compressed_tokens);

        // Step 5: Generate image if requested
        let image_data = if format == OutputFormat::Image {
            #[cfg(feature = "image")]
            {
                use crate::image_renderer::ImageRenderer;
                let renderer = ImageRenderer::default();
                match renderer.render_to_png(&compressed) {
                    Ok(data) => Some(data),
                    Err(_) => None, // Fallback: no image on error
                }
            }
            #[cfg(not(feature = "image"))]
            {
                None // Image feature not enabled
            }
        } else {
            None
        };

        Ok(CompressionResult {
            compressed,
            image_data,
            format,
            original_tokens,
            compressed_tokens,
            compression_ratio,
            tokens_removed,
        })
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new(CompressorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_too_short() {
        let compressor = Compressor::default();
        let input = "short text";

        let result = compressor.compress(input);
        assert!(matches!(result, Err(CompressionError::InputTooShort(_, _))));
    }

    #[test]
    fn test_compression_min_bytes() {
        let compressor = Compressor::default();

        // Create input < 1024 bytes but > 100 tokens
        let mut input = String::new();
        for _ in 0..200 {
            input.push_str("ab ");
        }

        // Should fail due to min_input_bytes
        assert!(input.len() < 1024);
        let result = compressor.compress(&input);
        assert!(matches!(result, Err(CompressionError::InputTooShort(_, _))));
    }

    #[test]
    fn test_compression_min_tokens() {
        let config = CompressorConfig {
            min_input_bytes: 10, // Lower byte requirement
            min_input_tokens: 500,
            ..Default::default()
        };
        let compressor = Compressor::new(config);

        // Create input > 10 bytes but < 500 tokens
        let input = "some short text with few tokens";

        // Should fail due to min_input_tokens
        let result = compressor.compress(input);
        assert!(matches!(result, Err(CompressionError::InputTooShort(_, _))));
    }
}
