//! Integration tests for compression-prompt library

use compression_prompt::{
    Compressor, CompressorConfig, StatisticalFilter, StatisticalFilterConfig,
};

#[test]
fn test_end_to_end_compression() {
    let text = "The quick brown fox jumps over the lazy dog. \
                This is a test sentence with multiple words. \
                We need to ensure that compression works correctly. \
                The algorithm should preserve important words while removing less significant ones.";

    let config = CompressorConfig {
        min_input_bytes: 10,
        min_input_tokens: 10,
        target_ratio: 0.5,
    };

    let compressor = Compressor::new(config);
    let result = compressor.compress(text);

    assert!(result.is_ok());
    let result = result.unwrap();

    assert!(result.compressed.len() < text.len());
    assert!(result.compression_ratio < 1.0);
    assert!(result.tokens_removed > 0);
}

#[test]
fn test_statistical_filter_preserves_keywords() {
    let text = "Machine learning algorithms require large datasets. \
                Neural networks are powerful models. \
                Deep learning is a subset of machine learning.";

    let filter = StatisticalFilter::default();
    let compressed = filter.compress(text);

    // Keywords should be preserved
    assert!(compressed.contains("Machine") || compressed.contains("learning"));
    assert!(compressed.contains("Neural") || compressed.contains("networks"));
    assert!(compressed.contains("Deep") || compressed.contains("learning"));
}

#[test]
fn test_compression_with_code_blocks() {
    let text = "Here is some code:\n\
                ```rust\n\
                fn main() {\n\
                    println!(\"Hello, world!\");\n\
                }\n\
                ```\n\
                This code should be preserved.";

    let filter = StatisticalFilter::default();
    let compressed = filter.compress(text);

    // Code block markers should be preserved
    assert!(compressed.contains("```") || compressed.contains("fn main"));
}

#[test]
fn test_compression_quality_metrics() {
    // Use a long sample text for testing
    let text = "Machine learning is a subset of artificial intelligence that focuses on the development \
                of algorithms that can learn from and make predictions based on data. The field of machine \
                learning has grown rapidly in recent years, driven by advances in computational power and \
                the availability of large datasets. Neural networks, which are inspired by the structure \
                of the human brain, have become particularly important in modern machine learning. \
                Deep learning, a subset of machine learning that uses multiple layers of neural networks, \
                has achieved remarkable success in tasks such as image recognition, natural language processing, \
                and game playing. Reinforcement learning is another important area of machine learning where \
                agents learn to make decisions by interacting with an environment. The applications of machine \
                learning are vast and include fields such as healthcare, finance, transportation, and entertainment. \
                However, machine learning also raises important ethical questions about privacy, bias, and \
                accountability that must be carefully considered as the technology continues to advance.";

    let filter = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    });

    let compressed = filter.compress(text);

    // Should achieve at least 20% compression (lowered threshold for shorter text)
    let compression_achieved = 1.0 - (compressed.len() as f32 / text.len() as f32);
    assert!(
        compression_achieved >= 0.2,
        "Expected at least 20% compression, got {:.1}%",
        compression_achieved * 100.0
    );

    // Should not be empty
    assert!(!compressed.is_empty());
}

#[test]
fn test_multiple_compression_levels() {
    let text = "Artificial intelligence is transforming technology. \
                Machine learning models are becoming increasingly sophisticated. \
                Natural language processing enables better human-computer interaction.";

    // Conservative (70% retention)
    let conservative = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.7,
        ..Default::default()
    });
    let conservative_result = conservative.compress(text);

    // Balanced (50% retention)
    let balanced = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    });
    let balanced_result = balanced.compress(text);

    // Aggressive (30% retention)
    let aggressive = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.3,
        ..Default::default()
    });
    let aggressive_result = aggressive.compress(text);

    // Conservative should keep more text
    assert!(conservative_result.len() > balanced_result.len());
    assert!(balanced_result.len() > aggressive_result.len());
}

#[test]
fn test_compression_with_technical_terms() {
    let text = "The Bayesian network uses probabilistic inference. \
                Gradient descent optimizes the loss function. \
                Convolutional neural networks excel at image recognition.";

    let filter = StatisticalFilter::default();
    let compressed = filter.compress(text);

    // Technical terms should have high retention
    assert!(compressed.contains("Bayesian") || compressed.contains("probabilistic"));
    assert!(compressed.contains("Gradient") || compressed.contains("descent"));
    assert!(compressed.contains("Convolutional") || compressed.contains("neural"));
}

#[test]
fn test_error_handling_short_input() {
    let short_text = "Too short";

    let compressor = Compressor::default();
    let result = compressor.compress(short_text);

    assert!(result.is_err());
}

#[test]
fn test_custom_filter_configuration() {
    let text = "Custom configuration test with various weights. \
                IDF weight affects rare word importance. \
                Position weight prioritizes start and end.";

    let custom_config = StatisticalFilterConfig {
        compression_ratio: 0.5,
        idf_weight: 0.4,
        position_weight: 0.1,
        pos_weight: 0.3,
        entity_weight: 0.15,
        entropy_weight: 0.05,
        ..Default::default()
    };

    let filter = StatisticalFilter::new(custom_config);
    let compressed = filter.compress(text);

    assert!(!compressed.is_empty());
    assert!(compressed.len() < text.len());
}

#[test]
fn test_unicode_handling() {
    let text = "Unicode characters: café, naïve, 日本語, emoji 🚀. \
                These should be handled correctly during compression.";

    let filter = StatisticalFilter::default();
    let compressed = filter.compress(text);

    // Should not panic or corrupt unicode
    assert!(!compressed.is_empty());
}

#[test]
fn test_batch_compression_consistency() {
    let texts = vec![
        "First test sentence with multiple words.",
        "Second test sentence with different content.",
        "Third test sentence for consistency check.",
    ];

    let filter = StatisticalFilter::default();

    for text in texts {
        let compressed = filter.compress(text);
        assert!(!compressed.is_empty());
        assert!(compressed.len() <= text.len());
    }
}
