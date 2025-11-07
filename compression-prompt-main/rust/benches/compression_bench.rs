//! Benchmark suite for compression techniques
//!
//! Tests dictionary compression, statistical filtering, and hybrid approaches
//! Generates A/B test results for comparison with larger models

use compression_prompt::compressor::{Compressor, CompressorConfig};
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::fs;

/// Load benchmark datasets
fn load_datasets() -> Vec<(String, String)> {
    let mut datasets = Vec::new();

    // Try to load benchmark files
    let files = vec![
        (
            "100_papers",
            "../benchmarks/datasets/prompts/benchmark_100_papers.txt",
        ),
        (
            "200_papers",
            "../benchmarks/datasets/prompts/benchmark_200_papers.txt",
        ),
    ];

    for (name, path) in files {
        if let Ok(content) = fs::read_to_string(path) {
            datasets.push((name.to_string(), content));
        }
    }

    // Fallback: create synthetic dataset
    if datasets.is_empty() {
        let synthetic = generate_synthetic_paper();
        datasets.push(("synthetic".to_string(), synthetic));
    }

    datasets
}

/// Generate synthetic academic paper for testing
fn generate_synthetic_paper() -> String {
    let mut paper = String::new();

    paper.push_str("# A Survey of Large Language Models\n\n");
    paper.push_str("## Abstract\n\n");
    paper.push_str("Large language models (LLMs) have demonstrated remarkable capabilities across various natural language processing tasks. ");
    paper.push_str("This paper surveys recent advances in LLMs, including transformer architectures, pre-training methods, and fine-tuning strategies. ");
    paper.push_str("We analyze the performance characteristics of leading models and discuss future research directions.\n\n");

    for section in 1..=5 {
        paper.push_str(&format!("## Section {}\n\n", section));
        for _ in 0..10 {
            paper.push_str("Transformer models utilize self-attention mechanisms to process sequential data efficiently. ");
            paper.push_str("The multi-head attention allows the model to focus on different aspects of the input simultaneously. ");
            paper.push_str("Pre-training on large corpora enables these models to learn rich representations of language. ");
            paper.push_str("Fine-tuning on task-specific data further improves performance on downstream applications.\n\n");
        }
    }

    paper
}

/// Benchmark dictionary compression
fn bench_dictionary_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("dictionary_compression");
    let config = CompressorConfig::default();
    let compressor = Compressor::new(config);

    for (name, dataset) in load_datasets() {
        let size = dataset.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("dict", &name), &dataset, |b, data| {
            b.iter(|| {
                let _ = compressor.compress(black_box(data));
            });
        });
    }

    group.finish();
}

/// Benchmark statistical filtering
fn bench_statistical_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistical_filtering");

    let ratios = vec![0.3, 0.5, 0.7];

    for (name, dataset) in load_datasets() {
        let size = dataset.len();
        group.throughput(Throughput::Bytes(size as u64));

        for ratio in &ratios {
            let config = StatisticalFilterConfig {
                compression_ratio: *ratio,
                ..Default::default()
            };
            let filter = StatisticalFilter::new(config);

            group.bench_with_input(
                BenchmarkId::new(format!("stat_{:.0}%", ratio * 100.0), &name),
                &dataset,
                |b, data| {
                    b.iter(|| {
                        let _ = filter.compress(black_box(data));
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark hybrid compression (dictionary + statistical)
fn bench_hybrid_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_compression");

    let dict_config = CompressorConfig::default();
    let dict_compressor = Compressor::new(dict_config);

    let stat_config = StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    };
    let stat_filter = StatisticalFilter::new(stat_config);

    for (name, dataset) in load_datasets() {
        let size = dataset.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("hybrid", &name), &dataset, |b, data| {
            b.iter(|| {
                // First apply dictionary compression
                if let Ok(dict_result) = dict_compressor.compress(black_box(data)) {
                    // Then apply statistical filtering
                    let _ = stat_filter.compress(&dict_result.compressed);
                }
            });
        });
    }

    group.finish();
}

/// Compare compression ratios achieved by different methods
fn bench_compression_ratios(c: &mut Criterion) {
    let group = c.benchmark_group("compression_ratios");

    for (name, dataset) in load_datasets() {
        let original_tokens = dataset.split_whitespace().count();

        // Dictionary compression
        let dict_config = CompressorConfig::default();
        let dict_compressor = Compressor::new(dict_config);

        if let Ok(result) = dict_compressor.compress(&dataset) {
            let ratio = result.compression_ratio;
            println!(
                "Dataset '{}' - Dictionary: {:.3} compression ratio ({:.1}% savings)",
                name,
                ratio,
                (1.0 - ratio) * 100.0
            );
        }

        // Statistical filtering
        for target_ratio in &[0.3, 0.5, 0.7] {
            let stat_config = StatisticalFilterConfig {
                compression_ratio: *target_ratio,
                ..Default::default()
            };
            let stat_filter = StatisticalFilter::new(stat_config);
            let compressed = stat_filter.compress(&dataset);
            let compressed_tokens = compressed.split_whitespace().count();
            let actual_ratio = compressed_tokens as f64 / original_tokens as f64;

            println!(
                "Dataset '{}' - Statistical {:.0}%: {:.3} actual ratio",
                name,
                target_ratio * 100.0,
                actual_ratio
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dictionary_compression,
    bench_statistical_filtering,
    bench_hybrid_compression,
    bench_compression_ratios
);
criterion_main!(benches);
