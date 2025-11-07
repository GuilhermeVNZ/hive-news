/**
 * Configuration for statistical filtering
 */
export interface StatisticalFilterConfig {
  /** Target compression ratio (0.0-1.0). 0.5 = keep 50% of tokens */
  compressionRatio: number;

  /** Weight for inverse document frequency (rare words) */
  idfWeight: number;

  /** Weight for position in document (start/end more important) */
  positionWeight: number;

  /** Weight for part-of-speech heuristics */
  posWeight: number;

  /** Weight for named entity patterns */
  entityWeight: number;

  /** Weight for local entropy (vocabulary diversity) */
  entropyWeight: number;

  /** Enable protection masks for code/JSON/paths */
  enableProtectionMasks: boolean;

  /** Enable contextual stopword filtering */
  enableContextualStopwords: boolean;

  /** Preserve negations (not, never, don't, etc.) */
  preserveNegations: boolean;

  /** Preserve comparators (!=, <=, >=, etc.) */
  preserveComparators: boolean;

  /** Domain-specific terms to always preserve */
  domainTerms: string[];

  /** Minimum gap between critical tokens */
  minGapBetweenCritical: number;
}

/**
 * Result of compression operation
 */
export interface CompressionResult {
  /** The compressed text */
  compressed: string;

  /** Original token count */
  originalTokens: number;

  /** Compressed token count */
  compressedTokens: number;

  /** Compression ratio (compressed/original) */
  compressionRatio: number;

  /** Number of tokens removed */
  tokensRemoved: number;

  /** Optional image data (for future image output support) */
  imageData?: Uint8Array;

  /** Output format used */
  format?: 'text' | 'image';
}

/**
 * Quality metrics for compression
 */
export interface QualityMetrics {
  /** Overall quality score (0-1) */
  overallScore: number;

  /** Keyword retention percentage */
  keywordRetention: number;

  /** Entity retention percentage */
  entityRetention: number;

  /** Vocabulary diversity ratio */
  vocabularyRatio: number;

  /** Information density */
  informationDensity: number;
}

/**
 * Word importance score
 */
export interface WordImportance {
  /** Position in text */
  position: number;

  /** Word text */
  text: string;

  /** Combined importance score */
  score: number;

  /** Original character position in the source text */
  charPosition: number;
}

/**
 * Protected span type
 */
export enum SpanType {
  CodeBlock = 'CodeBlock',
  JsonBlock = 'JsonBlock',
  Path = 'Path',
  Identifier = 'Identifier',
  HashOrNumber = 'HashOrNumber',
  Bracket = 'Bracket',
}

/**
 * Protected text span
 */
export interface ProtectedSpan {
  start: number;
  end: number;
  spanType: SpanType;
}
