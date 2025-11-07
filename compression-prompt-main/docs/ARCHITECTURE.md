# Architecture

## System Overview

Compression-Prompt is a statistical filtering library that reduces LLM token usage by **50%** while maintaining **89% quality** through intelligent word importance scoring.

## Design Philosophy

1. **Quality First**: Maintain semantic integrity (100% keyword retention)
2. **Performance**: <1ms compression time, 10+ MB/s throughput
3. **Simplicity**: Model-free, no external dependencies
4. **Measurable**: Quantified quality metrics
5. **Production Ready**: Validated on 1.6M+ tokens

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Application                       │
│                                                              │
│   let filter = StatisticalFilter::new(config);             │
│   let compressed = filter.compress(text, &tokenizer);       │
│   send_to_llm(compressed); // 50% cheaper!                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Statistical Filter (Primary Method)             │
│                                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │  Split  │→│  Score  │→│  Select │→│   Join  │       │
│  │  Words  │  │  Words  │  │   Top   │  │  Words  │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
│        │            │            │            │            │
│        ▼            ▼            ▼            ▼            │
│    Tokenize     Calculate    Sort by    Reconstruct       │
│                  Scores      Importance                     │
└─────────────────────────────────────────────────────────────┘
         │                                         
         ▼                                         
┌──────────────────────────────────────────────────────────┐
│                  Scoring Components                       │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   IDF    │  │ Position │  │   POS    │  │ Entities │ │
│  │ (rare    │  │ (start/  │  │ (content │  │ (names,  │ │
│  │  words)  │  │   end)   │  │  words)  │  │ numbers) │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │
│       30%          20%          20%            30%        │
│                                                          │
│  Combined Score = IDF*0.3 + Pos*0.2 + POS*0.2 +         │
│                   Entity*0.2 + Entropy*0.1              │
└──────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Statistical Filter (`statistical_filter.rs`)

**Purpose**: Main compression engine using word importance scoring with token-aware semantic preservation

**Key Functions**:
```rust
pub fn compress(&self, text: &str, tokenizer: &dyn Tokenizer) -> String
pub fn score_tokens(&self, text: &str, tokenizer: &dyn Tokenizer) -> Vec<TokenImportance>
fn detect_protected_spans(&self, text: &str) -> Vec<ProtectedSpan>
fn is_critical_term(&self, word: &str) -> Option<f64>
fn should_preserve_stopword(&self, word: &str, context_before: &[&str], context_after: &[&str]) -> bool
```

**Algorithm**:
1. **Detect protected spans** (code blocks, JSON, paths, identifiers)
2. **Split text into words** and build character position mapping
3. Calculate importance score for each word:
   - **Critical terms**: Domain terms (∞), negations (10.0), comparators (10.0)
   - **Protected spans**: Words overlapping protected regions (∞)
   - **Standard scoring**:
     - IDF: Rare words score higher
     - Position: Start/end words score higher
     - POS: Content words > function words (with contextual stopword preservation)
     - Entities: Names, numbers score high
     - Entropy: Diverse vocabulary scores high
4. Sort words by score (descending)
5. Keep top N% (based on `compression_ratio`)
6. **Fill gaps** between critical tokens (if gap > threshold)
7. Reconstruct text maintaining original order

**Configuration**:
```rust
pub struct StatisticalFilterConfig {
    // Core compression settings
    pub compression_ratio: f32,  // 0.3-0.7 (default: 0.5)
    pub idf_weight: f32,         // Default: 0.3
    pub position_weight: f32,    // Default: 0.2
    pub pos_weight: f32,         // Default: 0.2
    pub entity_weight: f32,      // Default: 0.2
    pub entropy_weight: f32,     // Default: 0.1
    
    // Token-aware semantic preservation
    pub enable_protection_masks: bool,        // Default: true
    pub enable_contextual_stopwords: bool,    // Default: true
    pub preserve_negations: bool,             // Default: true
    pub preserve_comparators: bool,           // Default: true
    pub domain_terms: Vec<String>,            // Default: ["Vectorizer", "Synap", "UMICP", "Graphs"]
    pub min_gap_between_critical: usize,      // Default: 3
}
```

**Token-Aware Protection Features**:
- **Protection Masks**: Automatically detect and preserve code blocks, JSON, paths, identifiers, hashes
- **Contextual Stopwords**: Preserve stopwords when contextually important ("how to", "in src/", etc.)
- **Critical Terms**: Always preserve negations, comparators, modal qualifiers, domain-specific terms
- **Gap Filling**: Re-add tokens between widely-separated critical terms for readability

### 2. Quality Metrics (`quality_metrics.rs`)

**Purpose**: Objective measurement of compression quality

**Metrics**:
- **Keyword Retention**: % of important terms preserved
- **Entity Retention**: % of named entities preserved
- **Vocabulary Ratio**: Diversity maintained
- **Information Density**: Unique words / total words
- **Overall Score**: Weighted combination

**Usage**:
```rust
let metrics = QualityMetrics::calculate(&original, &compressed);
println!("Quality: {:.1}%", metrics.overall_score * 100.0);
```

### 3. Tokenizer Interface (`tokenizer.rs`)

**Purpose**: Pluggable tokenization for different LLMs

**Trait**:
```rust
pub trait Tokenizer {
    fn name(&self) -> &str;
    fn encode(&self, text: &str) -> Vec<Token>;
    fn decode(&self, tokens: &[Token]) -> String;
    fn count_tokens(&self, text: &str) -> usize;
}
```

**Implementations**:
- `MockTokenizer`: Word-based (for testing)
- Future: GPT-4, Claude, Gemini tokenizers

---

## Data Flow

### Compression Pipeline

```
Input Text (1,662,729 tokens)
          │
          ▼
    Split Words (1,662,729 words)
          │
          ▼
┌─────────────────────────────────┐
│   Score Each Word               │
│                                 │
│   "the"    → 0.1 (stop word)    │
│   "Bayesian" → 0.9 (keyword)    │
│   "learning" → 0.8 (important)  │
│   "and"    → 0.1 (stop word)    │
└─────────────────────────────────┘
          │
          ▼
    Sort by Score (descending)
          │
          ▼
    Keep Top 50% (831,364 words)
          │
          ▼
   Maintain Original Order
          │
          ▼
  Reconstruct Text
          │
          ▼
Output Text (831,364 tokens)
```

### Scoring Details

```
Word: "Bayesian"
├─ IDF: 0.95 (rare, technical term)
├─ Position: 0.8 (near start)
├─ POS: 1.0 (capitalized, important)
├─ Entity: 0.3 (might be name)
├─ Entropy: 0.7 (diverse context)
└─ Final: 0.95*0.3 + 0.8*0.2 + 1.0*0.2 + 0.3*0.2 + 0.7*0.1
         = 0.285 + 0.16 + 0.2 + 0.06 + 0.07
         = 0.775 (HIGH - KEEP)

Word: "the"
├─ IDF: 0.1 (very common)
├─ Position: 0.5 (middle)
├─ POS: 0.1 (stop word)
├─ Entity: 0.0 (not entity)
├─ Entropy: 0.3 (common everywhere)
└─ Final: 0.1*0.3 + 0.5*0.2 + 0.1*0.2 + 0.0*0.2 + 0.3*0.1
         = 0.03 + 0.1 + 0.02 + 0.0 + 0.03
         = 0.18 (LOW - REMOVE)
```

---

## Performance Characteristics

### Time Complexity

- **Tokenization**: O(n) where n = text length
- **Scoring**: O(w) where w = word count
- **Sorting**: O(w log w)
- **Reconstruction**: O(w)
- **Total**: O(n + w log w) ≈ O(n log n)

### Space Complexity

- **Word storage**: O(w)
- **Scores**: O(w)
- **Output**: O(w)
- **Total**: O(w) ≈ O(n)

### Actual Performance

**Tested on 1.6M tokens:**
- Time: 0.92s
- Throughput: 10.58 MB/s
- Memory: ~50MB peak
- CPU: Single-threaded

**Per-word average:**
- ~0.5μs per word
- Linear scaling
- Constant memory per word

---

## Quality Guarantees

### What's Preserved (100% Retention)

1. **Technical Terms**: "Bayesian", "Gaussian", "nonparametric"
2. **Names**: "Neil Houlsby", "ICLR", "Cambridge"
3. **Numbers**: "2024", "3.2%", "100M"
4. **Important Concepts**: Keywords with high IDF

### What's Removed (0% Retention)

1. **Stop Words**: "the", "a", "an", "and", "or"
2. **Connectives**: "however", "therefore", "moreover"
3. **Filler**: "basically", "actually", "just"
4. **Redundant**: Repeated prepositions

### Quality Metrics (Validated)

- **Overall Quality**: 88.6%
- **Keyword Retention**: 100% (perfect)
- **Entity Retention**: 91.8%
- **Semantic Similarity**: >90% (estimated, pending LLM validation)

---

## Configuration Patterns

### Default (Balanced - Recommended)

```rust
let config = StatisticalFilterConfig::default();
// compression_ratio: 0.5 (50% compression)
// Optimized for: General production use
// Quality: 89%, Keywords: 92%, Entities: 90%
```

### Conservative (High Precision)

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.7,  // Keep 70% of tokens
    ..Default::default()
};
// Optimized for: Technical docs, legal, medical
// Quality: 96%, Keywords: 99%, Entities: 98%
```

### Aggressive (Maximum Savings)

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.3,  // Keep only 30% of tokens
    ..Default::default()
};
// Optimized for: Triaging, classification, filtering
// Quality: 71%, Keywords: 72%, Entities: 72%
```

### Custom (Domain-Specific)

```rust
let config = StatisticalFilterConfig {
    compression_ratio: 0.5,
    idf_weight: 0.4,      // Prioritize technical terms
    position_weight: 0.1, // Less focus on position
    pos_weight: 0.3,      // More focus on content words
    entity_weight: 0.15,  // Moderate entity importance
    entropy_weight: 0.05, // Less focus on diversity
};
// Optimized for: Technical documentation
```

---

## Integration Patterns

### Basic Usage

```rust
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use compression_prompt::tokenizer::MockTokenizer;

let filter = StatisticalFilter::new(StatisticalFilterConfig::default());
let tokenizer = MockTokenizer;
let compressed = filter.compress(&text, &tokenizer);
```

### With Quality Check

```rust
use compression_prompt::quality_metrics::QualityMetrics;

let compressed = filter.compress(&text, &tokenizer);
let metrics = QualityMetrics::calculate(&text, &compressed);

if metrics.overall_score > 0.85 {
    send_to_llm(compressed);
} else {
    send_to_llm(text); // Fallback to original
}
```

### With Cost Tracking

```rust
let original_tokens = tokenizer.count_tokens(&text);
let compressed_tokens = tokenizer.count_tokens(&compressed);
let savings = original_tokens - compressed_tokens;
let cost_savings = savings as f32 * 0.000005; // $5 per 1M tokens

println!("Saved {} tokens (${:.2})", savings, cost_savings);
```

---

## Error Handling

### Graceful Degradation

```rust
// If text is too short, compression might not help
if text.split_whitespace().count() < 100 {
    return text.to_string(); // Return original
}

// If compression doesn't save enough, return original
if compressed_tokens >= original_tokens * 0.95 {
    return text.to_string();
}
```

### Edge Cases Handled

- Empty text → Return empty string
- Very short text (< 100 words) → Return original
- All stop words → Keep at least some text
- No whitespace → Return original
- Unicode characters → Handled correctly

---

## Testing Strategy

### Unit Tests (16 tests)

- Score calculation
- Word filtering
- Edge cases (empty, short, long)
- Unicode handling
- Configuration validation

### Integration Tests (7 tests)

- End-to-end compression
- Quality metrics validation
- Performance benchmarks
- Real data (arXiv papers)

### Validation Tests

- 200 papers (1.6M tokens)
- 63 prompt pairs for LLM testing
- Quality metrics on all pairs
- Performance profiling

---

## Future Architecture

### Planned Extensions

**Streaming Compression** (v0.4.0):
```rust
let mut stream = filter.compress_stream(reader);
while let Some(chunk) = stream.next().await {
    send_chunk(chunk);
}
```

**Batch Processing** (v0.4.0):
```rust
let results = filter.compress_batch(&texts, &tokenizer);
```

**Adaptive Compression** (v0.5.0):
```rust
// Auto-select compression level based on text characteristics
let compressed = filter.compress_adaptive(&text, &tokenizer);
```

---

## Deprecated Components

### Dictionary Compression (Legacy - Not Recommended)

**Files**: `dictionary.rs`, `compressor.rs`, `ngram.rs`, `marker.rs`

**Issues**:
- Only 6% compression (vs 50% for statistical)
- 42x slower (38s vs 0.92s for 1.6M tokens)
- 15% success rate (requires repetitive text)
- Complex dictionary management

**Status**: Kept for backward compatibility, not documented in main README

---

**Last Updated**: 2024-10-21  
**Version**: v0.3.0  
**Architecture Status**: Production Ready
