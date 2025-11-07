# Compression Benchmark Suite

This directory contains comprehensive benchmarks for the compression-prompt library, including A/B test generation for LLM evaluation.

## Structure

```
benches/
├── compression_bench.rs      # Criterion benchmarks for compression techniques
├── ab_test_generator.rs      # A/B test file generator for LLM comparison
└── README.md                 # This file
```

## Benchmark Types

### 1. Dictionary Compression Benchmark
- Tests the dictionary-based compression algorithm
- Measures compression ratio, throughput, and processing time
- Uses real arXiv paper datasets

### 2. Statistical Filtering Benchmark
- Tests statistical filtering at different compression ratios (30%, 50%, 70%)
- Compares actual vs target compression ratios
- Measures performance across different datasets

### 3. Hybrid Compression Benchmark
- Tests combined dictionary + statistical filtering
- Evaluates two-stage compression pipeline
- Optimal for maximum compression with semantic preservation

### 4. Compression Ratio Comparison
- Side-by-side comparison of all techniques
- Statistical analysis of compression effectiveness
- Helps choose optimal technique for specific use cases

## A/B Test Generation

The `ab_test_generator` creates matched pairs of original vs compressed prompts for testing with actual language models:

### Test Types Generated

1. **Dictionary Compression Tests**
   - Full dictionary-based compression
   - Includes dictionary metadata

2. **Statistical Filtering Tests (50%)**
   - Moderate compression
   - Good balance of size reduction and semantic preservation

3. **Statistical Filtering Tests (70%)**
   - Aggressive compression
   - Maximum size reduction

4. **Hybrid Compression Tests**
   - Dictionary followed by statistical filtering
   - Optimal compression ratio

### Output Files

```
benchmarks/ab_tests/
├── ab_test_suite.json           # Complete test suite
├── ab_test_comparison.md        # Human-readable comparison
└── individual_tests/            # Individual test files
    ├── benchmark_100_paper_1_dictionary.json
    ├── benchmark_100_paper_1_statistical_50.json
    └── ...
```

## Running Benchmarks

### Quick Start

```bash
# Linux/Mac
./scripts/run_benchmarks.sh

# Windows (PowerShell)
.\scripts\run_benchmarks.ps1

# Or manually
cd rust
cargo bench
```

### Individual Benchmarks

```bash
# Run only dictionary compression benchmark
cargo bench --bench compression_bench -- dictionary

# Run only statistical filtering benchmark
cargo bench --bench compression_bench -- statistical

# Generate A/B tests only
cargo run --release --bench ab_test_generator
```

## Viewing Results

### Criterion HTML Reports

1. Open `rust/target/criterion/index.html` in a browser
2. Navigate through the interactive reports
3. Compare performance across different datasets

### A/B Test Files

1. Review `benchmarks/ab_tests/ab_test_comparison.md` for summary statistics
2. Use individual test files in `individual_tests/` for LLM evaluation
3. Load test suite JSON for programmatic analysis

## Benchmark Configuration

Configure benchmarks by modifying:

- `CompressorConfig` in `compression_bench.rs`
- `StatisticalFilterConfig` for filtering parameters
- Dataset selection in `load_datasets()` function

## Adding New Benchmarks

1. Create new benchmark function in `compression_bench.rs`:

```rust
fn bench_my_new_technique(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_technique");
    // ... benchmark code
    group.finish();
}
```

2. Add to `criterion_group!`:

```rust
criterion_group!(
    benches,
    bench_dictionary_compression,
    bench_my_new_technique  // Add here
);
```

3. Run: `cargo bench`

## LLM Evaluation Workflow

1. Generate A/B tests: `cargo run --release --bench ab_test_generator`
2. Load test files into your LLM evaluation framework
3. For each test pair:
   - Send original prompt to LLM
   - Send compressed prompt to LLM
   - Compare outputs for semantic equivalence
4. Calculate success metrics:
   - Semantic similarity score
   - Task completion rate
   - Response quality

## Performance Targets

- **Dictionary Compression**: > 20% token reduction on academic text
- **Statistical Filtering (50%)**: ~50% token reduction
- **Hybrid Compression**: > 40% token reduction
- **Throughput**: > 1 MB/s on modern hardware

## Continuous Benchmarking

For CI/CD integration:

```bash
# Run benchmarks without HTML reports
cargo bench --bench compression_bench --no-fail-fast

# Compare with baseline
cargo bench --bench compression_bench -- --baseline main
```

## Troubleshooting

### Missing Dataset Files

If benchmark datasets are not found, the benchmarks will generate synthetic data. To use real datasets:

```bash
cd benchmarks/datasets/prompts
# Ensure benchmark_100_papers.txt and benchmark_200_papers.txt exist
```

### Slow Benchmarks

Criterion runs multiple iterations for statistical significance. To speed up during development:

```bash
# Quick mode (fewer samples)
cargo bench --bench compression_bench -- --quick
```

### Out of Memory

For large datasets, reduce the number of papers processed:

```rust
// In ab_test_generator.rs
.take(10) // Reduce from 20 to 10
```

## Contributing

When adding new compression techniques:

1. Add benchmark in `compression_bench.rs`
2. Add A/B test generation in `ab_test_generator.rs`
3. Update this README with new technique description
4. Run full benchmark suite to establish baseline

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Benchmarking Methodology](../docs/specs/BENCHMARK_METHODOLOGY.md)
- [Compression Algorithm Spec](../docs/specs/COMPRESSION_ALGORITHM.md)

