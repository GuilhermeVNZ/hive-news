#!/usr/bin/env python3
"""Benchmark compression performance"""

import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from compression_prompt import (
    Compressor, CompressorConfig, StatisticalFilterConfig, QualityMetrics
)


def benchmark_compression():
    """Benchmark compression with different configurations."""
    
    # Load sample text
    sample_text = """
    Artificial Intelligence (AI) has revolutionized numerous fields, from healthcare 
    to finance, transportation to entertainment. Machine Learning, a subset of AI, 
    enables computers to learn from data without being explicitly programmed. Deep 
    Learning, using neural networks with multiple layers, has achieved remarkable 
    results in image recognition, natural language processing, and game playing.
    
    Natural Language Processing (NLP) focuses on enabling computers to understand 
    and generate human language. Modern NLP systems use transformer architectures 
    like BERT, GPT, and T5, which have demonstrated unprecedented capabilities in 
    tasks such as translation, summarization, question answering, and text generation.
    
    Computer Vision allows machines to interpret visual information. Applications 
    include autonomous vehicles, medical image analysis, facial recognition, and 
    augmented reality. Recent advances in convolutional neural networks (CNNs) and 
    vision transformers have pushed the boundaries of what's possible in visual 
    understanding.
    
    Reinforcement Learning trains agents to make sequential decisions by rewarding 
    desired behaviors. This approach has led to breakthroughs in robotics, game AI 
    (like AlphaGo), and autonomous systems. The combination of deep learning and 
    reinforcement learning has opened new possibilities in complex decision-making 
    scenarios.
    """ * 10  # Repeat to get more data
    
    configs = [
        ("Conservative (70%)", 0.7),
        ("Balanced (50%)", 0.5),
        ("Aggressive (30%)", 0.3),
    ]
    
    print("=" * 70)
    print("COMPRESSION BENCHMARK")
    print("=" * 70)
    print(f"\nInput size: {len(sample_text)} bytes")
    print(f"Estimated tokens: {len(sample_text) // 4}")
    print()
    
    for name, ratio in configs:
        print(f"\n{name}")
        print("-" * 70)
        
        # Configure compressor
        config = CompressorConfig(
            target_ratio=ratio,
            min_input_bytes=100,
            min_input_tokens=10
        )
        filter_config = StatisticalFilterConfig(compression_ratio=ratio)
        compressor = Compressor(config, filter_config)
        
        # Warm up
        _ = compressor.compress(sample_text)
        
        # Benchmark
        iterations = 10
        start = time.time()
        
        results = []
        for _ in range(iterations):
            result = compressor.compress(sample_text)
            results.append(result)
        
        elapsed = time.time() - start
        avg_time = elapsed / iterations
        
        # Get metrics from last result
        result = results[-1]
        metrics = QualityMetrics.calculate(sample_text, result.compressed)
        
        # Calculate throughput
        throughput_mb_s = (len(sample_text) / 1024 / 1024) / avg_time
        
        print(f"Compression:      {(1.0 - result.compression_ratio) * 100:.1f}%")
        print(f"Avg time:         {avg_time * 1000:.2f}ms")
        print(f"Throughput:       {throughput_mb_s:.2f} MB/s")
        print(f"Keyword retention: {metrics.keyword_retention * 100:.1f}%")
        print(f"Entity retention:  {metrics.entity_retention * 100:.1f}%")
        print(f"Overall quality:   {metrics.overall_score * 100:.1f}%")
    
    print("\n" + "=" * 70)


if __name__ == '__main__':
    benchmark_compression()

