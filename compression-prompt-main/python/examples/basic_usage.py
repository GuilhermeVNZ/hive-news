#!/usr/bin/env python3
"""Basic usage example for compression-prompt."""

import sys
from pathlib import Path

# Add parent directory to path for local development
sys.path.insert(0, str(Path(__file__).parent.parent))

from compression_prompt import Compressor, QualityMetrics


def main():
    """Demonstrate basic compression usage."""
    
    # Sample text (needs to be >1024 bytes)
    base_text = """
    Machine Learning is a subset of Artificial Intelligence that focuses on 
    building systems that can learn from data. Deep Learning, a specialized 
    branch of Machine Learning, uses neural networks with multiple layers to 
    process complex patterns in data. Natural Language Processing (NLP) is 
    another important area that enables computers to understand and generate 
    human language. Computer Vision allows machines to interpret visual 
    information from the world. These technologies are transforming industries 
    like healthcare, finance, and autonomous vehicles. Reinforcement Learning 
    enables agents to learn optimal behaviors through trial and error. Transfer 
    Learning allows models to leverage knowledge from one task to improve 
    performance on related tasks. Generative AI creates new content including 
    text, images, and code.
    """
    
    # Repeat to meet minimum size requirement
    text = base_text * 3
    
    print("=" * 70)
    print("COMPRESSION-PROMPT - Python Example")
    print("=" * 70)
    print()
    
    # Create compressor with default settings (50% compression)
    compressor = Compressor()
    
    # Compress
    result = compressor.compress(text)
    
    # Show results
    print("ORIGINAL TEXT:")
    print("-" * 70)
    print(text.strip())
    print()
    
    print("COMPRESSED TEXT:")
    print("-" * 70)
    print(result.compressed)
    print()
    
    print("COMPRESSION STATISTICS:")
    print("-" * 70)
    print(f"Original tokens:   {result.original_tokens}")
    print(f"Compressed tokens: {result.compressed_tokens}")
    print(f"Tokens removed:    {result.tokens_removed}")
    print(f"Compression ratio: {(1.0 - result.compression_ratio) * 100:.1f}%")
    print()
    
    # Calculate quality metrics
    metrics = QualityMetrics.calculate(text, result.compressed)
    
    print("QUALITY METRICS:")
    print("-" * 70)
    print(metrics.format())
    print()
    
    print("=" * 70)
    print("✅ Compression successful!")
    print(f"💰 Saved {result.tokens_removed} tokens ({(1-result.compression_ratio)*100:.1f}%)")
    print(f"🎯 Quality retained: {metrics.overall_score * 100:.1f}%")
    print("=" * 70)


if __name__ == '__main__':
    main()

