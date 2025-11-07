#!/bin/bash
# Run compression benchmarks and generate A/B test files

set -e

echo "🔬 Starting Compression Benchmark Suite"
echo "========================================"
echo ""

# Navigate to rust directory
cd "$(dirname "$0")/../rust"

# Clean previous build artifacts
echo "🧹 Cleaning previous builds..."
cargo clean
echo ""

# Build in release mode
echo "🔨 Building in release mode..."
cargo build --release --benches
echo ""

# Run Criterion benchmarks
echo "📊 Running Criterion benchmarks..."
echo "   This will generate HTML reports in target/criterion/"
cargo bench --bench compression_bench
echo ""

# Generate A/B test files
echo "🧪 Generating A/B test files..."
mkdir -p ../benchmarks/ab_tests
cargo bench --bench ab_test_generator --no-run
cargo run --release --bench ab_test_generator
echo ""

# Display results summary
echo "✅ Benchmark suite complete!"
echo ""
echo "📁 Results locations:"
echo "   - Criterion reports: rust/target/criterion/"
echo "   - A/B test files: benchmarks/ab_tests/"
echo ""
echo "📊 View results:"
echo "   - Open rust/target/criterion/index.html in browser"
echo "   - Review benchmarks/ab_tests/ab_test_comparison.md"
echo ""

