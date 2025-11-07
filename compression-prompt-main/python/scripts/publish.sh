#!/bin/bash
# Publish script for compression-prompt Python package

set -e

# Check if we should publish to test or production
if [ "$1" = "test" ]; then
    REPO="testpypi"
    echo "Publishing to TestPyPI..."
elif [ "$1" = "prod" ]; then
    REPO="pypi"
    echo "Publishing to PyPI..."
else
    echo "Usage: $0 [test|prod]"
    echo ""
    echo "  test - Publish to TestPyPI"
    echo "  prod - Publish to PyPI"
    exit 1
fi

# Verify dist/ exists
if [ ! -d "dist" ] || [ -z "$(ls -A dist)" ]; then
    echo "Error: dist/ directory is empty. Run build.sh first."
    exit 1
fi

# Show what will be uploaded
echo ""
echo "Files to upload:"
ls -lh dist/
echo ""

# Confirm
read -p "Continue with upload to $REPO? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

# Upload
if [ "$REPO" = "testpypi" ]; then
    python3 -m twine upload --repository testpypi dist/*
else
    python3 -m twine upload dist/*
fi

echo ""
echo "==================================================================="
echo "Published to $REPO successfully!"
echo "==================================================================="

if [ "$REPO" = "testpypi" ]; then
    echo ""
    echo "Install from TestPyPI:"
    echo "  pip install --index-url https://test.pypi.org/simple/ compression-prompt"
else
    echo ""
    echo "Install from PyPI:"
    echo "  pip install compression-prompt"
fi
echo ""

