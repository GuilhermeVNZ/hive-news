.PHONY: install test lint format clean build

install:
	pip install -e ".[dev]"

test:
	pytest tests/ -v --cov=compression_prompt --cov-report=term-missing

lint:
	ruff check compression_prompt/ tests/
	mypy compression_prompt/

format:
	black compression_prompt/ tests/ examples/
	ruff check --fix compression_prompt/ tests/

clean:
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	rm -rf .ruff_cache/
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

build:
	python -m build

examples:
	@echo "Running basic example..."
	python examples/basic_usage.py
	@echo "\nRunning custom config example..."
	python examples/custom_config.py
	@echo "\nRunning RAG example..."
	python examples/rag_compression.py
	@echo "\nRunning benchmark..."
	python examples/benchmark.py

.DEFAULT_GOAL := help

help:
	@echo "Available targets:"
	@echo "  install   - Install package in development mode"
	@echo "  test      - Run tests with coverage"
	@echo "  lint      - Run linters (ruff, mypy)"
	@echo "  format    - Format code with black and ruff"
	@echo "  clean     - Remove build artifacts"
	@echo "  build     - Build distribution packages"
	@echo "  examples  - Run all examples"

