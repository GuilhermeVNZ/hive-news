#!/bin/bash
# Test installation of built package

set -e

echo "==================================================================="
echo "Testing package installation"
echo "==================================================================="
echo ""

# Create temporary virtual environment
VENV_DIR=$(mktemp -d)
echo "Creating virtual environment in $VENV_DIR..."
python3 -m venv "$VENV_DIR"
source "$VENV_DIR/bin/activate"

# Install from dist
echo ""
echo "Installing package from dist/..."
pip install --quiet --upgrade pip
pip install dist/*.whl

# Test import
echo ""
echo "Testing imports..."
python3 -c "from compression_prompt import Compressor, StatisticalFilter, QualityMetrics; print('✅ Core imports OK')"

# Test optional image import
python3 -c "
try:
    from compression_prompt import ImageRenderer
    print('✅ Image rendering available')
except ImportError:
    print('⚠️  Image rendering not available (Pillow not installed)')
"

# Test CLI
echo ""
echo "Testing CLI..."
which compress
compress --help | head -5

# Quick functional test
echo ""
echo "Running quick functional test..."
python3 << 'EOF'
from compression_prompt import Compressor

text = "This is a test. " * 200
compressor = Compressor()
result = compressor.compress(text)

assert result.compression_ratio < 1.0
assert result.tokens_removed > 0
print(f"✅ Functional test OK (compressed {result.tokens_removed} tokens)")
EOF

# Cleanup
echo ""
echo "Cleaning up..."
deactivate
rm -rf "$VENV_DIR"

echo ""
echo "==================================================================="
echo "✅ Package installation test PASSED!"
echo "==================================================================="

