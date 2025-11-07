/**
 * Main compression pipeline and result structures
 */

import { StatisticalFilter, StatisticalFilterConfig } from './statistical-filter';
import { ImageRenderer } from './image-renderer';

export enum OutputFormat {
  TEXT = 'text',
  IMAGE = 'image',
}

export class CompressionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CompressionError';
  }
}

export class NegativeGainError extends CompressionError {
  constructor(public ratio: number) {
    super(`Compression ratio ${ratio.toFixed(2)} >= 1.0, would increase tokens`);
    this.name = 'NegativeGainError';
  }
}

export class InputTooShortError extends CompressionError {
  constructor(
    public size: number,
    public minimum: number
  ) {
    super(`Input too short (${size} tokens/bytes), minimum is ${minimum}`);
    this.name = 'InputTooShortError';
  }
}

export interface CompressorConfig {
  targetRatio: number;
  minInputTokens: number;
  minInputBytes: number;
}

export interface CompressionResult {
  compressed: string;
  imageData?: Buffer;
  format: OutputFormat;
  originalTokens: number;
  compressedTokens: number;
  compressionRatio: number;
  tokensRemoved: number;
}

export interface CompressWithFormatOptions {
  imageFormat?: 'png' | 'jpeg';
  jpegQuality?: number;
}

export class Compressor {
  private config: CompressorConfig;
  private filter: StatisticalFilter;

  constructor(config?: Partial<CompressorConfig>, filterConfig?: Partial<StatisticalFilterConfig>) {
    this.config = {
      targetRatio: 0.5,
      minInputTokens: 100,
      minInputBytes: 1024,
      ...config,
    };

    const finalFilterConfig: StatisticalFilterConfig = {
      compressionRatio: this.config.targetRatio,
      idfWeight: 0.3,
      positionWeight: 0.2,
      posWeight: 0.2,
      entityWeight: 0.2,
      entropyWeight: 0.1,
      enableProtectionMasks: true,
      enableContextualStopwords: true,
      preserveNegations: true,
      preserveComparators: true,
      domainTerms: ['Vectorizer', 'Synap', 'UMICP', 'Graphs'],
      minGapBetweenCritical: 3,
      ...filterConfig,
    };

    this.filter = new StatisticalFilter(finalFilterConfig);
  }

  compress(inputText: string): CompressionResult {
    return this.compressWithFormat(inputText, OutputFormat.TEXT);
  }

  compressWithFormat(
    inputText: string,
    format: OutputFormat,
    options?: CompressWithFormatOptions
  ): CompressionResult {
    // Step 1: Check input size (bytes)
    const inputBytes = Buffer.byteLength(inputText, 'utf8');
    if (inputBytes < this.config.minInputBytes) {
      throw new InputTooShortError(inputBytes, this.config.minInputBytes);
    }

    // Step 2: Estimate tokens (using char count / 4 as rough estimate)
    const originalTokens = Math.floor(inputText.length / 4);
    if (originalTokens < this.config.minInputTokens) {
      throw new InputTooShortError(originalTokens, this.config.minInputTokens);
    }

    // Step 3: Apply statistical filtering
    const compressed = this.filter.compress(inputText);

    // Step 4: Validate compression ratio
    const compressedTokens = Math.floor(compressed.length / 4);
    const compressionRatio = originalTokens > 0 ? compressedTokens / originalTokens : 1.0;

    if (compressionRatio >= 1.0) {
      throw new NegativeGainError(compressionRatio);
    }

    const tokensRemoved = Math.max(0, originalTokens - compressedTokens);

    // Step 5: Generate image if requested
    let imageData: Buffer | undefined;
    if (format === OutputFormat.IMAGE) {
      try {
        const renderer = new ImageRenderer();
        const imageFormat = options?.imageFormat || 'png';

        if (imageFormat === 'jpeg') {
          const quality = options?.jpegQuality || 85;
          imageData = renderer.renderToJpeg(compressed, quality);
        } else {
          imageData = renderer.renderToPng(compressed);
        }
      } catch (error) {
        throw new CompressionError(
          `Failed to render image: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    }

    return {
      compressed,
      imageData,
      format,
      originalTokens,
      compressedTokens,
      compressionRatio,
      tokensRemoved,
    };
  }
}
