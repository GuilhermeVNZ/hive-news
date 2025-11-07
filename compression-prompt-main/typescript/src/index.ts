/**
 * Compression Prompt - Fast statistical compression for LLM prompts
 *
 * Statistical compression using intelligent filtering to achieve 50% token reduction
 * with 91% quality retention.
 */

export { Compressor, CompressorConfig, CompressionResult, OutputFormat } from './compressor';
export { StatisticalFilter, StatisticalFilterConfig, WordImportance } from './statistical-filter';
export { QualityMetrics } from './quality-metrics';
export {
  ImageRenderer,
  ImageRendererConfig,
  ImageFormat,
  ImageError,
  TextTooLargeError,
  DEFAULT_IMAGE_RENDERER_CONFIG,
} from './image-renderer';

export const VERSION = '0.1.0';
