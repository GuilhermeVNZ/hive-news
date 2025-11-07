"""Tests for statistical filter module."""

import pytest
from compression_prompt import StatisticalFilter, StatisticalFilterConfig


def test_basic_compression():
    """Test basic compression functionality."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.5)
    filter = StatisticalFilter(filter_config)
    
    text = "The quick brown fox jumps over the lazy dog"
    compressed = filter.compress(text)
    
    original_words = len(text.split())
    compressed_words = len(compressed.split())
    
    assert compressed_words <= original_words
    assert compressed_words > 0


def test_code_block_protection():
    """Test that code blocks are protected from removal."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    filter = StatisticalFilter(filter_config)
    
    text = 'Here is some code ```rust fn main() { println!("Hello"); }``` that should be preserved'
    compressed = filter.compress(text)
    
    # Code block content should be preserved
    assert ("```rust" in compressed or "println!" in compressed or
            "main" in compressed)


def test_json_protection():
    """Test that JSON structures are protected."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    filter = StatisticalFilter(filter_config)
    
    text = 'The config is {"key": "value"} and it should remain intact'
    compressed = filter.compress(text)
    
    # JSON should be preserved
    assert ('{"key":' in compressed or '"key"' in compressed or
            "value" in compressed)


def test_path_preservation():
    """Test that file paths are preserved."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.4)
    filter = StatisticalFilter(filter_config)
    
    text = "Check the file in src/main.rs for the implementation details"
    compressed = filter.compress(text)
    
    # Path should be preserved
    assert ("src/main.rs" in compressed or 
            ("src" in compressed and "main.rs" in compressed))


def test_negation_preservation():
    """Test that negations are always preserved."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    filter = StatisticalFilter(filter_config)
    
    text = "do not remove this critical information"
    compressed = filter.compress(text)
    
    # "not" should always be preserved
    assert "not" in compressed


def test_domain_terms_preservation():
    """Test that domain-specific terms are preserved."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    filter = StatisticalFilter(filter_config)
    
    text = "use the Vectorizer tool to process data"
    compressed = filter.compress(text)
    
    # Domain term "Vectorizer" should be preserved
    assert "Vectorizer" in compressed


def test_identifier_protection():
    """Test that code identifiers are protected."""
    filter_config = StatisticalFilterConfig(compression_ratio=0.3)
    filter = StatisticalFilter(filter_config)
    
    text = "call the getUserData function from user_service module"
    compressed = filter.compress(text)
    
    # Identifiers should be preserved
    assert "getUserData" in compressed or "user_service" in compressed


def test_protection_masks_can_be_disabled():
    """Test that protection masks can be disabled."""
    filter_config = StatisticalFilterConfig(
        compression_ratio=0.3,
        enable_protection_masks=False
    )
    filter = StatisticalFilter(filter_config)
    
    text = "Check src/main.rs for details"
    compressed = filter.compress(text)
    
    # With protection disabled, should still work
    assert len(compressed) > 0


def test_empty_text():
    """Test handling of empty text."""
    filter = StatisticalFilter()
    
    result = filter.compress("")
    assert result == ""


def test_score_words():
    """Test word scoring functionality."""
    filter = StatisticalFilter()
    
    text = "The important ImportantWord should be preserved"
    importances = filter.score_words(text)
    
    assert len(importances) == len(text.split())
    
    # Find scores for different word types
    scores = {imp.text: imp.score for imp in importances}
    
    # "The" (stopword) should have lower score than "important"
    assert scores.get("The", 0) < scores.get("important", 1)
    
    # CamelCase identifier should have high score
    assert scores.get("ImportantWord", 0) > 1.0  # Should be protected (inf)

