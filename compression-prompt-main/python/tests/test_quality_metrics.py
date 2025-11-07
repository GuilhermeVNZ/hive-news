"""Tests for quality metrics module."""

import pytest
from compression_prompt import QualityMetrics


def test_perfect_preservation():
    """Test metrics when text is perfectly preserved."""
    text = "Machine Learning is a subset of Artificial Intelligence"
    metrics = QualityMetrics.calculate(text, text)
    
    assert metrics.keyword_retention == 1.0
    assert metrics.entity_retention == 1.0
    assert metrics.vocabulary_ratio == 1.0


def test_lossy_compression():
    """Test metrics for lossy compression."""
    original = "Machine Learning is a powerful subset of Artificial Intelligence"
    compressed = "Machine Learning subset Artificial Intelligence"
    
    metrics = QualityMetrics.calculate(original, compressed)
    
    # Should retain important keywords
    assert metrics.keyword_retention > 0.7
    assert metrics.entity_retention > 0.7
    assert metrics.overall_score > 0.5


def test_entity_extraction():
    """Test entity extraction."""
    text = "Dr. John Smith works at IBM and uses john@example.com"
    words = text.split()
    entities = QualityMetrics._extract_entities(words)
    
    assert "IBM" in entities
    assert "john@example.com" in entities


def test_keyword_extraction():
    """Test keyword extraction."""
    words = ["the", "important", "MachineLearning", "is", "useful"]
    keywords = QualityMetrics._extract_keywords(words)
    
    # Should extract important words, not stopwords
    assert "important" in keywords
    assert "machinelearning" in keywords  # Lowercase
    assert "the" not in keywords
    assert "is" not in keywords


def test_format_output():
    """Test formatted output."""
    original = "Test text with some content"
    compressed = "Test content"
    
    metrics = QualityMetrics.calculate(original, compressed)
    formatted = metrics.format()
    
    assert "Keyword Retention" in formatted
    assert "Entity Retention" in formatted
    assert "Overall Score" in formatted
    assert "%" in formatted


def test_empty_texts():
    """Test handling of empty texts."""
    metrics = QualityMetrics.calculate("", "")
    
    # Should handle gracefully
    assert 0.0 <= metrics.overall_score <= 1.0

