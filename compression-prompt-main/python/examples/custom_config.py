#!/usr/bin/env python3
"""Example showing custom configuration options."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from compression_prompt import (
    Compressor, CompressorConfig, StatisticalFilterConfig, QualityMetrics
)


def compress_with_config(text: str, ratio: float, name: str):
    """Compress text with given ratio and display results."""
    
    config = CompressorConfig(
        target_ratio=ratio,
        min_input_bytes=100,
        min_input_tokens=10
    )
    
    filter_config = StatisticalFilterConfig(compression_ratio=ratio)
    
    compressor = Compressor(config, filter_config)
    result = compressor.compress(text)
    metrics = QualityMetrics.calculate(text, result.compressed)
    
    print(f"\n{name}:")
    print("-" * 70)
    print(f"Compression: {(1.0 - result.compression_ratio) * 100:.1f}%")
    print(f"Quality: {metrics.overall_score * 100:.1f}%")
    print(f"Compressed: {result.compressed}")


def main():
    """Demonstrate different compression configurations."""
    
    base_text = """
    The quick brown fox jumps over the lazy dog. This sentence contains 
    every letter of the alphabet. Natural Language Processing uses 
    statistical methods to analyze text. Machine Learning models can 
    be trained on large datasets to understand patterns. Deep Learning 
    architectures process information through multiple layers. Computer 
    Vision interprets visual data from images and videos. Reinforcement 
    Learning optimizes sequential decision-making processes.
    """
    
    # Repeat to meet minimum size
    text = base_text * 5
    
    print("=" * 70)
    print("CUSTOM CONFIGURATION EXAMPLES")
    print("=" * 70)
    
    print("\nORIGINAL TEXT:")
    print(text.strip())
    
    # Conservative (70% kept)
    compress_with_config(text, 0.7, "CONSERVATIVE (70% kept, 30% removed)")
    
    # Balanced (50% kept) - DEFAULT
    compress_with_config(text, 0.5, "BALANCED (50% kept, 50% removed) ⭐")
    
    # Aggressive (30% kept)
    compress_with_config(text, 0.3, "AGGRESSIVE (30% kept, 70% removed)")
    
    print("\n" + "=" * 70)
    
    # Custom domain terms example
    print("\n\nCUSTOM DOMAIN TERMS EXAMPLE:")
    print("-" * 70)
    
    tech_base = "Use TensorFlow and PyTorch for deep learning on GPU with CUDA. " \
                "These frameworks provide optimized implementations for neural networks. "
    tech_text = tech_base * 20  # Make it long enough
    
    filter_config = StatisticalFilterConfig(
        compression_ratio=0.5,
        domain_terms=["TensorFlow", "PyTorch", "CUDA", "GPU"]
    )
    
    config = CompressorConfig(min_input_bytes=100, min_input_tokens=10)
    compressor = Compressor(config, filter_config)
    result = compressor.compress(tech_text)
    
    print(f"Original:   {tech_text}")
    print(f"Compressed: {result.compressed}")
    print("Note: Domain terms are preserved!")
    
    print("\n" + "=" * 70)


if __name__ == '__main__':
    main()

