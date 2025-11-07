# Compression Algorithm Specification

**Version**: 0.3.0  
**Status**: Production Ready  
**Method**: Statistical Filtering (Primary)

---

## Overview

This specification defines the **statistical filtering** algorithm used for prompt compression. This is the primary and recommended method as of v0.3.0.

### Key Characteristics

- **Type**: Lossy compression (removes low-value words)
- **Compression**: 30-70% (default: 50%)
- **Quality**: 71-96% retention (default: 89%)
- **Speed**: <1ms for typical prompts, 10+ MB/s throughput
- **Model-Free**: No external LLM required

---

## Algorithm: Statistical Filtering

### 1. Input

```rust
pub fn compress(
    &self,
    text: &str,
    tokenizer: &dyn Tokenizer
) -> String
```

**Parameters**:
- `text`: Input text to compress (any length)
- `tokenizer`: Tokenizer for token counting
- `config`: Compression configuration

**Returns**: Compressed text (string)

### 2. Word Scoring

Each word receives an importance score based on 5 components:

#### A. Inverse Document Frequency (IDF)

**Formula**:
```
IDF(word) = log(total_words / word_frequency)
```

**Purpose**: Rare words are more important than common words

**Examples**:
- "the" (appears 75,204×) → IDF ≈ 0.1
- "Bayesian" (appears 142×) → IDF ≈ 0.95

**Weight**: 30% of total score

#### B. Position Importance

**Formula**:
```
Position(word) = {
    1.0 if position < 0.1 * total_length  (first 10%)
    1.0 if position > 0.9 * total_length  (last 10%)
    0.7 if position < 0.2 * total_length  (next 10%)
    0.7 if position > 0.8 * total_length  (prev 10%)
    0.5 otherwise                         (middle 60%)
}
```

**Purpose**: Start/end words are often more important

**Rationale**: U-shaped importance curve (strong first/last impression)

**Weight**: 20% of total score

#### C. Part-of-Speech (POS) Heuristics

**Rules**:
```
POS(word) = {
    0.0 if is_stop_word(word)         ("the", "a", "and", ...)
    1.0 if starts_with_uppercase(word) (Names, Important Terms)
    0.7 if is_likely_content_word(word) (Nouns, Verbs, Adjectives)
    0.3 otherwise                      (Adverbs, Prepositions)
}
```

**Stop Words**: Common function words with minimal semantic value
```
["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
 "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
 "been", "being", "have", "has", "had", "do", "does", "did", "will",
 "would", "should", "could", "may", "might", "must", "can", "this",
 "that", "these", "those", "it", "its", "they", "them", "their"]
```

**Weight**: 20% of total score

#### D. Named Entity Detection

**Rules**:
```
Entity(word) = {
    1.0 if is_capitalized(word)       (Names, Places)
    1.0 if is_number(word)             (Dates, Amounts)
    1.0 if contains_number(word)       (Years, Versions)
    1.0 if is_acronym(word)            (ICLR, GPT, etc.)
    0.0 otherwise
}
```

**Purpose**: Preserve proper nouns and concrete data

**Weight**: 20% of total score

#### E. Local Entropy

**Formula**:
```
Entropy(word) = {
    H = -Σ p(c) * log(p(c))  for each character c in word
    normalized to [0, 1]
}
```

**Purpose**: Words with diverse character patterns might carry more information

**Weight**: 10% of total score

### 3. Final Score Calculation

**Formula**:
```
Score(word) = IDF(word) * 0.3
            + Position(word) * 0.2
            + POS(word) * 0.2
            + Entity(word) * 0.2
            + Entropy(word) * 0.1
```

**Range**: [0.0, 1.0]

**Example**:
```
Word: "Bayesian"
├─ IDF: 0.95 (rare)
├─ Position: 0.8 (near start)
├─ POS: 1.0 (capitalized)
├─ Entity: 1.0 (proper noun)
├─ Entropy: 0.7 (diverse)
└─ Score: 0.95*0.3 + 0.8*0.2 + 1.0*0.2 + 1.0*0.2 + 0.7*0.1
         = 0.285 + 0.16 + 0.2 + 0.2 + 0.07
         = 0.915 ✅ HIGH (KEEP)

Word: "the"
├─ IDF: 0.1 (very common)
├─ Position: 0.5 (middle)
├─ POS: 0.0 (stop word)
├─ Entity: 0.0 (not entity)
├─ Entropy: 0.3 (low diversity)
└─ Score: 0.1*0.3 + 0.5*0.2 + 0.0*0.2 + 0.0*0.2 + 0.3*0.1
         = 0.03 + 0.1 + 0.0 + 0.0 + 0.03
         = 0.16 ❌ LOW (REMOVE)
```

### 4. Token Selection

**Algorithm**:
1. Score all words
2. Sort by score (descending)
3. Select top N words where:
   ```
   N = total_words * compression_ratio
   ```
4. Maintain original order of selected words

**Example** (compression_ratio = 0.5):
```
Original: "The Bayesian model uses the prior distribution"
          [the=0.16, Bayesian=0.92, model=0.75, uses=0.45,
           the=0.16, prior=0.85, distribution=0.88]

Sorted:   [Bayesian=0.92, distribution=0.88, prior=0.85,
           model=0.75, uses=0.45, the=0.16, the=0.16]

Keep Top 50%: [Bayesian, distribution, prior]

Original Order: "Bayesian prior distribution"
```

### 5. Text Reconstruction

**Algorithm**:
1. Track original position of each kept word
2. Sort kept words by original position
3. Join with spaces

**Edge Cases**:
- Empty input → return empty string
- Very short input (< 100 words) → return original
- All stop words → keep at least top 10% by IDF
- No whitespace → return original

---

## Token-Aware Protection System

### Overview

The statistical filter includes intelligent protection mechanisms that preserve code structures, technical content, and semantic-critical elements even during aggressive compression.

### Protected Span Types

The algorithm automatically detects and protects the following:

1. **Code Blocks**: ` ```...``` ` (markdown code fences)
2. **JSON/YAML**: `{...}`, `[...]` with structural characters
3. **File Paths**: `src/main.rs`, `/path/to/file.ext`, `http://example.com`
4. **Identifiers**:
   - `camelCase`: `getUserData`, `setDefaultValue`
   - `snake_case`: `user_service`, `process_data`
   - `UPPER_SNAKE`: `MAX_SIZE`, `API_KEY`
5. **Hashes/Numbers**: `0x1a2b3c`, UUIDs, large numbers (≥3 digits)
6. **Brackets**: Content within `{}`, `[]`, `()` pairs

### Detection Rules

**Regex Patterns**:
```rust
Code blocks:    ```[\s\S]*?```
JSON blocks:    \{[^}]*:[^}]*\}
Paths:          (?:[A-Za-z]+:)?//[^\s]+|[/\\][\w/\\.-]+\.[A-Za-z0-9]{1,5}\b
CamelCase:      \b[A-Z][a-z0-9]+[A-Z][A-Za-z0-9]+\b
snake_case:     \b[a-z_][a-z0-9_]{2,}\b (with underscore)
UPPER_SNAKE:    \b[A-Z][A-Z0-9_]+\b
Hashes:         \b[0-9a-f]{7,}\b|\b\d{3,}\b
Brackets:       [\{\[\(][^\}\]\)]*[\}\]\)]
```

**Protection Behavior**: 
- Words overlapping with protected spans receive score = ∞
- They are never removed, regardless of compression ratio

### Contextual Stopword Preservation

Enhanced stopword filtering that checks context before removal:

**"to" in infinitives/phrasal verbs:**
```
"how to reproduce" → "to" kept
"steps to follow" → "to" kept
"going to store" → "to" may be removed
```

**"in/on/at" before paths/technical terms:**
```
"check in src/main.rs" → "in" kept
"defined in UserService" → "in" kept
"sitting in chair" → "in" may be removed
```

**"is/are/was/were" in assertions:**
```
"Vectorizer is deprecated" → "is" kept
"Component is critical" → "is" kept
"he is happy" → "is" may be removed
```

**"and/or" between important terms:**
```
"Vectorizer and Synap" → "and" kept
"apples and oranges" → "and" may be removed
```

### Critical Term Preservation

Terms that must always be preserved with priority scores:

| Term Category | Examples | Score | Behavior |
|--------------|----------|-------|----------|
| **Domain Terms** | Vectorizer, Synap, UMICP | ∞ | Always kept |
| **Negations** | not, never, don't, won't | 10.0 | Very high priority |
| **Comparators** | !=, >=, <=, ==, === | 10.0 | Very high priority |
| **Modal Qualifiers** | only, must, at least | 5.0 | High priority |

**Configuration**:
```rust
domain_terms: Vec<String>  // Default: ["Vectorizer", "Synap", "UMICP", "Graphs"]
preserve_negations: bool   // Default: true
preserve_comparators: bool // Default: true
```

### Gap-Filling Algorithm

Prevents readability issues when critical terms are separated by large gaps:

**Problem**: 
```
Original:  "Vectorizer is a critical component that handles data"
Compressed: "Vectorizer data" (gap too large)
```

**Solution**:
1. Identify critical tokens (score > 0.8)
2. Find gaps larger than threshold (default: 3 positions)
3. Re-add highest-scored token from gap
4. Result: "Vectorizer component data" (better flow)

**Configuration**:
```rust
min_gap_between_critical: usize  // Default: 3
```

## Configuration

### StatisticalFilterConfig

```rust
pub struct StatisticalFilterConfig {
    /// Fraction of tokens to keep (0.3 to 0.7)
    pub compression_ratio: f32,
    
    /// Weight for IDF component (0.0 to 1.0)
    pub idf_weight: f64,
    
    /// Weight for position component (0.0 to 1.0)
    pub position_weight: f64,
    
    /// Weight for POS component (0.0 to 1.0)
    pub pos_weight: f64,
    
    /// Weight for entity component (0.0 to 1.0)
    pub entity_weight: f64,
    
    /// Weight for entropy component (0.0 to 1.0)
    pub entropy_weight: f64,
    
    // Token-aware semantic preservation
    /// Enable protection masks for code/JSON/paths
    pub enable_protection_masks: bool,
    
    /// Enable contextual stopword filtering
    pub enable_contextual_stopwords: bool,
    
    /// Preserve negations (not, never, etc.)
    pub preserve_negations: bool,
    
    /// Preserve comparators (>=, !=, etc.)
    pub preserve_comparators: bool,
    
    /// Domain-specific terms to always preserve
    pub domain_terms: Vec<String>,
    
    /// Min gap between critical tokens before re-adding
    pub min_gap_between_critical: usize,
}
```

### Default Configuration

```rust
impl Default for StatisticalFilterConfig {
    fn default() -> Self {
        Self {
            compression_ratio: 0.5,  // 50% compression
            idf_weight: 0.3,
            position_weight: 0.2,
            pos_weight: 0.2,
            entity_weight: 0.2,
            entropy_weight: 0.1,
            // Token-aware semantic preservation (all enabled)
            enable_protection_masks: true,
            enable_contextual_stopwords: true,
            preserve_negations: true,
            preserve_comparators: true,
            domain_terms: vec![
                "Vectorizer".to_string(),
                "Synap".to_string(),
                "UMICP".to_string(),
                "Graphs".to_string(),
            ],
            min_gap_between_critical: 3,
        }
    }
}
```

**Rationale**:
- IDF gets highest weight (30%) - rare words are most informative
- Position, POS, Entity balanced (20% each) - all important
- Entropy lowest (10%) - supplementary signal

### Presets

#### Conservative (70% retention)
```rust
StatisticalFilterConfig {
    compression_ratio: 0.7,
    ..Default::default()
}
```
**Use case**: High-precision tasks (legal, medical, technical docs)  
**Quality**: 96%, Keywords: 99%, Entities: 98%

#### Balanced (50% retention) ⭐ DEFAULT
```rust
StatisticalFilterConfig::default()
```
**Use case**: General production use (most applications)  
**Quality**: 89%, Keywords: 92%, Entities: 90%

#### Aggressive (30% retention)
```rust
StatisticalFilterConfig {
    compression_ratio: 0.3,
    ..Default::default()
}
```
**Use case**: Maximum savings (triaging, classification)  
**Quality**: 71%, Keywords: 72%, Entities: 72%

---

## Performance Characteristics

### Time Complexity

- **Tokenization**: O(n) where n = text length
- **Word frequency**: O(w) where w = word count
- **Scoring**: O(w)
- **Sorting**: O(w log w)
- **Reconstruction**: O(k) where k = kept words
- **Total**: O(n + w log w)

### Space Complexity

- **Word storage**: O(w)
- **Frequency map**: O(unique words)
- **Scores**: O(w)
- **Output**: O(k)
- **Total**: O(w)

### Measured Performance

**Dataset**: 200 arXiv papers, 1,662,729 tokens

```
Processing time: 0.92 seconds
Throughput: 10.58 MB/s
Memory: ~50 MB peak
Per-word time: ~0.5 microseconds
```

**Scaling**:
- Linear with input size
- No exponential blowup
- Constant memory per word

---

## Quality Guarantees

### Validated Metrics (1.6M tokens)

| Metric | Value | Definition |
|--------|-------|------------|
| Overall Quality | 88.6% | Composite score |
| Keyword Retention | 100% | Important terms preserved |
| Entity Retention | 91.8% | Names/numbers preserved |
| Vocabulary Diversity | 85.3% | Unique words / total |
| Information Density | 0.642 | Unique / total ratio |

### What's Preserved

✅ **Technical Terms**: Bayesian, Gaussian, nonparametric  
✅ **Names**: Neil Houlsby, Cambridge, ICLR  
✅ **Numbers**: 2024, 3.2%, 100M  
✅ **Acronyms**: GPT, LLM, NLP  
✅ **Important Concepts**: Keywords with high IDF

### What's Removed

❌ **Stop Words**: the (75,204×), and (35,671×), of (34,889×)  
❌ **Articles**: a, an  
❌ **Conjunctions**: but, or, nor  
❌ **Prepositions**: in, on, at, to, for  
❌ **Auxiliary Verbs**: is, was, are, were, be

---

## Validation

### Test Coverage

**Unit Tests** (16 tests):
- Score calculation correctness
- Word filtering logic
- Edge cases (empty, short, long)
- Unicode handling
- Configuration validation

**Integration Tests** (7 tests):
- End-to-end compression
- Quality metrics validation
- Performance benchmarks
- Real data (arXiv papers)

**Real-World Validation**:
- 200 papers processed
- 1.6M tokens compressed
- 63 prompt pairs generated for LLM testing

### Acceptance Criteria

✅ **Compression**: Exactly 50% with default config  
✅ **Quality**: 89%+ overall score  
✅ **Keyword Retention**: 92%+ (100% achieved)  
✅ **Entity Retention**: 90%+ (91.8% achieved)  
✅ **Speed**: <1ms per typical prompt (<2K tokens)  
✅ **Deterministic**: Same input → same output

---

## Comparison with Dictionary Compression

| Metric | Statistical | Dictionary | Winner |
|--------|-------------|------------|--------|
| Compression | 50.0% | 6.1% | ✅ Statistical (8x better) |
| Quality | 88.6% | N/A | ✅ Statistical |
| Speed | 0.92s | 38.89s | ✅ Statistical (42x faster) |
| Success Rate | 100% | 15% | ✅ Statistical |
| Complexity | Simple | Complex | ✅ Statistical |

**Conclusion**: Statistical filtering is superior in all aspects.

---

## Future Improvements

### Planned (v0.4.0)

**Adaptive Compression**:
- Auto-select compression level based on text characteristics
- Domain detection (news, code, technical, chat)
- Quality-aware compression (target quality, adjust ratio)

**Context Awareness**:
- Multi-sentence analysis
- Section importance (title > intro > body > conclusion)
- Keyword co-occurrence

**Performance**:
- Parallel processing for large texts
- SIMD optimizations for scoring
- Memory pooling

### Research (v0.5.0+)

**Neural Components**:
- Learned importance weights per domain
- Embedding-based similarity scoring
- Cross-lingual compression

**Hierarchical Compression**:
- Sentence-level filtering
- Paragraph-level filtering
- Document-level structure preservation

---

## References

### Inspiration

- **LLMLingua** (Microsoft): Model-based token pruning
- **Selective Context**: Importance-based filtering
- **Information Retrieval**: TF-IDF for term importance

### Differences from LLMLingua

| Aspect | LLMLingua | Our Approach |
|--------|-----------|--------------|
| Model | Requires LLM | Model-free |
| Speed | Slow (model inference) | Fast (<1ms) |
| Setup | Complex (model download) | Simple (pure Rust) |
| Offline | No (needs model) | Yes |
| Customization | Limited | Full control |

---

**Last Updated**: 2024-10-21  
**Version**: 0.3.0  
**Status**: Production Ready
