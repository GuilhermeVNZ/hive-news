#!/usr/bin/env python3
"""
Full benchmark suite matching Rust implementation.

Tests compression across multiple files and configurations.
"""

import sys
import time
import json
from pathlib import Path
from typing import List, Dict, Any

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from compression_prompt import (
    Compressor, CompressorConfig, StatisticalFilterConfig,
    QualityMetrics
)


def load_test_files(directory: Path) -> List[str]:
    """Load all text files from a directory."""
    texts = []
    
    if not directory.exists():
        print(f"Warning: Directory {directory} not found, using sample text")
        # Generate sample text for testing
        sample = """
        Machine Learning is revolutionizing technology. Deep Learning processes 
        complex patterns. Natural Language Processing understands human language.
        Computer Vision interprets visual data. These AI technologies transform industries.
        """ * 100
        return [sample] * 20
    
    for file_path in directory.glob("*.txt"):
        try:
            text = file_path.read_text(encoding='utf-8')
            if len(text) > 1000:  # Only include substantial files
                texts.append(text)
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
    
    return texts


def benchmark_configuration(texts: List[str], ratio: float, name: str) -> Dict[str, Any]:
    """Benchmark a specific compression configuration."""
    
    print(f"\nBenchmarking: {name} (ratio={ratio})")
    print("-" * 70)
    
    config = CompressorConfig(
        target_ratio=ratio,
        min_input_bytes=100,
        min_input_tokens=10
    )
    filter_config = StatisticalFilterConfig(compression_ratio=ratio)
    compressor = Compressor(config, filter_config)
    
    total_original_tokens = 0
    total_compressed_tokens = 0
    total_time = 0.0
    total_bytes = 0
    
    keyword_retentions = []
    entity_retentions = []
    overall_scores = []
    
    successful = 0
    
    for i, text in enumerate(texts, 1):
        try:
            start = time.time()
            result = compressor.compress(text)
            elapsed = time.time() - start
            
            total_original_tokens += result.original_tokens
            total_compressed_tokens += result.compressed_tokens
            total_time += elapsed
            total_bytes += len(text)
            
            # Calculate quality
            metrics = QualityMetrics.calculate(text, result.compressed)
            keyword_retentions.append(metrics.keyword_retention)
            entity_retentions.append(metrics.entity_retention)
            overall_scores.append(metrics.overall_score)
            
            successful += 1
            
            if i % 10 == 0:
                print(f"  Processed {i}/{len(texts)} files...")
        
        except Exception as e:
            print(f"  Error processing file {i}: {e}")
    
    # Calculate averages
    avg_keyword = sum(keyword_retentions) / len(keyword_retentions) if keyword_retentions else 0
    avg_entity = sum(entity_retentions) / len(entity_retentions) if entity_retentions else 0
    avg_overall = sum(overall_scores) / len(overall_scores) if overall_scores else 0
    
    actual_ratio = total_compressed_tokens / total_original_tokens if total_original_tokens > 0 else 0
    tokens_saved = total_original_tokens - total_compressed_tokens
    
    throughput_mb_s = (total_bytes / 1024 / 1024) / total_time if total_time > 0 else 0
    avg_time_ms = (total_time / successful * 1000) if successful > 0 else 0
    
    results = {
        "name": name,
        "target_ratio": ratio,
        "files_processed": successful,
        "original_tokens": total_original_tokens,
        "compressed_tokens": total_compressed_tokens,
        "tokens_saved": tokens_saved,
        "actual_compression_ratio": actual_ratio,
        "compression_percent": (1.0 - actual_ratio) * 100,
        "total_time_s": total_time,
        "avg_time_ms": avg_time_ms,
        "throughput_mb_s": throughput_mb_s,
        "keyword_retention": avg_keyword * 100,
        "entity_retention": avg_entity * 100,
        "overall_quality": avg_overall * 100,
    }
    
    # Print summary
    print(f"\n  Files processed:      {successful}")
    print(f"  Original tokens:      {total_original_tokens:,}")
    print(f"  Compressed tokens:    {total_compressed_tokens:,}")
    print(f"  Tokens saved:         {tokens_saved:,} ({(1.0 - actual_ratio) * 100:.1f}%)")
    print(f"  Avg time:             {avg_time_ms:.2f}ms")
    print(f"  Throughput:           {throughput_mb_s:.2f} MB/s")
    print(f"  Keyword retention:    {avg_keyword * 100:.1f}%")
    print(f"  Entity retention:     {avg_entity * 100:.1f}%")
    print(f"  Overall quality:      {avg_overall * 100:.1f}%")
    
    return results


def main():
    """Run full benchmark suite."""
    
    print("=" * 70)
    print("COMPRESSION-PROMPT - Python Benchmark Suite")
    print("=" * 70)
    
    # Try to load test files, otherwise use generated content
    test_dir = Path(__file__).parent.parent.parent / "benchmarks" / "inputs"
    texts = load_test_files(test_dir)
    
    if not texts:
        print("No test files found, generating sample data...")
        sample = "This is a test text for compression. " * 1000
        texts = [sample] * 20
    
    print(f"\nLoaded {len(texts)} test files")
    total_size = sum(len(t) for t in texts)
    print(f"Total size: {total_size / 1024:.1f} KB")
    
    # Benchmark different configurations
    configurations = [
        (0.7, "Conservative (70%)"),
        (0.5, "Balanced (50%)"),
        (0.3, "Aggressive (30%)"),
    ]
    
    all_results = []
    
    for ratio, name in configurations:
        results = benchmark_configuration(texts, ratio, name)
        all_results.append(results)
    
    # Save results
    output_file = Path("benchmark_results.json")
    with open(output_file, 'w') as f:
        json.dump(all_results, f, indent=2)
    
    print(f"\n{'=' * 70}")
    print(f"Results saved to: {output_file}")
    print(f"{'=' * 70}\n")
    
    # Print comparison table
    print("\nCOMPARISON TABLE:")
    print("-" * 70)
    print(f"{'Config':<20} {'Compression':<15} {'Quality':<15} {'Speed':<15}")
    print("-" * 70)
    
    for result in all_results:
        print(f"{result['name']:<20} "
              f"{result['compression_percent']:.1f}%{'':<11} "
              f"{result['overall_quality']:.1f}%{'':<11} "
              f"{result['avg_time_ms']:.2f}ms")
    
    print("-" * 70)


if __name__ == '__main__':
    main()

