#!/bin/bash
# Convert arXiv PDFs to Markdown using transmutation

set -e

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PDF_DIR="$PROJECT_ROOT/benchmarks/datasets/arxiv_pdfs"
MD_DIR="$PROJECT_ROOT/benchmarks/datasets/arxiv_markdown"
TRANSMUTATION_DIR="$PROJECT_ROOT/../transmutation"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Create output directory
mkdir -p "$MD_DIR"

# Check if transmutation exists
if [ ! -d "$TRANSMUTATION_DIR" ]; then
    log_error "Transmutation not found at: $TRANSMUTATION_DIR"
    log_info "Please ensure transmutation is installed"
    exit 1
fi

# Check if PDFs exist
if [ ! -d "$PDF_DIR" ]; then
    log_error "PDF directory not found: $PDF_DIR"
    exit 1
fi

PDF_COUNT=$(find "$PDF_DIR" -name "*.pdf" | wc -l)
if [ "$PDF_COUNT" -eq 0 ]; then
    log_error "No PDF files found in: $PDF_DIR"
    exit 1
fi

log_info "=============================================="
log_info "arXiv PDF to Markdown Conversion"
log_info "=============================================="
log_info "PDFs directory: $PDF_DIR"
log_info "Markdown output: $MD_DIR"
log_info "Transmutation: $TRANSMUTATION_DIR"
log_info "Total PDFs: $PDF_COUNT"
echo ""

# Convert each PDF
SUCCESS=0
SKIP=0
FAIL=0
COUNTER=0

for pdf_file in "$PDF_DIR"/*.pdf; do
    COUNTER=$((COUNTER + 1))
    
    # Get filename without extension
    filename=$(basename "$pdf_file" .pdf)
    md_file="$MD_DIR/${filename}.md"
    
    # Skip if already converted
    if [ -f "$md_file" ]; then
        echo -e "${BLUE}[$COUNTER/$PDF_COUNT]${NC} Skipping $filename (already exists)"
        SKIP=$((SKIP + 1))
        continue
    fi
    
    echo -e "${BLUE}[$COUNTER/$PDF_COUNT]${NC} Converting: $filename"
    
    # Convert using transmutation binary
    if "$TRANSMUTATION_DIR/target/release/transmutation" convert "$pdf_file" \
        -o "$md_file" \
        -f markdown \
        --optimize-llm 2>&1 | tail -1 | grep -q "Successfully"; then
        
        # Check if output file was created and has content
        if [ -f "$md_file" ] && [ -s "$md_file" ]; then
            file_size=$(du -h "$md_file" | cut -f1)
            echo -e "  ${GREEN}✓${NC} Success ($file_size)"
            SUCCESS=$((SUCCESS + 1))
        else
            echo -e "  ${RED}✗${NC} Failed (empty output)"
            rm -f "$md_file"
            FAIL=$((FAIL + 1))
        fi
    else
        echo -e "  ${RED}✗${NC} Conversion failed"
        FAIL=$((FAIL + 1))
    fi
    
    # Small delay to avoid overwhelming the system
    sleep 0.5
done

# Summary
echo ""
log_info "=============================================="
log_info "Conversion Summary"
log_info "=============================================="
echo -e "  ${GREEN}✓${NC} Converted: $SUCCESS"
echo -e "  ${BLUE}⏭${NC}  Skipped: $SKIP"
echo -e "  ${RED}✗${NC} Failed: $FAIL"
echo -e "  ${YELLOW}📊${NC} Total: $PDF_COUNT"
echo ""

# List converted files
MD_COUNT=$(find "$MD_DIR" -name "*.md" | wc -l)
MD_SIZE=$(du -sh "$MD_DIR" 2>/dev/null | cut -f1 || echo "0")

log_info "Output directory contains $MD_COUNT Markdown files ($MD_SIZE)"
log_info "Location: $MD_DIR"

if [ $FAIL -gt 0 ]; then
    log_warn "Some conversions failed. Check transmutation output above."
fi

echo ""
log_info "Done! ✨"

