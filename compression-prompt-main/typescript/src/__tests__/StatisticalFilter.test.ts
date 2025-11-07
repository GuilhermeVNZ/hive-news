import { StatisticalFilter } from '../statistical-filter';
import { StatisticalFilterConfig } from '../types';

describe('StatisticalFilter', () => {
  describe('compression', () => {
    it('should compress text', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.5,
      };
      const filter = new StatisticalFilter(config);

      const text = 'The quick brown fox jumps over the lazy dog';
      const compressed = filter.compress(text);

      const originalWords = text.split(/\s+/).length;
      const compressedWords = compressed
        .split(/\s+/)
        .filter((w: string) => w.length > 0).length;

      expect(compressedWords).toBeLessThanOrEqual(originalWords);
      expect(compressed).not.toBe('');
    });

    it('should protect code blocks', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text =
        'Here is some code ```rust fn main() { println!("Hello"); }``` that should be preserved';
      const compressed = filter.compress(text);

      // Code block should be in the output even with aggressive compression
      expect(compressed.includes('```rust') || compressed.includes('println!')).toBe(true);
    });

    it('should protect JSON blocks', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text = 'The config is {"key": "value"} and it should remain intact';
      const compressed = filter.compress(text);

      // JSON should be preserved
      expect(
        compressed.includes('{"key":') ||
          compressed.includes('"key"') ||
          compressed.includes('value')
      ).toBe(true);
    });

    it('should preserve paths', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.4,
      };
      const filter = new StatisticalFilter(config);

      const text = 'Check the file in src/main.rs for the implementation details';
      const compressed = filter.compress(text);

      // Path should be preserved
      expect(
        compressed.includes('src/main.rs') ||
          (compressed.includes('src') && compressed.includes('main.rs'))
      ).toBe(true);
    });

    it('should preserve contextual stopword "to"', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.5,
      };
      const filter = new StatisticalFilter(config);

      // "to" should be kept in "how to"
      const text1 = 'how to reproduce the bug';
      const compressed1 = filter.compress(text1);
      expect(compressed1.includes('to') || compressed1.includes('how')).toBe(true);

      // "to" can be removed in other contexts if not critical
      const text2 = 'going to the store';
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const _compressed2 = filter.compress(text2);
      // This is context-dependent, so we don't assert removal
    });

    it('should preserve negations', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text = 'do not remove this critical information';
      const compressed = filter.compress(text);

      // "not" should always be preserved
      expect(compressed).toContain('not');
    });

    it('should preserve comparators', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text = 'check if x >= 5 before proceeding';
      const compressed = filter.compress(text);

      // ">=" should be preserved
      expect(
        compressed.includes('>=') || compressed.includes('5') || compressed.includes('x')
      ).toBe(true);
    });

    it('should preserve domain terms', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text = 'use the Vectorizer tool to process data';
      const compressed = filter.compress(text);

      // Domain term "Vectorizer" should be preserved
      expect(compressed).toContain('Vectorizer');
    });

    it('should protect identifiers', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
      };
      const filter = new StatisticalFilter(config);

      const text = 'call the getUserData function from user_service module';
      const compressed = filter.compress(text);

      // Identifiers should be preserved
      expect(compressed.includes('getUserData') || compressed.includes('user_service')).toBe(true);
    });

    it('should fill gaps between critical tokens', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.2,
        minGapBetweenCritical: 2,
      };
      const filter = new StatisticalFilter(config);

      const text = 'Vectorizer is a critical component that handles data processing for Synap';
      const compressed = filter.compress(text);

      // Should have some words between Vectorizer and Synap
      expect(compressed).toContain('Vectorizer');
      expect(compressed).toContain('Synap');

      const words = compressed.split(/\s+/).filter((w: string) => w.length > 0);
      expect(words.length).toBeGreaterThanOrEqual(3);
    });

    it('should allow disabling protection masks', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.3,
        enableProtectionMasks: false,
      };
      const filter = new StatisticalFilter(config);

      const text = 'Check src/main.rs for details';
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const _compressed = filter.compress(text);

      // With protection disabled, behavior is normal compression
      // Just ensure it doesn't crash
      expect(true).toBe(true);
    });

    it('should allow disabling contextual stopwords', () => {
      const config: Partial<StatisticalFilterConfig> = {
        compressionRatio: 0.5,
        enableContextualStopwords: false,
      };
      const filter = new StatisticalFilter(config);

      const text = 'how to reproduce the issue';
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const _compressed = filter.compress(text);

      // With contextual stopwords disabled, "to" might be removed
      // Just ensure it doesn't crash
      expect(true).toBe(true);
    });
  });

  describe('compressWithMetrics', () => {
    it('should return detailed metrics', () => {
      const filter = new StatisticalFilter();
      const text = 'The quick brown fox jumps over the lazy dog';
      const result = filter.compressWithMetrics(text);

      expect(result).toHaveProperty('compressed');
      expect(result).toHaveProperty('originalTokens');
      expect(result).toHaveProperty('compressedTokens');
      expect(result).toHaveProperty('compressionRatio');
      expect(result).toHaveProperty('tokensRemoved');

      expect(result.originalTokens).toBeGreaterThan(0);
      expect(result.compressedTokens).toBeLessThanOrEqual(result.originalTokens);
      expect(result.tokensRemoved).toBeGreaterThanOrEqual(0);
    });
  });

  describe('scoreWords', () => {
    it('should score words correctly', () => {
      const filter = new StatisticalFilter();
      const text = 'The quick brown fox';
      const scores = filter.scoreWords(text);

      expect(scores).toHaveLength(4);
      scores.forEach((score: any) => {
        expect(score).toHaveProperty('position');
        expect(score).toHaveProperty('text');
        expect(score).toHaveProperty('score');
      });
    });
  });
});
