# Tokenizer Interface Specification

## Overview

The compression algorithm must work with multiple LLM tokenizers (Claude, GPT, Mistral, etc.). This document specifies the pluggable tokenizer interface and testing requirements.

## Trait Definition

```rust
pub trait Tokenizer: Send + Sync {
    /// Encode text into a sequence of tokens.
    fn encode(&self, text: &str) -> Vec<Token>;

    /// Decode a sequence of tokens back into text.
    fn decode(&self, tokens: &[Token]) -> String;

    /// Count tokens in text without allocating the token vector.
    fn count_tokens(&self, text: &str) -> usize {
        self.encode(text).len()
    }

    /// Test how many tokens a marker string produces.
    fn test_marker(&self, marker: &str) -> usize {
        self.count_tokens(marker)
    }

    /// Get tokenizer name for debugging/logging.
    fn name(&self) -> &str;
}
```

## Token Representation

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(pub u32);
```

**Design Rationale**:
- `u32`: Sufficient for all known tokenizers (vocabularies < 4B tokens)
- `Copy`: Enables efficient array operations
- `Hash`: Required for n-gram frequency counting

## Thread Safety

**Requirement**: All tokenizers must be `Send + Sync`

**Rationale**: Enables parallel n-gram extraction using rayon

**Implementation Notes**:
- Stateless tokenizers: Trivially thread-safe
- Stateful tokenizers: Use interior mutability (Arc<RwLock<State>>)

## Method Specifications

### `encode(text: &str) -> Vec<Token>`

**Purpose**: Convert text to token sequence

**Requirements**:
- Deterministic: Same input → same output
- Complete: Handle all valid UTF-8 text
- Consistent: Match provider's actual tokenization

**Performance**: Should be O(n) in text length

**Example**:
```rust
let tokens = tokenizer.encode("Hello, world!");
// Result: [Token(15339), Token(11), Token(1917), Token(0)]
```

### `decode(tokens: &[Token]) -> String`

**Purpose**: Convert token sequence back to text

**Requirements**:
- Round-trip: `decode(encode(text))` should ≈ text
- Handle unknown tokens: Use replacement char (�)

**Performance**: Should be O(k) in token count

**Example**:
```rust
let text = tokenizer.decode(&[Token(15339), Token(11), Token(1917)]);
// Result: "Hello, world"
```

### `count_tokens(text: &str) -> usize`

**Purpose**: Fast token counting without allocation

**Default Implementation**:
```rust
fn count_tokens(&self, text: &str) -> usize {
    self.encode(text).len()
}
```

**Optimization Opportunity**: Override if tokenizer supports streaming count

**Critical Usage**: Used extensively in gain calculations

### `test_marker(marker: &str) -> usize`

**Purpose**: Determine token cost of dictionary markers

**Default Implementation**:
```rust
fn test_marker(&self, marker: &str) -> usize {
    self.count_tokens(marker)
}
```

**Usage**: Called once per marker format during dictionary building

**Example**:
```rust
assert_eq!(tokenizer.test_marker("⟦12⟧"), 2);
assert_eq!(tokenizer.test_marker("[#12]"), 3);
```

### `name() -> &str`

**Purpose**: Identify tokenizer for logging/debugging

**Requirements**: Return stable, unique name

**Examples**: "ClaudeTokenizer", "GPT4Tokenizer", "MistralTokenizer"

## Supported Tokenizers

### Phase 1: Mock Tokenizer

**Purpose**: Testing and development

**Implementation**: Simple whitespace splitting
```rust
impl Tokenizer for MockTokenizer {
    fn encode(&self, text: &str) -> Vec<Token> {
        text.split_whitespace()
            .enumerate()
            .map(|(i, _)| Token(i as u32))
            .collect()
    }
    // ...
}
```

### Phase 2: Real Tokenizers (Future)

#### Claude (Anthropic)

**Library**: `tiktoken-rs` or `tokenizers` with Claude vocab

**Vocabulary Size**: ~100K tokens

**Special Considerations**:
- Byte-pair encoding (BPE)
- Handles Unicode well
- Test marker format: `⟦12⟧` typically 1-2 tokens

#### GPT-4 (OpenAI)

**Library**: `tiktoken-rs`

**Vocabulary Size**: ~100K tokens (cl100k_base)

**Special Considerations**:
- BPE with merges
- Whitespace handling differs from Claude
- Test multiple marker formats

#### Mistral

**Library**: `tokenizers` with Mistral vocab

**Vocabulary Size**: ~32K tokens

**Special Considerations**:
- SentencePiece-based
- May tokenize markers differently than Claude/GPT
- Verify marker cost ≤ 2 tokens

## Marker Tokenization Testing

**Critical Requirement**: Marker format must tokenize efficiently across all target tokenizers

### Test Protocol

For each tokenizer:

1. **Generate test markers**:
   ```rust
   let test_ids = [1, 12, 123, 999];
   ```

2. **Test all candidate formats**:
   ```rust
   let formats = ["⟦{}⟧", "[#{}]", "⦃{}⦄", "«{}»"];
   ```

3. **Measure token costs**:
   ```rust
   for format in formats {
       for id in test_ids {
           let marker = format.replace("{}", &id.to_string());
           let cost = tokenizer.test_marker(&marker);
           println!("{}: {} tokens", marker, cost);
       }
   }
   ```

4. **Select best format**:
   - Prefer format with lowest max cost across all IDs
   - Break ties by consistency (lowest variance)

### Expected Results

| Tokenizer | `⟦12⟧` | `[#12]` | `⦃12⦄` | `«12»` | Best |
|-----------|--------|---------|--------|--------|------|
| Mock | 1 | 1 | 1 | 1 | Any |
| Claude | 1-2 | 2-3 | 1-2 | 2-3 | `⟦{}⟧` or `⦃{}⦄` |
| GPT-4 | 1-2 | 3-4 | 1-2 | 2-3 | `⟦{}⟧` or `⦃{}⦄` |
| Mistral | 2-3 | 3-4 | 2-3 | 2-3 | `⟦{}⟧` or `⦃{}⦄` |

**Recommendation**: Use `⟦{}⟧` as default (good across all tokenizers)

## Testing Requirements

### Unit Tests

**File**: `compression-core/src/tokenizer.rs`

Test cases:
1. ✅ Round-trip: `decode(encode(text))` ≈ text
2. ✅ Determinism: Multiple calls return same tokens
3. ✅ Count efficiency: `count_tokens` matches `encode().len()`
4. ✅ Marker testing: Various formats produce expected costs
5. ✅ Unicode handling: Non-ASCII text encodes correctly

### Integration Tests

**File**: `compression-core/tests/tokenizer_tests.rs`

Test real tokenizers with:
1. Code samples (imports, function definitions)
2. Natural language (paragraphs, sentences)
3. Mixed content (docstrings, comments)
4. Edge cases (empty string, single char, very long)

### Compatibility Matrix

Maintain test suite that validates:
- All tokenizers produce sensible token counts
- Marker costs are acceptable (≤ 3 tokens)
- Compression algorithm works correctly with each

**Test Data**: `compression-core/tests/fixtures/sample_inputs/`
- `code_sample.txt`: Typical source code
- `prose_sample.txt`: Natural language
- `mixed_sample.txt`: Code + documentation

## Error Handling

### Encoding Errors

**Scenario**: Invalid UTF-8 or unsupported characters

**Behavior**: Replace with special token or skip

**Testing**: Ensure no panics on malformed input

### Token Overflow

**Scenario**: Token ID exceeds u32::MAX (theoretical)

**Behavior**: Saturate at u32::MAX or return error

**Likelihood**: Negligible (no tokenizer has >4B vocab)

## Performance Benchmarks

**File**: `compression-core/benches/tokenizer_bench.rs`

Benchmark each tokenizer:

| Operation | Input Size | Target Performance |
|-----------|------------|-------------------|
| `encode` | 1KB | < 1ms |
| `encode` | 10KB | < 10ms |
| `encode` | 100KB | < 100ms |
| `count_tokens` | 10KB | < 5ms (if optimized) |

**Methodology**: Use `criterion` for statistical analysis

## Future Extensions

### Streaming Tokenization

For very large inputs (>1MB):
```rust
trait StreamingTokenizer: Tokenizer {
    fn encode_stream(&self, reader: impl Read) -> impl Iterator<Item = Token>;
}
```

### Caching

For repeated tokenization:
```rust
struct CachedTokenizer<T: Tokenizer> {
    inner: T,
    cache: LruCache<String, Vec<Token>>,
}
```

### Vocabulary Inspection

For advanced optimization:
```rust
trait InspectableTokenizer: Tokenizer {
    fn vocab_size(&self) -> usize;
    fn token_to_string(&self, token: Token) -> Option<String>;
    fn string_to_token(&self, s: &str) -> Option<Token>;
}
```

## Implementation Checklist

- [ ] Define `Token` struct and `Tokenizer` trait
- [ ] Implement `MockTokenizer` for testing
- [ ] Write unit tests for trait methods
- [ ] Create marker format test harness
- [ ] Document tokenizer-specific quirks
- [ ] Benchmark performance on large inputs
- [ ] Validate thread safety with concurrent tests
- [ ] Prepare for real tokenizer integration (Phase 2)

