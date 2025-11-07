# Compression Prompt - TypeScript/JavaScript Implementation

> Fast, intelligent prompt compression for LLMs - Save 50% tokens while maintaining 91% quality

TypeScript/JavaScript port of the Rust implementation. Achieves **50% token reduction** with **91% quality retention** using pure statistical filtering.

## Quick Start

### Installation

```bash
npm install @hivellm/compression-prompt
```

Or from source:

```bash
cd typescript
npm install
npm run build
```

### Basic Usage

```typescript
import { Compressor } from '@hivellm/compression-prompt';

const compressor = new Compressor();

const text = `
Your long text here...
This will be compressed using statistical filtering
to save 50% tokens while maintaining quality.
`;

const result = compressor.compress(text);

console.log(`Original: ${result.originalTokens} tokens`);
console.log(`Compressed: ${result.compressedTokens} tokens`);
console.log(`Saved: ${result.tokensRemoved} tokens (${(1-result.compressionRatio)*100}%)`);
console.log(`\nCompressed text:\n${result.compressed}`);
```

### Advanced Configuration

```typescript
import { Compressor } from '@hivellm/compression-prompt';

const compressor = new Compressor(
  { targetRatio: 0.7 },  // Keep 70% of tokens
  {
    compressionRatio: 0.7,
    idfWeight: 0.3,
    positionWeight: 0.2,
    posWeight: 0.2,
    entityWeight: 0.2,
    entropyWeight: 0.1,
    domainTerms: ['YourTerm'],
  }
);

const result = compressor.compress(text);
```

### Quality Metrics

```typescript
import { QualityMetricsCalculator } from '@hivellm/compression-prompt';

const metrics = QualityMetricsCalculator.calculate(original, compressed);
console.log(QualityMetricsCalculator.format(metrics));
```

### Image Compression for Vision Models

Render compressed text as images for optical context compression (inspired by DeepSeek-OCR):

```typescript
import { Compressor, OutputFormat } from '@hivellm/compression-prompt';

const compressor = new Compressor();

// Generate PNG image
const result = compressor.compressWithFormat(text, OutputFormat.IMAGE);

if (result.imageData) {
  fs.writeFileSync('compressed.png', result.imageData);
  console.log(`Image: ${result.imageData.length} bytes`);
}

// Generate JPEG with custom quality
const jpegResult = compressor.compressWithFormat(
  text, 
  OutputFormat.IMAGE,
  { imageFormat: 'jpeg', jpegQuality: 90 }
);
```

### Command Line Usage

```bash
# Text compression
npx @hivellm/compression-prompt input.txt
npx @hivellm/compression-prompt -r 0.7 input.txt              # Conservative (70%)
npx @hivellm/compression-prompt -s input.txt                  # Show statistics
npx @hivellm/compression-prompt -o output.txt input.txt       # Save to file

# Image compression
npx @hivellm/compression-prompt -f png -o output.png input.txt              # Generate PNG
npx @hivellm/compression-prompt -f jpeg -o output.jpg input.txt             # Generate JPEG
npx @hivellm/compression-prompt -f jpeg --jpeg-quality 95 -s input.txt      # High-quality JPEG
```

## Features

- ✅ **Statistical Compression**: 50% token reduction with 91% quality retention
- ✅ **Image Output**: Render as PNG/JPEG for vision models (optical context compression)
- ✅ **Fast**: Optimized statistical filtering
- ✅ **Type-Safe**: Full TypeScript support with type definitions
- ✅ **Node.js & Browser**: Works in both environments
- ✅ **Smart Filtering**: Preserves code, JSON, paths, identifiers
- ✅ **Customizable**: Fine-tune weights for your use case

## API Reference

### Compressor

```typescript
class Compressor {
  constructor(
    config?: Partial<CompressorConfig>,
    filterConfig?: Partial<StatisticalFilterConfig>
  );
  
  compress(text: string): CompressionResult;
  compressWithFormat(
    text: string, 
    format: OutputFormat,
    options?: CompressWithFormatOptions
  ): CompressionResult;
}

interface CompressWithFormatOptions {
  imageFormat?: 'png' | 'jpeg';
  jpegQuality?: number;  // 1-100, default: 85
}

enum OutputFormat {
  TEXT = 'text',
  IMAGE = 'image',
}
```

### CompressorConfig

```typescript
interface CompressorConfig {
  targetRatio: number;       // 0.0-1.0, default: 0.5
  minInputTokens: number;    // default: 100
  minInputBytes: number;     // default: 1024
}
```

### StatisticalFilterConfig

```typescript
interface StatisticalFilterConfig {
  compressionRatio: number;
  idfWeight: number;
  positionWeight: number;
  posWeight: number;
  entityWeight: number;
  entropyWeight: number;
  enableProtectionMasks: boolean;
  enableContextualStopwords: boolean;
  preserveNegations: boolean;
  preserveComparators: boolean;
  domainTerms: string[];
  minGapBetweenCritical: number;
}
```

### ImageRenderer

```typescript
import { ImageRenderer, ImageFormat } from '@hivellm/compression-prompt';

const renderer = new ImageRenderer({
  width: 1024,
  height: 1024,
  fontSize: 12.5,
  lineSpacing: 1.2,
  // ... more options
});

const pngData = renderer.renderToPng(text);
const jpegData = renderer.renderToJpeg(text, 85);  // quality: 1-100
```

## Examples

### Paper to Images

Convert academic papers or long documents to compressed images:

```bash
cd typescript
npx ts-node examples/paper-to-images.ts
```

Generates compressed images at different compression ratios (30%, 50%, 70%).

## Development

```bash
# Build
npm run build

# Test
npm test

# Lint
npm run lint

# Format
npm run format
```

## License

MIT

## See Also

- [Rust Implementation](../rust/) - High-performance original
- [Python Implementation](../python/) - Python port
- [Main README](../README.md) - Project overview
