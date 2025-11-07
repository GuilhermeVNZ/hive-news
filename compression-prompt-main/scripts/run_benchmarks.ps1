# PowerShell script to run compression benchmarks and generate A/B test files

Write-Host "🔬 Starting Compression Benchmark Suite" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Navigate to rust directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location "$scriptDir\..\rust"

# Clean previous build artifacts
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
cargo clean
Write-Host ""

# Build in release mode
Write-Host "🔨 Building in release mode..." -ForegroundColor Yellow
cargo build --release --benches
Write-Host ""

# Run Criterion benchmarks
Write-Host "📊 Running Criterion benchmarks..." -ForegroundColor Yellow
Write-Host "   This will generate HTML reports in target/criterion/" -ForegroundColor Gray
cargo bench --bench compression_bench
Write-Host ""

# Generate A/B test files
Write-Host "🧪 Generating A/B test files..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "..\benchmarks\ab_tests" | Out-Null
cargo run --release --bench ab_test_generator
Write-Host ""

# Display results summary
Write-Host "✅ Benchmark suite complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📁 Results locations:" -ForegroundColor Cyan
Write-Host "   - Criterion reports: rust/target/criterion/" -ForegroundColor Gray
Write-Host "   - A/B test files: benchmarks/ab_tests/" -ForegroundColor Gray
Write-Host ""
Write-Host "📊 View results:" -ForegroundColor Cyan
Write-Host "   - Open rust/target/criterion/index.html in browser" -ForegroundColor Gray
Write-Host "   - Review benchmarks/ab_tests/ab_test_comparison.md" -ForegroundColor Gray
Write-Host ""

