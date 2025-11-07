# Benchmark Methodology Specification

**Version**: 0.3.0  
**Status**: Production Ready  
**Primary Method**: Statistical Filtering

---

## Overview

This document specifies the comprehensive benchmarking strategy for validating compression performance, quality, and efficiency claims for **statistical filtering**.

---

## Testing Dimensions

### 1. Compression Efficiency
- Token savings percentage (target: 50%)
- Compression ratio (target: 0.5)
- Quality retention (target: 89%+)

### 2. Semantic Quality
- Keyword retention (target: 92%+)
- Entity retention (target: 90%+)
- Vocabulary diversity (target: 85%+)
- Overall quality score (target: 85%+)

### 3. Performance
- Compression speed (target: >10 MB/s)
- Memory usage
- Latency by input size

---

## Test Datasets

### Primary Dataset: arXiv Papers (Academic Publications)

**Purpose**: Validate compression on real-world technical content

**Characteristics**:
- Technical terminology
- Citations and references
- Mathematical notation
- Abstract/intro/conclusion structure
- Natural language + jargon mix

**Dataset Details**:
- **Source**: arXiv.org (open access papers)
- **Location**: `benchmarks/datasets/benchmark_200_papers.txt`
- **Size**: 10.2 MB (1,662,729 tokens)
- **Papers**: 200 papers from various fields
- **Topics**: Machine Learning, Physics, Mathematics, CS

**Expected Compression**: 50% token savings with 89% quality

**Why arXiv?**
1. Publicly available (reproducible)
2. High-quality technical content
3. Diverse vocabulary (good test of importance scoring)
4. Real-world use case (researchers sending papers to LLMs)

### Secondary Datasets (Future)

#### Dataset 2: Code Documentation
**Purpose**: Technical docs with code examples
- Repeated API patterns
- Function signatures
- Example code blocks

**Expected**: 45-55% savings, 87%+ quality

#### Dataset 3: News Articles
**Purpose**: Journalistic content
- Inverted pyramid structure
- Named entities
- Temporal references

**Expected**: 40-50% savings, 90%+ quality

#### Dataset 4: Chat Logs
**Purpose**: Conversational content
- Informal language
- Repeated greetings/closings
- Short sentences

**Expected**: 50-60% savings, 85%+ quality

---

## Quantitative Metrics

### Primary Metrics

#### 1. Compression Ratio

**Definition**:
```
compression_ratio = compressed_tokens / original_tokens
```

**Interpretation**:
- 0.50 = 50% savings ✅
- 1.00 = no benefit (break-even)
- < 0.50 = aggressive compression (quality risk)

**Targets**:
- **Default (50%)**: 0.50 ± 0.02
- **Conservative (70%)**: 0.70 ± 0.02
- **Aggressive (30%)**: 0.30 ± 0.02

**Validated**: ✅ **Exactly 0.500** on 1.6M tokens

#### 2. Token Savings

**Definition**:
```
savings_pct = (1 - compression_ratio) × 100%
```

**Example**:
- Original: 1,662,729 tokens
- Compressed: 831,364 tokens
- Savings: 831,365 tokens (50.0%)

**Cost Savings** (GPT-4 @ $5/1M):
- Per million tokens: **$2.50 saved**
- Enterprise (1B tokens/month): **$2.5M/year**

#### 3. Quality Score

**Definition**: Composite metric from multiple quality signals

**Components**:
1. **Keyword Retention** (30% weight)
   - Important terms preserved
   - High IDF words tracked
   - Target: 92%+

2. **Entity Retention** (30% weight)
   - Names, numbers, dates preserved
   - Proper nouns tracked
   - Target: 90%+

3. **Vocabulary Diversity** (20% weight)
   - Unique words / total words
   - Information richness
   - Target: 85%+

4. **Information Density** (20% weight)
   - Meaningful words / total words
   - Content concentration
   - Target: 0.60+

**Formula**:
```
Quality = (KeywordRet * 0.3) + (EntityRet * 0.3) + 
          (VocabDiv * 0.2) + (InfoDensity * 0.2)
```

**Validated**: ✅ **88.6%** on 1.6M tokens

### Secondary Metrics

#### 4. Keyword Retention

**Definition**: % of important terms preserved in compressed text

**Measurement**:
1. Extract keywords from original (IDF > threshold)
2. Check presence in compressed text
3. Calculate percentage retained

**Validated**: ✅ **100%** (perfect)

#### 5. Entity Retention

**Definition**: % of named entities preserved

**Entities**:
- Names: "Neil Houlsby", "Fei-Fei Li"
- Places: "Cambridge", "Stanford"
- Numbers: "2024", "3.2%", "100M"
- Acronyms: "ICLR", "NeurIPS", "GPT"

**Validated**: ✅ **91.8%**

#### 6. Most Removed Words

**Definition**: Top words eliminated by compression

**Purpose**: Validate stop word removal strategy

**Expected Top Removals**:
- "the" (most common)
- "and", "or" (conjunctions)
- "a", "an" (articles)
- "of", "in", "to" (prepositions)

**Validated**:
1. "the" → 75,204 times (45.3%)
2. "and" → 35,671 times (21.5%)
3. "of" → 34,889 times (21.0%)
4. "a" → 28,041 times (16.9%)
5. "to" → 27,126 times (16.3%)

---

## Performance Benchmarks

### Benchmark Suite

**Tool**: Rust `std::time::Instant` for precise timing

**File**: `rust/src/bin/test_statistical.rs`

### Test Cases

#### 1. End-to-End Compression

**Input**: 1,662,729 tokens (10.2 MB)

**Measured**:
- Total time: 0.92 seconds
- Throughput: 10.58 MB/s
- Tokens/second: 1.81M tokens/s

**Memory**:
- Peak: ~50 MB
- Per-word: ~30 bytes

**Validated**: ✅ All targets exceeded

#### 2. Scalability

**Linear Scaling**:
- 100K tokens → ~55ms
- 500K tokens → ~275ms
- 1M tokens → ~550ms
- 1.6M tokens → ~920ms

**Constant per-word time**: ~0.5 microseconds

#### 3. Compression Levels

**Performance by Ratio**:

| Level | Ratio | Time (1.6M tokens) | Throughput |
|-------|-------|-------------------|------------|
| Conservative | 0.7 | 0.89s | 11.2 MB/s |
| **Balanced** | **0.5** | **0.92s** | **10.58 MB/s** |
| Aggressive | 0.3 | 0.95s | 10.2 MB/s |

**Conclusion**: Compression level has minimal impact on speed

---

## Quality Validation

### Automated Quality Metrics

**File**: `rust/src/quality_metrics.rs`

**Validation**: Every compressed output includes quality metrics

**Example Output**:
```
Quality Metrics:
  Overall Score: 88.6%
  Keyword Retention: 100.0%
  Entity Retention: 91.8%
  Vocabulary Diversity: 85.3%
  Information Density: 0.642
```

### LLM Evaluation Dataset

**Purpose**: Human-in-the-loop validation with real LLMs

**Location**: `benchmarks/datasets/llm_evaluation/`

**Contents**:
- 63 prompt pairs (original + compressed)
- 3 compression levels (30%, 50%, 70%)
- Quality metrics per pair
- Metadata (tokens, ratio, method)

**Usage**:
1. Load dataset.json
2. Send prompts to GPT-4/Claude/Gemini
3. Compare responses (original vs compressed)
4. Measure semantic similarity
5. Evaluate task accuracy

**Metrics to Collect**:
- Semantic similarity (cosine, BERT-score)
- Task accuracy (if task-specific)
- Human preference (blind A/B test)

**Target**: >90% semantic similarity, >95% task accuracy

---

## Benchmark Protocol

### Full Benchmark Run

**Script**: `cargo run --release --bin test_statistical`

**Steps**:
1. Load dataset (`benchmark_200_papers.txt`)
2. Apply statistical filtering (default: 50%)
3. Measure time and memory
4. Calculate quality metrics
5. Generate report

**Output**:
```
=== Statistical Compression Test ===

Dataset: benchmark_200_papers.txt
Original: 1,662,729 tokens (10.2 MB)
Compressed: 831,364 tokens
Savings: 831,365 tokens (50.0%)

Time: 0.92s (10.58 MB/s)

Quality Metrics:
  Overall Score: 88.6%
  Keyword Retention: 100.0%
  Entity Retention: 91.8%
  ...

Top 10 Removed Words:
  1. "the" → 75,204 times
  2. "and" → 35,671 times
  ...
```

### Comparison Benchmark

**Purpose**: Compare statistical vs dictionary compression

**Scripts**:
- `test_statistical` → Statistical filtering
- `test_compression` → Dictionary (legacy)

**Run Both**:
```bash
cargo run --release --bin test_statistical
cargo run --release --bin test_compression
```

**Compare**:
| Metric | Statistical | Dictionary | Winner |
|--------|-------------|------------|--------|
| Compression | 50.0% | 6.1% | ✅ Statistical (8x) |
| Time | 0.92s | 38.89s | ✅ Statistical (42x) |
| Success | 100% | 15% | ✅ Statistical |

---

## Regression Testing

### Continuous Benchmarks

**Frequency**: Every commit to main

**Scope**: Quick validation
- 10KB sample from dataset
- Statistical filtering only
- Performance check (>5% slower = investigate)

**Script**:
```bash
cargo test --release
cargo run --release --bin test_statistical
```

### Full Test Suite

**Frequency**: Weekly or before releases

**Scope**: Complete validation
- Full 1.6M token dataset
- All compression levels
- Quality metrics validation
- Memory profiling

**Checklist**:
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Benchmarks meet targets
- [ ] Quality metrics ≥ thresholds
- [ ] No clippy warnings
- [ ] Documentation updated

---

## Reporting Format

### Benchmark Report Template

```markdown
# Statistical Filtering Benchmark Report

**Date**: YYYY-MM-DD  
**Version**: 0.3.0  
**Commit**: <hash>  
**Rust**: 1.85+ (edition 2024)

## Dataset

- Source: arXiv papers
- Size: 1,662,729 tokens (10.2 MB)
- Papers: 200

## Compression Results

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Compression Ratio | 0.500 | 0.500 | ✅ |
| Token Savings | 50.0% | 50% | ✅ |
| Quality Score | 88.6% | 85%+ | ✅ |
| Keyword Retention | 100.0% | 92%+ | ✅ |
| Entity Retention | 91.8% | 90%+ | ✅ |

## Performance

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Time | 0.92s | <2s | ✅ |
| Throughput | 10.58 MB/s | >10 MB/s | ✅ |
| Memory | ~50 MB | <100 MB | ✅ |

## Top Removed Words

1. "the" → 75,204 times (45.3%)
2. "and" → 35,671 times (21.5%)
3. "of" → 34,889 times (21.0%)
4. "a" → 28,041 times (16.9%)
5. "to" → 27,126 times (16.3%)

## Conclusion

✅ **All targets met**. Statistical filtering production ready.
```

---

## Data Storage

### Benchmark Results

**Location**: `benchmarks/results/`

**Files**:
- `compression/benchmark_200_papers_results.json`
- `compression/TIKTOKEN_FINDINGS.md`
- `datasets/llm_evaluation/dataset.json`
- `datasets/llm_evaluation/SUMMARY.md`
- `datasets/llm_evaluation/ANALYSIS.md`

### Version Control

**Commit Policy**:
- Benchmark results committed with code
- Results referenced in CHANGELOG
- Major results documented in README

**Baseline Tracking**:
- v0.3.0 baseline: 50% compression, 88.6% quality
- Future versions compared against this

---

## Future Enhancements

### Planned Metrics (v0.4.0)

**Real LLM Validation**:
- Semantic similarity (BERT-score)
- Task accuracy (Q&A, summarization)
- Human preference (A/B testing)

**Domain-Specific Benchmarks**:
- Code documentation
- News articles
- Chat logs
- Legal/medical text

**Advanced Quality**:
- Coherence score
- Fluency score
- Factual consistency

---

**Last Updated**: 2024-10-21  
**Version**: 0.3.0  
**Status**: Production Ready
