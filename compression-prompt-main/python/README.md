# Compression Prompt - Python Implementation

> Fast, intelligent prompt compression for LLMs - Save 50% tokens while maintaining 91% quality

Python port of the Rust implementation. Achieves **50% token reduction** with **91% quality retention** using pure statistical filtering.

## Quick Start

### Installation

```bash
cd python
pip install -e .
```

Or install from source:

```bash
pip install -e ".[dev]"  # With development dependencies
```

### Basic Usage

```python
from compression_prompt import Compressor, CompressorConfig

# Use default configuration (50% compression)
compressor = Compressor()

text = """
Your long text here...
This will be compressed using statistical filtering
to save 50% tokens while maintaining quality.
"""

result = compressor.compress(text)

print(f"Original: {result.original_tokens} tokens")
print(f"Compressed: {result.compressed_tokens} tokens")
print(f"Saved: {result.tokens_removed} tokens ({(1-result.compression_ratio)*100:.1f}%)")
print(f"\nCompressed text:\n{result.compressed}")
```

### Advanced Configuration

```python
from compression_prompt import (
    Compressor, CompressorConfig, 
    StatisticalFilterConfig
)

# Custom compression ratio
config = CompressorConfig(target_ratio=0.7)  # Keep 70% of tokens
filter_config = StatisticalFilterConfig(
    compression_ratio=0.7,
    idf_weight=0.3,
    position_weight=0.2,
    pos_weight=0.2,
    entity_weight=0.2,
    entropy_weight=0.1,
)

compressor = Compressor(config, filter_config)
result = compressor.compress(text)
```

### Quality Metrics

```python
from compression_prompt import QualityMetrics

original = "Your original text..."
compressed = "Your compressed text..."

metrics = QualityMetrics.calculate(original, compressed)
print(metrics.format())
```

Output:
```
Quality Metrics:
- Keyword Retention: 92.0%
- Entity Retention: 89.5%
- Vocabulary Ratio: 78.3%
- Info Density: 0.845
- Overall Score: 89.2%
```

### Command Line Usage

```bash
# Compress file to stdout
compress input.txt

# Conservative compression (70%)
compress -r 0.7 input.txt

# Aggressive compression (30%)
compress -r 0.3 input.txt

# Show statistics
compress -s input.txt

# Save to file
compress -o output.txt input.txt

# Read from stdin
cat input.txt | compress
```

## Features

- ✅ **Zero Dependencies**: Pure Python implementation, no external libraries required
- ✅ **Fast**: Optimized statistical filtering
- ✅ **Multilingual**: Supports 10+ languages (EN, ES, PT, FR, DE, IT, RU, ZH, JA, AR, HI)
- ✅ **Smart Filtering**: Preserves code blocks, JSON, paths, identifiers
- ✅ **Contextual**: Intelligent stopword handling based on context
- ✅ **Customizable**: Fine-tune weights and parameters for your use case

## Configuration Options

### CompressorConfig

```python
CompressorConfig(
    target_ratio=0.5,        # Target compression ratio (0.0-1.0)
    min_input_tokens=100,    # Minimum tokens to attempt compression
    min_input_bytes=1024     # Minimum bytes to attempt compression
)
```

### StatisticalFilterConfig

```python
StatisticalFilterConfig(
    compression_ratio=0.5,              # Keep 50% of tokens
    
    # Feature weights (sum should be ~1.0)
    idf_weight=0.3,                     # Inverse document frequency
    position_weight=0.2,                # Position in text (start/end important)
    pos_weight=0.2,                     # Part-of-speech heuristics
    entity_weight=0.2,                  # Named entity detection
    entropy_weight=0.1,                 # Local vocabulary diversity
    
    # Protection features
    enable_protection_masks=True,       # Protect code/JSON/paths
    enable_contextual_stopwords=True,   # Smart stopword filtering
    preserve_negations=True,            # Keep "not", "never", etc.
    preserve_comparators=True,          # Keep ">=", "!=", etc.
    
    # Domain-specific
    domain_terms=["YourTerm"],          # Always preserve these terms
    min_gap_between_critical=3          # Fill gaps between important tokens
)
```

## Examples

### Example 1: RAG System Context Compression

```python
from compression_prompt import Compressor

# Compress retrieved context before sending to LLM
retrieved_docs = get_documents(query)
context = "\n\n".join(doc.text for doc in retrieved_docs)

compressor = Compressor()
result = compressor.compress(context)

# Save 50% tokens while maintaining quality
prompt = f"Context: {result.compressed}\n\nQuestion: {user_question}"
response = llm.generate(prompt)
```

### Example 2: Custom Domain Terms

```python
from compression_prompt import StatisticalFilterConfig, Compressor

# Preserve domain-specific terms
filter_config = StatisticalFilterConfig(
    domain_terms=["TensorFlow", "PyTorch", "CUDA", "GPU"]
)

compressor = Compressor(filter_config=filter_config)
result = compressor.compress(technical_text)
```

### Example 3: Aggressive Compression

```python
from compression_prompt import CompressorConfig, StatisticalFilterConfig, Compressor

# 70% compression (keep only 30% of tokens)
config = CompressorConfig(target_ratio=0.3, min_input_tokens=50)
filter_config = StatisticalFilterConfig(compression_ratio=0.3)

compressor = Compressor(config, filter_config)
result = compressor.compress(text)

print(f"Compressed to {result.compressed_tokens} tokens (from {result.original_tokens})")
```

## Performance Characteristics

| Compression | Token Savings | Keyword Retention | Entity Retention | Use Case |
|-------------|--------------|-------------------|------------------|----------|
| **50% (default)** ⭐ | **50%** | **92.0%** | **89.5%** | Best balance |
| 70% (conservative) | 30% | 99.2% | 98.4% | High precision |
| 30% (aggressive) | 70% | 72.4% | 71.5% | Maximum savings |

## Development

### Running Tests

```bash
pytest tests/
```

### Code Formatting

```bash
black compression_prompt/
```

### Type Checking

```bash
mypy compression_prompt/
```

## Differences from Rust Version

The Python implementation maintains feature parity with the Rust version:

- ✅ Same statistical filtering algorithm
- ✅ Same configuration options
- ✅ Same quality metrics
- ✅ CLI tool with identical interface
- ⏳ Image output (optional, requires Pillow)

Performance differences:
- **Rust**: ~0.16ms average, 10.58 MB/s throughput
- **Python**: ~1-5ms average (still very fast for most use cases)

## License

MIT

## See Also

- [Rust Implementation](../rust/) - Original high-performance implementation
- [Main README](../README.md) - Project overview and benchmarks
- [Architecture](../docs/ARCHITECTURE.md) - Technical details

