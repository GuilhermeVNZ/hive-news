#!/bin/bash
# Build script for compression-prompt Python package

set -e

echo "==================================================================="
echo "Building compression-prompt Python package"
echo "==================================================================="
echo ""

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf build/ dist/ *.egg-info
find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
find . -type f -name "*.pyc" -delete 2>/dev/null || true

# Run tests
echo ""
echo "Running tests..."
python3 -m pytest tests/ -v || {
    echo "Tests failed! Aborting build."
    exit 1
}

# Build package
echo ""
echo "Building package..."
python3 -m build

# Check package
echo ""
echo "Checking package..."
python3 -m twine check dist/*

echo ""
echo "==================================================================="
echo "Build complete! Artifacts in dist/"
echo "==================================================================="
ls -lh dist/

echo ""
echo "To publish to TestPyPI:"
echo "  python3 -m twine upload --repository testpypi dist/*"
echo ""
echo "To publish to PyPI:"
echo "  python3 -m twine upload dist/*"
echo ""

