# arXiv Benchmark Dataset Specification

## Overview

Primary benchmark dataset using real academic papers from arXiv converted to Markdown for realistic, large-scale compression testing.

## Dataset Construction

### Source Material

**Source**: arXiv.org academic papers (PDFs)  
**Conversion Tool**: Transmutation (HiveLLM PDF → Markdown converter)  
**Format**: Markdown with preserved structure (citations, equations, sections)

### Why arXiv Papers?

✅ **Highly Repetitive Structure**:
- Bibliographies with repeated author names, venues, years
- LaTeX formatting patterns (equations, citations)
- Standard section headers (Abstract, Introduction, Methods, etc.)
- Common phrases in academic writing

✅ **Real-World Content**:
- Not synthetic/generated data
- Actual use case (researchers analyzing papers)
- Natural language + technical content mix

✅ **Scalable**:
- Millions of papers available
- Easy to scale from 100 to 10,000+ papers
- Controlled dataset sizes

✅ **Objective Validation**:
- Can ask specific questions (author names, paper titles, methodologies)
- Ground truth available in original PDFs
- Verifiable comprehension metrics

## Dataset Variants

### Small Scale (100 Papers)

**Purpose**: Quick validation, development testing

**Characteristics**:
- ~100 papers from cs.AI, cs.CL, cs.LG categories
- Estimated size: 5-10 MB Markdown (~1.5-3M tokens)
- Conversion time: ~5-10 minutes (transmutation)
- Compression time target: <30 seconds

**Use Cases**:
- Algorithm development
- Quick iteration testing
- CI/CD automated tests

### Medium Scale (200 Papers)

**Purpose**: Standard benchmark

**Characteristics**:
- 200 papers from mixed CS categories
- Estimated size: 10-20 MB (~3-6M tokens)
- More diverse vocabulary, authors, topics
- Compression time target: <60 seconds

**Use Cases**:
- Primary benchmark for paper results
- Comparison baseline
- Quality validation

### Large Scale (500 Papers)

**Purpose**: Stress testing, scalability validation

**Characteristics**:
- 500 papers from broader categories
- Estimated size: 25-50 MB (~7-15M tokens)
- Tests algorithm scalability
- Compression time target: <3 minutes

**Use Cases**:
- Performance profiling
- Memory usage analysis
- Dictionary saturation testing

### Extra Large Scale (1000 Papers)

**Purpose**: Production scenario simulation

**Characteristics**:
- 1000 papers (full repository analysis simulation)
- Estimated size: 50-100 MB (~15-30M tokens)
- Maximum compression challenge
- Compression time target: <10 minutes

**Use Cases**:
- Worst-case performance
- Production readiness validation
- Long-tail compression gains

## Dataset Curation

### Paper Selection Criteria

1. **Diversity**: Mix of subfields (AI, ML, NLP, CV, etc.)
2. **Recency**: Papers from 2020-2024 (modern writing style)
3. **Length**: Standard conference/journal length (8-20 pages)
4. **Quality**: Well-cited, established venues

### Excluded Papers

- ❌ Very short papers (<4 pages)
- ❌ Extremely long papers (>50 pages, books)
- ❌ Papers with excessive equations (reduces text repetition)
- ❌ Non-English papers

### Sample arXiv Categories

- `cs.AI` - Artificial Intelligence
- `cs.CL` - Computation and Language
- `cs.LG` - Machine Learning
- `cs.CV` - Computer Vision
- `cs.IR` - Information Retrieval

## Conversion Pipeline

### Step 1: Download Papers

```bash
# Download 100 papers from cs.CL
arxiv-downloader --category cs.CL --count 100 --output ./arxiv_pdfs/
```

**Tools**: 
- `arxiv` Python package
- `arxiv-downloader` CLI
- Manual download from arXiv.org

### Step 2: Convert to Markdown

```bash
# Convert using transmutation
cd transmutation/
for pdf in ../arxiv_pdfs/*.pdf; do
    cargo run --release -- convert "$pdf" \
        --output "../arxiv_markdown/$(basename "$pdf" .pdf).md" \
        --format markdown
done
```

**Transmutation Settings**:
- Format: Markdown
- Preserve equations: Yes
- Extract citations: Yes
- OCR fallback: No (arXiv PDFs are text-based)

### Step 3: Concatenate into Prompt

```bash
# Create single concatenated file
cat arxiv_markdown/*.md > benchmark_100_papers.txt

# Add separators for clarity (optional)
python scripts/concat_papers.py \
    --input arxiv_markdown/ \
    --output benchmark_100_papers.txt \
    --separator "\n\n---PAPER_SEPARATOR---\n\n"
```

### Step 4: Validate Dataset

```python
# Verify token counts
from compression_core import MockTokenizer

tokenizer = MockTokenizer()
with open('benchmark_100_papers.txt') as f:
    content = f.read()
    
token_count = tokenizer.count_tokens(content)
print(f"Total tokens: {token_count:,}")
print(f"Size: {len(content) / 1024 / 1024:.2f} MB")
```

## Benchmark Metrics

### Primary Metrics

#### 1. Compression Ratio

```
compression_ratio = compressed_tokens / original_tokens
```

**Expected Results**:
- 100 papers: 0.60-0.75 (25-40% savings)
- 200 papers: 0.55-0.70 (30-45% savings)
- 500 papers: 0.50-0.65 (35-50% savings)
- 1000 papers: 0.45-0.60 (40-55% savings)

**Hypothesis**: More papers → better compression (more repetition)

#### 2. Compression Time

Measure end-to-end compression latency:

```rust
let start = Instant::now();
let result = compressor.compress(&input, &tokenizer)?;
let duration = start.elapsed();
```

**Target Performance**:
- 100 papers (~2M tokens): <30s
- 200 papers (~4M tokens): <60s
- 500 papers (~10M tokens): <3min
- 1000 papers (~20M tokens): <10min

#### 3. Dictionary Characteristics

- Number of entries (should saturate at max_dict_entries)
- Average gain per entry
- Most common n-grams (bibliographic patterns expected)
- Distribution of entry frequencies

#### 4. Token Savings

```
savings = original_tokens - compressed_tokens
savings_pct = (1 - compression_ratio) * 100
```

**Target Savings**:
- 100 papers: 500K-1M tokens saved
- 1000 papers: 8M-12M tokens saved

### Secondary Metrics

#### 5. Memory Usage

Peak RSS during compression:

```bash
/usr/bin/time -v cargo run --release -- compress benchmark_100_papers.txt
# Check "Maximum resident set size"
```

**Target**: <5× input size peak memory

#### 6. Dictionary Overhead

```
overhead_ratio = dict_header_tokens / compressed_total_tokens
```

**Target**: <25% (should decrease as dataset grows)

## Quality Validation with LLMs

### Test Queries

For each dataset size, submit compressed prompt to LLMs with validation queries:

#### Query Set 1: Factual Retrieval

```
Q1: List all papers authored by "Geoffrey Hinton" in this collection.
Q2: What are the main contributions of the paper titled "[specific title]"?
Q3: How many papers discuss "transformer architectures"?
Q4: Which papers were published in 2023?
```

**Validation**: Compare answers against ground truth (paper metadata)

#### Query Set 2: Comprehension

```
Q5: Summarize the main research trends across these papers.
Q6: Compare the approaches to [specific problem] in papers A, B, and C.
Q7: What are the common evaluation metrics used?
```

**Validation**: Manual review for coherence and accuracy

#### Query Set 3: Cross-Paper Reasoning

```
Q8: Which papers cite paper X?
Q9: What are the most frequently cited works across all papers?
Q10: Identify papers that propose similar architectures.
```

**Validation**: Check against bibliographies in original papers

### LLM Testing Matrix

| LLM | Context Window | Cost/M tokens | Test Sizes |
|-----|----------------|---------------|------------|
| Claude Sonnet 3.5 | 200K | $3.00 | 100, 200 |
| Claude Opus 3 | 200K | $15.00 | 100 |
| GPT-4 Turbo | 128K | $10.00 | 100 |
| GPT-4o | 128K | $5.00 | 100, 200 |
| Gemini Pro 1.5 | 2M | $1.25 | 100, 200, 500, 1000 |

**Note**: Gemini Pro 1.5 can handle all sizes due to 2M context window

### A/B Testing Protocol

For each (dataset_size, query) pair:

1. **Baseline (Original)**:
   - Submit uncompressed prompt
   - Record: response, latency, tokens used

2. **Treatment (Compressed)**:
   - Submit compressed prompt (with RULES + DICT + BODY)
   - Record: response, latency, tokens used

3. **Comparison**:
   - **Accuracy**: Response matches baseline? (binary)
   - **Quality**: Human rating 1-5 (if different from baseline)
   - **Latency Δ**: (compressed_time - original_time) / original_time
   - **Token Savings**: original_tokens - compressed_tokens

### Quality Scoring

**Automated Metrics**:
- Exact match (for factual queries)
- F1 score (for lists of papers/authors)
- BLEU score (for summaries)

**Manual Review** (sample 10% of responses):
- Semantic equivalence (5-point scale)
- Information completeness (5-point scale)
- Hallucination detection (binary)

## Expected Repetition Patterns

### High-Frequency N-grams (Predicted)

1. **Bibliography Patterns**:
   ```
   "In Proceedings of"
   "Conference on Neural Information Processing Systems"
   "arXiv preprint arXiv:"
   "et al."
   ```

2. **Section Headers**:
   ```
   "## Introduction"
   "## Related Work"
   "## Experiments"
   "## Conclusion"
   ```

3. **LaTeX Remnants**:
   ```
   "\\cite{"
   "\\ref{fig:"
   "\\textbf{"
   ```

4. **Academic Phrases**:
   ```
   "we propose"
   "in this paper"
   "state-of-the-art"
   "experimental results show"
   ```

## Dataset Storage

### Directory Structure

```
benchmarks/
├── datasets/
│   ├── arxiv_pdfs/           # Original PDFs
│   │   ├── 2301.00001v1.pdf
│   │   ├── 2301.00002v1.pdf
│   │   └── ...
│   ├── arxiv_markdown/       # Converted Markdown
│   │   ├── 2301.00001v1.md
│   │   ├── 2301.00002v1.md
│   │   └── ...
│   ├── prompts/              # Concatenated prompts
│   │   ├── benchmark_100_papers.txt
│   │   ├── benchmark_200_papers.txt
│   │   ├── benchmark_500_papers.txt
│   │   └── benchmark_1000_papers.txt
│   └── metadata/             # Paper metadata (for validation)
│       ├── papers_100.json   # titles, authors, years, etc.
│       └── ...
├── results/
│   ├── compression/          # Compression results
│   │   ├── 100_papers_result.json
│   │   └── ...
│   └── llm_validation/       # A/B test results
│       ├── 100_papers_claude_sonnet.json
│       └── ...
└── scripts/
    ├── download_papers.py    # arXiv downloader
    ├── convert_papers.sh     # Transmutation batch conversion
    ├── concat_papers.py      # Create concatenated prompts
    └── validate_quality.py   # LLM A/B testing
```

### Metadata Format

`papers_100.json`:
```json
{
  "papers": [
    {
      "arxiv_id": "2301.00001",
      "title": "Example Paper Title",
      "authors": ["Author A", "Author B"],
      "year": 2023,
      "category": "cs.CL",
      "abstract": "...",
      "pdf_path": "arxiv_pdfs/2301.00001v1.pdf",
      "markdown_path": "arxiv_markdown/2301.00001v1.md"
    }
  ],
  "total_papers": 100,
  "total_tokens_estimate": 2000000,
  "categories": ["cs.CL", "cs.AI", "cs.LG"]
}
```

## Reproducibility

### Version Control

- Commit exact list of arXiv IDs used
- Pin transmutation version
- Document tokenizer versions

### Random Seed

Set random seed when selecting papers:
```python
import random
random.seed(42)
papers = random.sample(all_papers, 100)
```

### Docker Environment

Provide Dockerfile for reproducible environment:
```dockerfile
FROM rust:1.85-nightly
# Install transmutation
# Install Python tools
# Run benchmark scripts
```

## Cost Estimation

### LLM API Costs (per test run)

Assuming 100 papers = 2M tokens, compressed to 1.4M tokens (30% savings):

| Model | Original Cost | Compressed Cost | Savings |
|-------|---------------|-----------------|---------|
| Claude Sonnet | $6.00 | $4.20 | $1.80 |
| GPT-4 Turbo | $20.00 | $14.00 | $6.00 |
| Gemini Pro 1.5 | $2.50 | $1.75 | $0.75 |

**Total validation cost estimate** (all models, all sizes): ~$200-500

## Timeline

1. **Week 1**: Download and convert papers (100, 200, 500, 1000)
2. **Week 2**: Run compression benchmarks, collect metrics
3. **Week 3**: LLM A/B testing, quality validation
4. **Week 4**: Analysis, paper update, results documentation

## Success Criteria

- ✅ Compression ratio <0.75 for all dataset sizes
- ✅ Quality accuracy ≥95% on factual queries
- ✅ No catastrophic failures (LLM refuses prompt or hallucinates)
- ✅ Compression time scales sub-quadratically with input size
- ✅ Dictionary overhead <30% across all sizes

