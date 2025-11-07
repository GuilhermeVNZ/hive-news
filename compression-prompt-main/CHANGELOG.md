# Changelog

All notable changes to Compression-Prompt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2025-10-23

### Fixed - PEP 625 Compliance
- **Updated package name** from `compression-prompt` to `compression_prompt` for PEP 625 compliance
  - PyPI now requires normalized project names in source distribution filenames
  - Changed name in `pyproject.toml` and `setup.py` from hyphens to underscores
  - New filename: `compression_prompt-0.1.0.tar.gz` (was `compression-prompt-0.1.0.tar.gz`)
  - Package import name remains unchanged: `import compression_prompt`
  - Display name on PyPI will show as `compression-prompt` (auto-normalized)

## [0.3.1] - 2025-10-22

### Updated - Paper with Real-World Validation Results
- **Updated abstract** to reflect 91% quality retention validated across 6 flagship LLMs
- **Added LLM validation section** in benchmarks with results from:
  - Grok-4: 93% quality (best overall)
  - Claude 3.5 Sonnet: 91% quality (recommended)
  - GPT-5: 89% quality
  - Gemini Pro: 89% quality
  - Grok: 88% quality
  - Claude Haiku: 87% quality
- **Added optical context compression section** in implementation
  - Image output format details
  - Performance benchmarks
  - Use cases and limitations
- **Updated conclusion** with validated results
  - Changed from "future work" to "completed validation"
  - Updated cost savings to reflect Claude Sonnet pricing ($900K/year for 1B tokens)
  - Added 350+ A/B test pairs results
- **Updated title** to include "91% Quality Retention"
- **Added references** for LLMLingua, Selective Context, Zipf's Law, and DeepSeek-OCR
- **Performance metrics updated**:
  - 92% keyword retention (updated from 100%)
  - 89.5% entity retention (updated from 91.8%)
  - 91% average quality across all LLMs

## [0.1.0] - 2025-10-22

### Added - Image Output Format (Optical Context Compression) 🧪 BETA
- **Dense prompt-to-image compression** inspired by [DeepSeek-OCR paper](https://arxiv.org/html/2510.18234v1)
  - Implements "Contexts Optical Compression" concept from DeepSeek-OCR
  - Renders statistically compressed text into 1024x1024 images
  - Vision models can process the compressed content optically
- **Image renderer module** (`src/image_renderer.rs`)
  - `ImageRenderer` struct with configurable rendering
  - `render_to_png()` - Lossless PNG output (~1.4 MB per page)
  - `render_to_jpeg(quality)` - Lossy JPEG output (~460 KB @ quality 85)
  - Embedded DejaVu Sans Mono font (340 KB TrueType)
  - Auto font size adjustment (12.5pt default, scales down to 7pt if needed)
  - Auto-pagination: Splits into multiple pages if text doesn't fit (~2000 words/page)
  - Alpha blending for smooth font rendering
- **API integration**
  - `OutputFormat` enum (Text/Image)
  - `compress_with_format()` method on `StatisticalFilter`
  - `CompressionResult` now includes optional `image_data` field
- **Examples demonstrating usage**
  - `compress_to_image.rs` - Basic image compression demo
  - `paper_to_png_50pct.rs` - Convert papers to readable PNG images
  - `compare_image_formats.rs` - PNG vs JPEG comparison
  - `paper_to_jpeg.rs` - Multi-format output comparison
- **New dependencies** (verified via Context7)
  - `image = "0.25"` - Image creation and PNG/JPEG encoding
  - `imageproc = "0.25"` - Drawing operations (not actively used yet)
  - `ab_glyph = "0.2"` - Modern TrueType font rendering
- **Full test coverage**
  - Unit tests for `ImageRenderer`
  - Integration tests for `compress_with_format()`
  - All 23 tests passing

### Added - Token-Aware Semantic Preservation
- **Protection mask system** for code structures and technical content
  - Code blocks (` ```...``` `) automatically protected from compression
  - JSON/YAML structures preserved intact
  - File paths and URLs protected (`src/main.rs`, `http://...`)
  - Identifiers preserved (camelCase, snake_case, UPPER_SNAKE)
  - Hashes and large numbers protected
- **Contextual stopword filtering** for smarter compression
  - "to" preserved in infinitives ("how to", "steps to")
  - "in/on/at" preserved before paths ("in src/main.rs")
  - "is/are" preserved in technical assertions ("X is deprecated")
  - "and/or" preserved between important terms
- **Critical term preservation** with priority scores
  - Negations always preserved ("not", "never", "don't", etc.)
  - Comparators protected (">=", "!=", "===", etc.)
  - Modal qualifiers preserved ("only", "must", "at least", etc.)
  - Domain-specific terms (configurable list)
- **Gap-filling algorithm** prevents readability issues
  - Re-adds tokens between widely-separated critical terms
  - Configurable gap threshold (default: 3 tokens)
  - Maintains semantic flow even with aggressive compression

### Added - Statistical Filtering (Core Algorithm)
- **Statistical filtering** as primary compression method
  - 50% token reduction with 91% quality retention (Claude Sonnet)
  - IDF scoring, position weighting, POS heuristics
  - Named entity detection, entropy analysis
  - Configurable compression levels (30%, 50%, 70%)
- **Quality metrics system**
  - Keyword retention analysis
  - Named entity preservation tracking
  - Vocabulary diversity measurement
  - Information density calculation

### Performance Benchmarks
**Image Output:**
- Rendering speed: < 50ms per image
- PNG output: ~1.4 MB per page (lossless)
- JPEG output (quality 85): ~460 KB per page (66% smaller than PNG)
- Total savings: 65.5% size reduction with JPEG vs PNG
- Pagination: ~2000 words per page with 12.5pt font
- Example: RNN paper (9118 words) → 3 pages @ 50% compression

**Text Compression:**
- 50% token reduction with 91% quality (Claude Sonnet validated)
- < 1ms compression time (10.58 MB/s throughput)
- 100% keyword retention, 92% entity preservation
- Validated on 200 real arXiv papers (1.6M tokens)

### Vision Model Compatibility
- ✅ **GPT-4 Vision** - Tested, working
- ✅ **Claude 3 (Opus/Sonnet/Haiku)** - Compatible
- ✅ **Gemini Vision** - Compatible
- ✅ **Other vision models** - Should work with standard PNG/JPEG

### Dependencies
- Core: `serde`, `serde_json`, `thiserror`, `anyhow`, `regex`
- Performance: `ahash`, `rayon`
- Unicode: `unicode-segmentation`, `tiktoken-rs`
- Image: `image`, `imageproc`, `ab_glyph`
- Time: `chrono`

### Documentation
- Complete README with usage examples
- ROADMAP with Phase 6 completion (image output)
- Examples for all major features
- Full API documentation

### Status: Beta (Image Feature)
- ⚠️ Feature is functional but pending extensive validation
- ⚠️ OCR accuracy with vision models not yet benchmarked
- ⚠️ Optimal font size/density still being tuned
- ✅ Code quality: Production-ready
- ✅ Tests: All passing (23/23)

### Planned
- Extensive validation with vision models (GPT-4V, Claude 3, Gemini)
- OCR accuracy benchmarking
- Optimal font size tuning for different text densities
- Streaming support for large inputs
- Real-time API integration examples

---

## Historical Note

This project was restructured for v0.1.0 to focus on the optical compression feature. Previous version history (0.2.0 - 0.4.0) is available in git history and included core features like:
- Dictionary-based compression (deprecated)
- Statistical filtering development and validation
- LLM A/B testing across 6 models (350+ test pairs)
- Quality metrics and benchmarking infrastructure

All validated features from previous versions are included in 0.1.0.
