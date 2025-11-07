#!/bin/bash
# Monitor download progress

PDF_DIR="/mnt/f/Node/hivellm/compression-prompt/benchmarks/datasets/arxiv_pdfs"
TARGET=108

echo "📊 Monitoring arXiv PDF downloads..."
echo "🎯 Target: $TARGET papers"
echo ""

while true; do
    if [ -d "$PDF_DIR" ]; then
        COUNT=$(ls -1 "$PDF_DIR"/*.pdf 2>/dev/null | wc -l)
        SIZE=$(du -sh "$PDF_DIR" 2>/dev/null | cut -f1)
        PERCENT=$((COUNT * 100 / TARGET))
        
        # Progress bar
        BAR_LENGTH=50
        FILLED=$((COUNT * BAR_LENGTH / TARGET))
        BAR=$(printf "%${FILLED}s" | tr ' ' '█')
        EMPTY=$(printf "%$((BAR_LENGTH - FILLED))s" | tr ' ' '░')
        
        echo -ne "\r📥 Progress: [$BAR$EMPTY] $COUNT/$TARGET ($PERCENT%) - $SIZE   "
        
        if [ $COUNT -ge $TARGET ]; then
            echo ""
            echo "✅ Download complete!"
            break
        fi
    fi
    
    sleep 5
done

echo ""
echo "📁 Papers downloaded to: $PDF_DIR"
ls -lh "$PDF_DIR" | tail -5

