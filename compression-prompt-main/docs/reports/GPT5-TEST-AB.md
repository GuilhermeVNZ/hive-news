# GPT-5 A/B Compression Quality Report

Generated: 2025-10-21
Scope: A/B evaluation over llm_tests pairs (original vs compressed)

## Datasets

- 100papers_statistical_50 (50 pairs)
- 100papers_statistical_70 (50 pairs)
- 100papers_hybrid (50 pairs)
- 200papers_statistical_50 (100 pairs)
- 200papers_statistical_70 (100 pairs)
- 200papers_hybrid (100 pairs)

Source folders: `benchmarks/llm_tests/*`

## Methodology

- Compare ORIGINAL vs COMPRESSED for each pair
- Compute:
  - Compression ratio (tokens)
  - Keyword retention
  - Entity retention
  - Overall quality score (weighted):
    - 40% keywords, 30% entities, 20% vocabulary diversity, 10% information density
- Produce per-dataset averages and best/worst samples

Tooling:
- Rust example: `rust/examples/evaluate_compression_quality.rs`
- Fresh run output: `benchmarks/llm_tests/MY_QUALITY_EVAL.txt`

## Results (Averages)

| Dataset | Pairs | Compression | Quality | Keywords | Entities | Rating |
|--------|------:|------------:|--------:|---------:|---------:|:------:|
| 100papers_statistical_70 | 50  | 30.1% | 95.9% | 99.1% | 98.4% | 🟢 Excellent |
| 200papers_statistical_70 | 100 | 30.1% | 95.7% | 99.2% | 97.9% | 🟢 Excellent |
| 100papers_hybrid         | 50  | 50.3% | 89.3% | 91.3% | 90.6% | 🟡 Good |
| 100papers_statistical_50 | 50  | 50.1% | 89.2% | 91.3% | 90.7% | 🟡 Good |
| 200papers_hybrid         | 100 | 50.2% | 89.1% | 91.6% | 89.5% | 🟡 Good |
| 200papers_statistical_50 | 100 | 50.1% | 89.0% | 91.7% | 89.6% | 🟡 Good |

Notes:
- Statistical 70% (conservative compression) achieves ~96% quality with ~30% token savings.
- Statistical 50% and Hybrid provide ~50% token savings with ~89% quality.

## Best / Worst Samples (Examples)

- Statistical 70% (200 papers)
  - Best:  test_079 (quality: 98.0%, keywords: 100.0%)
  - Worst: test_055 (quality: 89.7%, keywords: 94.9%)
- Statistical 50% (200 papers)
  - Best:  test_068 (quality: 95.8%, keywords: 98.7%)
  - Worst: test_055 (quality: 76.1%, keywords: 82.9%)
- Hybrid (200 papers)
  - Best:  test_017 (quality: 97.2%, keywords: 95.7%)
  - Worst: test_055 (quality: 76.1%, keywords: 82.9%)

(See `MY_QUALITY_EVAL.txt` for full breakdown.)

## Recommendations

- Production default: Statistical 50%
  - 50% cost reduction with ~89% quality
  - Keyword retention ~92%, entity retention ~90%
- High-fidelity: Statistical 70%
  - 30% cost reduction with ~96% quality
  - Near-perfect keyword/entity retention
- Hybrid: same quality band as Statistical 50% with added complexity; use only if needed.

## Reproduce

- Regenerate pairs:
  - `cargo run --release --example extract_texts_for_llm`
- Re-run quality evaluation:
  - `cargo run --release --example evaluate_compression_quality > benchmarks/llm_tests/MY_QUALITY_EVAL.txt`

Artifacts:
- Fresh report: `benchmarks/llm_tests/MY_QUALITY_EVAL.txt`
- This summary: `benchmarks/GPT5-TEST-AB.md`
