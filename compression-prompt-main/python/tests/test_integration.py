"""Integration tests for end-to-end compression workflows."""

import pytest
from compression_prompt import (
    Compressor, CompressorConfig, StatisticalFilterConfig,
    QualityMetrics
)


def test_end_to_end_compression():
    """Test complete compression workflow."""
    
    text = """
    Machine Learning is a subset of Artificial Intelligence that focuses on 
    building systems that can learn from data. Deep Learning uses neural networks 
    with multiple layers to process complex patterns. Natural Language Processing 
    enables computers to understand human language. Computer Vision allows machines 
    to interpret visual information. These technologies are transforming industries.
    """ * 10  # Make it longer
    
    compressor = Compressor()
    result = compressor.compress(text)
    
    # Verify result structure
    assert result.compressed
    assert result.original_tokens > 0
    assert result.compressed_tokens > 0
    assert result.compression_ratio < 1.0
    assert result.tokens_removed > 0
    
    # Verify compression happened
    assert len(result.compressed) < len(text)
    
    # Check quality metrics
    metrics = QualityMetrics.calculate(text, result.compressed)
    assert metrics.overall_score > 0.5  # Should maintain reasonable quality


def test_different_compression_ratios():
    """Test different compression ratios produce different results."""
    
    text = """
    Artificial Intelligence has revolutionized many fields. Machine Learning 
    algorithms can learn from data. Deep Learning neural networks process 
    complex patterns. Natural Language Processing understands human language.
    """ * 20
    
    ratios = [0.3, 0.5, 0.7]
    results = []
    
    for ratio in ratios:
        config = CompressorConfig(target_ratio=ratio, min_input_bytes=100)
        compressor = Compressor(config)
        result = compressor.compress(text)
        results.append(result)
    
    # More aggressive compression should remove more tokens
    assert results[0].compressed_tokens < results[1].compressed_tokens < results[2].compressed_tokens


def test_preserve_important_content():
    """Test that important content is preserved."""
    
    text = """
    The Vectorizer tool is essential for processing data. Synap provides 
    distributed caching. UMICP handles inter-process communication. These 
    are critical components of the HiveLLM system architecture.
    """ * 10
    
    compressor = Compressor()
    result = compressor.compress(text)
    
    # Domain terms should be preserved
    assert "Vectorizer" in result.compressed
    assert "Synap" in result.compressed
    assert "UMICP" in result.compressed


def test_code_block_preservation():
    """Test that code blocks are protected."""
    
    text = """
    Here is an example of Rust code:
    
    ```rust
    fn main() {
        println!("Hello, world!");
        let x = 42;
    }
    ```
    
    This code demonstrates a simple Rust program with variable declaration
    and printing to stdout using the println! macro.
    """ * 5
    
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    config = CompressorConfig(min_input_bytes=100)
    compressor = Compressor(config, filter_config)
    result = compressor.compress(text)
    
    # Code should be largely preserved
    assert "```rust" in result.compressed or "println!" in result.compressed or "main" in result.compressed


def test_multilingual_support():
    """Test compression of multilingual text."""
    
    texts = {
        "english": "The quick brown fox jumps over the lazy dog. " * 50,
        "spanish": "El rápido zorro marrón salta sobre el perro perezoso. " * 50,
        "portuguese": "A rápida raposa marrom pula sobre o cão preguiçoso. " * 50,
    }
    
    compressor = Compressor()
    
    for lang, text in texts.items():
        result = compressor.compress(text)
        
        # Should compress successfully
        assert result.compression_ratio < 1.0
        assert len(result.compressed) > 0
        
        # Should maintain quality
        metrics = QualityMetrics.calculate(text, result.compressed)
        assert metrics.overall_score > 0.4, f"Low quality for {lang}"


def test_json_preservation():
    """Test that JSON structures are preserved."""
    
    text = """
    The API returns a response in JSON format: {"status": "success", "data": {"id": 123, "name": "test"}}
    This JSON structure contains nested objects and should be preserved during compression.
    """ * 10
    
    filter_config = StatisticalFilterConfig(compression_ratio=0.4)
    config = CompressorConfig(min_input_bytes=100)
    compressor = Compressor(config, filter_config)
    result = compressor.compress(text)
    
    # JSON should be preserved
    assert '{"status":' in result.compressed or '"status"' in result.compressed or "success" in result.compressed


def test_url_and_path_preservation():
    """Test that URLs and file paths are preserved."""
    
    text = """
    Check the documentation at https://github.com/hivellm/compression-prompt for details.
    The source code is located in src/main.rs and the tests are in tests/integration_test.rs.
    Visit http://example.com for more information.
    """ * 10
    
    filter_config = StatisticalFilterConfig(compression_ratio=0.4)
    config = CompressorConfig(min_input_bytes=100)
    compressor = Compressor(config, filter_config)
    result = compressor.compress(text)
    
    # URLs and paths should be preserved
    assert ("https://github.com" in result.compressed or 
            "src/main.rs" in result.compressed or 
            "http://example.com" in result.compressed)


def test_negation_preservation():
    """Test that negations are preserved."""
    
    text = """
    This is not a test. You should never ignore this warning. The system will not 
    proceed if you don't confirm. This cannot be undone. We haven't seen this issue 
    before. The data isn't available.
    """ * 10
    
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    config = CompressorConfig(min_input_bytes=100)
    compressor = Compressor(config, filter_config)
    result = compressor.compress(text)
    
    # Negations should be preserved
    negations_found = sum([
        "not" in result.compressed,
        "never" in result.compressed,
        "don't" in result.compressed,
        "cannot" in result.compressed,
        "haven't" in result.compressed,
        "isn't" in result.compressed,
    ])
    
    assert negations_found >= 3, "Most negations should be preserved"


def test_large_input():
    """Test compression of large input."""
    
    # Create a large text (>10KB)
    base_text = """
    Artificial Intelligence and Machine Learning have transformed the technology 
    landscape. Deep Learning models process vast amounts of data. Natural Language 
    Processing enables human-computer interaction. Computer Vision interprets 
    visual information. Reinforcement Learning optimizes decision-making.
    """
    
    large_text = base_text * 100  # ~50KB
    
    compressor = Compressor()
    result = compressor.compress(large_text)
    
    assert result.compression_ratio < 1.0
    assert result.tokens_removed > 0
    
    # Quality should still be good
    metrics = QualityMetrics.calculate(large_text, result.compressed)
    assert metrics.keyword_retention > 0.8
    assert metrics.overall_score > 0.7

