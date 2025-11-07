"""Tests for compressor module."""

import pytest
from compression_prompt import (
    Compressor, CompressorConfig, CompressionError,
    InputTooShortError, NegativeGainError
)


def test_compression_too_short():
    """Test that very short input raises an error."""
    compressor = Compressor()
    with pytest.raises(InputTooShortError):
        compressor.compress("short text")


def test_compression_min_bytes():
    """Test minimum bytes requirement."""
    compressor = Compressor()
    
    # Create input < 1024 bytes but > 100 tokens
    input_text = "ab " * 200  # 600 bytes, 200 tokens
    
    # Should fail due to min_input_bytes
    with pytest.raises(InputTooShortError):
        compressor.compress(input_text)


def test_compression_min_tokens():
    """Test minimum tokens requirement."""
    config = CompressorConfig(
        min_input_bytes=10,  # Lower byte requirement
        min_input_tokens=500
    )
    compressor = Compressor(config)
    
    # Create input > 10 bytes but < 500 tokens
    input_text = "some short text with few tokens"
    
    # Should fail due to min_input_tokens
    with pytest.raises(InputTooShortError):
        compressor.compress(input_text)


def test_successful_compression():
    """Test successful compression with valid input."""
    compressor = Compressor()
    
    # Create input that meets minimum requirements
    input_text = "This is a long text with many words. " * 100
    
    result = compressor.compress(input_text)
    
    assert result.compressed
    assert result.original_tokens > 0
    assert result.compressed_tokens > 0
    assert result.compression_ratio < 1.0
    assert result.tokens_removed > 0


def test_custom_compression_ratio():
    """Test custom compression ratio."""
    config = CompressorConfig(target_ratio=0.7, min_input_bytes=100)
    compressor = Compressor(config)
    
    input_text = "This is a test text with many words to compress. " * 50
    result = compressor.compress(input_text)
    
    # Should compress less aggressively (keep ~70%)
    assert result.compression_ratio < 0.8

