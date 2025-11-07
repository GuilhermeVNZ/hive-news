# Examples

Practical examples of using compression-prompt in different scenarios.

## Basic Statistical Compression (Recommended)

The **statistical_50pct** method is the recommended default for production use.

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::tokenizer::MockTokenizer;

fn main() {
    let text = "Your long prompt text here...";
    
    // Use recommended defaults (50% compression, 89% quality)
    let config = StatisticalFilterConfig::default();
    let filter = StatisticalFilter::new(config);
    let tokenizer = MockTokenizer;
    
    // Compress
    let compressed = filter.compress(&text, &tokenizer);
    
    // Results
    let original_tokens = tokenizer.count_tokens(&text);
    let compressed_tokens = tokenizer.count_tokens(&compressed);
    let ratio = compressed_tokens as f32 / original_tokens as f32;
    
    println!("Original: {} tokens", original_tokens);
    println!("Compressed: {} tokens ({:.1}% size)", compressed_tokens, ratio * 100.0);
    println!("Savings: {:.1}%", (1.0 - ratio) * 100.0);
}
```

**Expected Results:**
- Compression: ~50%
- Quality: ~89%
- Speed: <0.2ms

## Custom Compression Levels

### Aggressive (70% savings, 71% quality)

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.3,  // Keep only 30% of tokens
    ..Default::default()
};
let filter = StatisticalFilter::new(config);
```

**Use when:**
- Maximum token savings needed
- Quality loss is acceptable
- Triaging/filtering content

### Conservative (30% savings, 96% quality)

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.7,  // Keep 70% of tokens
    ..Default::default()
};
let filter = StatisticalFilter::new(config);
```

**Use when:**
- High precision required
- Technical documentation
- Scientific papers
- Legal/medical content

## RAG System Integration

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::tokenizer::MockTokenizer;

fn compress_rag_context(chunks: Vec<String>) -> String {
    let config = StatisticalFilterConfig::default(); // 50% compression
    let filter = StatisticalFilter::new(config);
    let tokenizer = MockTokenizer;
    
    // Compress each retrieved chunk
    let compressed_chunks: Vec<String> = chunks
        .iter()
        .map(|chunk| filter.compress(chunk, &tokenizer))
        .collect();
    
    // Join compressed chunks
    compressed_chunks.join("\n\n")
}

// Usage
let retrieved = vec![
    "First document chunk...".to_string(),
    "Second document chunk...".to_string(),
];

let compressed_context = compress_rag_context(retrieved);
let prompt = format!("Context: {}\n\nQuestion: What is...?", compressed_context);
// Send to LLM - now 50% smaller!
```

## Batch Processing

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::tokenizer::MockTokenizer;
use std::fs;

fn batch_compress_files(input_dir: &str, output_dir: &str) -> std::io::Result<()> {
    let config = StatisticalFilterConfig::default();
    let filter = StatisticalFilter::new(config);
    let tokenizer = MockTokenizer;
    
    fs::create_dir_all(output_dir)?;
    
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let text = fs::read_to_string(&path)?;
            let compressed = filter.compress(&text, &tokenizer);
            
            let output_path = format!("{}/{}", output_dir, path.file_name().unwrap().to_str().unwrap());
            fs::write(output_path, compressed)?;
        }
    }
    
    Ok(())
}
```

## Quality Metrics

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::quality_metrics::QualityMetrics;
use compression_prompt::tokenizer::MockTokenizer;

fn compress_with_metrics(text: &str) {
    let config = StatisticalFilterConfig::default();
    let filter = StatisticalFilter::new(config);
    let tokenizer = MockTokenizer;
    
    let compressed = filter.compress(text, &tokenizer);
    
    // Calculate quality metrics
    let metrics = QualityMetrics::calculate(text, &compressed);
    
    println!("Compression Results:");
    println!("  Keyword Retention: {:.1}%", metrics.keyword_retention * 100.0);
    println!("  Entity Retention: {:.1}%", metrics.entity_retention * 100.0);
    println!("  Vocabulary Ratio: {:.1}%", metrics.vocabulary_ratio * 100.0);
    println!("  Overall Score: {:.1}%", metrics.overall_score * 100.0);
}
```

## Dictionary Compression (for repetitive text)

```rust
use compression_prompt::{Compressor, CompressorConfig};
use compression_prompt::tokenizer::MockTokenizer;

fn try_dictionary_compression(text: &str) -> Result<String, String> {
    let config = CompressorConfig::default();
    let compressor = Compressor::new(config);
    let tokenizer = MockTokenizer;
    
    match compressor.compress(text, &tokenizer) {
        Ok(result) => {
            println!("Dictionary compression successful!");
            println!("  Ratio: {:.1}%", result.compression_ratio * 100.0);
            println!("  Dictionary entries: {}", result.dictionary.entries.len());
            Ok(result.compressed)
        }
        Err(e) => {
            println!("Dictionary failed: {:?}", e);
            println!("Falling back to statistical compression...");
            
            // Fallback to statistical
            let config = StatisticalFilterConfig::default();
            let filter = StatisticalFilter::new(config);
            Ok(filter.compress(text, &tokenizer))
        }
    }
}
```

## Advanced: Custom Weights

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.5,
    idf_weight: 0.4,         // Prioritize rare words more
    position_weight: 0.3,    // Prioritize start/end more
    pos_weight: 0.1,         // Care less about word type
    entity_weight: 0.15,     // Moderate entity importance
    entropy_weight: 0.05,    // Less focus on diversity
};
```

## Testing with LLMs

```python
from pathlib import Path
import json

def load_compressed_pair(paper_id: int, method: str = "statistical_50"):
    base_path = Path("benchmarks/datasets/llm_evaluation/prompts")
    pair_id = f"paper_{paper_id:03d}_{method}"
    
    original = (base_path / f"{pair_id}_original.txt").read_text()
    compressed = (base_path / f"{pair_id}_compressed.txt").read_text()
    metadata = json.loads((base_path / f"{pair_id}_metadata.json").read_text())
    
    return {
        "original": original,
        "compressed": compressed,
        "metadata": metadata
    }

# Usage
pair = load_compressed_pair(1, "statistical_50")
print(f"Compression: {pair['metadata']['compression_ratio']*100:.1f}%")
print(f"Quality: {pair['metadata']['quality']['overall_score']*100:.1f}%")

# Test with your LLM
# response_orig = llm.generate(pair['original'])
# response_comp = llm.generate(pair['compressed'])
```

## Performance Benchmark

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::tokenizer::MockTokenizer;
use std::time::Instant;

fn benchmark_compression(texts: Vec<String>) {
    let config = StatisticalFilterConfig::default();
    let filter = StatisticalFilter::new(config);
    let tokenizer = MockTokenizer;
    
    let mut total_time = 0.0;
    let mut total_original = 0;
    let mut total_compressed = 0;
    
    for text in texts {
        let start = Instant::now();
        let compressed = filter.compress(&text, &tokenizer);
        let duration = start.elapsed();
        
        total_time += duration.as_secs_f64();
        total_original += tokenizer.count_tokens(&text);
        total_compressed += tokenizer.count_tokens(&compressed);
    }
    
    println!("Benchmark Results:");
    println!("  Total texts: {}", texts.len());
    println!("  Avg time: {:.2}ms", (total_time / texts.len() as f64) * 1000.0);
    println!("  Total tokens: {} → {}", total_original, total_compressed);
    println!("  Compression: {:.1}%", (total_compressed as f32 / total_original as f32) * 100.0);
}
```

## See Also

- [README.md](../README.md) - Main documentation
- [ANALYSIS.md](../benchmarks/datasets/llm_evaluation/ANALYSIS.md) - Detailed results
- [test_with_llm.py](../benchmarks/datasets/llm_evaluation/test_with_llm.py) - Python helper

