#!/bin/bash
# Benchmark script for arXiv paper dataset compression
# This script automates the complete benchmarking pipeline

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_ROOT/benchmarks"

# Configuration
PAPER_COUNTS=(100 200 500 1000)
ARXIV_CATEGORIES=("cs.CL" "cs.AI" "cs.LG")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create directory structure
setup_directories() {
    log_info "Setting up benchmark directories..."
    mkdir -p "$BENCHMARK_DIR/datasets/arxiv_pdfs"
    mkdir -p "$BENCHMARK_DIR/datasets/arxiv_markdown"
    mkdir -p "$BENCHMARK_DIR/datasets/prompts"
    mkdir -p "$BENCHMARK_DIR/datasets/metadata"
    mkdir -p "$BENCHMARK_DIR/results/compression"
    mkdir -p "$BENCHMARK_DIR/results/llm_validation"
}

# Step 1: Download papers from arXiv
download_papers() {
    local count=$1
    log_info "Downloading $count papers from arXiv..."
    
    # TODO: Implement using arxiv Python package
    # For now, placeholder
    log_warn "Paper download not implemented yet"
    log_info "Manual download required from arXiv.org"
    log_info "Categories: ${ARXIV_CATEGORIES[*]}"
    log_info "Target count: $count papers"
}

# Step 2: Convert PDFs to Markdown using transmutation
convert_to_markdown() {
    log_info "Converting PDFs to Markdown using transmutation..."
    
    local pdf_dir="$BENCHMARK_DIR/datasets/arxiv_pdfs"
    local md_dir="$BENCHMARK_DIR/datasets/arxiv_markdown"
    
    if [ ! -d "$PROJECT_ROOT/../transmutation" ]; then
        log_error "Transmutation not found at $PROJECT_ROOT/../transmutation"
        log_info "Please ensure transmutation is installed"
        return 1
    fi
    
    local count=0
    for pdf in "$pdf_dir"/*.pdf; do
        if [ -f "$pdf" ]; then
            local basename=$(basename "$pdf" .pdf)
            local output="$md_dir/${basename}.md"
            
            log_info "Converting: $basename"
            
            # Run transmutation
            (cd "$PROJECT_ROOT/../transmutation" && \
             cargo run --release -- convert "$pdf" \
                --output "$output" \
                --format markdown)
            
            count=$((count + 1))
        fi
    done
    
    log_info "Converted $count papers to Markdown"
}

# Step 3: Concatenate papers into single prompts
concatenate_papers() {
    local count=$1
    log_info "Concatenating $count papers into single prompt..."
    
    local md_dir="$BENCHMARK_DIR/datasets/arxiv_markdown"
    local output="$BENCHMARK_DIR/datasets/prompts/benchmark_${count}_papers.txt"
    
    # Get first N markdown files
    local files=($(ls "$md_dir"/*.md | head -n "$count"))
    
    # Concatenate with separators
    > "$output"
    for file in "${files[@]}"; do
        cat "$file" >> "$output"
        echo -e "\n\n---PAPER_SEPARATOR---\n\n" >> "$output"
    done
    
    log_info "Created: $output"
    log_info "Size: $(du -h "$output" | cut -f1)"
}

# Step 4: Run compression benchmark
compress_dataset() {
    local count=$1
    log_info "Running compression on $count papers dataset..."
    
    local input="$BENCHMARK_DIR/datasets/prompts/benchmark_${count}_papers.txt"
    local output="$BENCHMARK_DIR/results/compression/${count}_papers_result.json"
    
    if [ ! -f "$input" ]; then
        log_error "Input file not found: $input"
        return 1
    fi
    
    # Run compression (requires compression-core CLI)
    log_info "Compressing: $input"
    
    local start_time=$(date +%s)
    
    # TODO: Implement CLI tool for compression
    # For now, placeholder
    log_warn "Compression CLI not implemented yet"
    log_info "Would compress: $input → $output"
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_info "Compression completed in ${duration}s"
}

# Step 5: Validate with LLMs (placeholder)
validate_with_llms() {
    local count=$1
    log_info "Validating compressed prompts with LLMs..."
    
    log_warn "LLM validation not implemented yet"
    log_info "Manual validation required with Claude/GPT/Gemini"
    log_info "See docs/specs/BENCHMARK_DATASET_ARXIV.md for test queries"
}

# Main benchmark pipeline
run_benchmark() {
    local count=$1
    
    log_info "========================================"
    log_info "Running benchmark for $count papers"
    log_info "========================================"
    
    # Check if papers already concatenated
    local prompt_file="$BENCHMARK_DIR/datasets/prompts/benchmark_${count}_papers.txt"
    if [ ! -f "$prompt_file" ]; then
        concatenate_papers "$count"
    else
        log_info "Using existing concatenated file: $prompt_file"
    fi
    
    # Run compression
    compress_dataset "$count"
    
    # Validate (optional)
    # validate_with_llms "$count"
    
    log_info "Benchmark for $count papers completed"
}

# Generate summary report
generate_report() {
    log_info "Generating benchmark summary report..."
    
    local report="$BENCHMARK_DIR/results/BENCHMARK_REPORT.md"
    
    cat > "$report" << EOF
# arXiv Benchmark Results

Generated: $(date)

## Dataset Sizes

EOF
    
    for count in "${PAPER_COUNTS[@]}"; do
        local prompt_file="$BENCHMARK_DIR/datasets/prompts/benchmark_${count}_papers.txt"
        if [ -f "$prompt_file" ]; then
            local size=$(du -h "$prompt_file" | cut -f1)
            local lines=$(wc -l < "$prompt_file")
            echo "- **${count} papers**: ${size}, ${lines} lines" >> "$report"
        fi
    done
    
    cat >> "$report" << EOF

## Compression Results

[TODO: Add compression results once benchmarks complete]

## LLM Validation

[TODO: Add A/B test results]

EOF
    
    log_info "Report generated: $report"
}

# Main execution
main() {
    log_info "arXiv Compression Benchmark Pipeline"
    log_info "====================================="
    
    setup_directories
    
    # Parse arguments
    if [ $# -eq 0 ]; then
        log_info "No paper count specified, running all sizes: ${PAPER_COUNTS[*]}"
        
        for count in "${PAPER_COUNTS[@]}"; do
            run_benchmark "$count"
        done
    else
        local count=$1
        run_benchmark "$count"
    fi
    
    generate_report
    
    log_info "====================================="
    log_info "Benchmark pipeline completed!"
    log_info "Results in: $BENCHMARK_DIR/results/"
}

# Show usage
usage() {
    echo "Usage: $0 [paper_count]"
    echo ""
    echo "Arguments:"
    echo "  paper_count   Number of papers to benchmark (100, 200, 500, 1000)"
    echo "                If omitted, runs all sizes"
    echo ""
    echo "Examples:"
    echo "  $0           # Run all benchmark sizes"
    echo "  $0 100       # Run only 100-paper benchmark"
    echo ""
    echo "Prerequisites:"
    echo "  1. Download arXiv PDFs to benchmarks/datasets/arxiv_pdfs/"
    echo "  2. Install transmutation for PDF→Markdown conversion"
    echo "  3. Build compression-core library"
}

# Handle --help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    usage
    exit 0
fi

main "$@"

