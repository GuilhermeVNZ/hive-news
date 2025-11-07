/**
 * Quality metrics for compression evaluation (model-free)
 */

export interface QualityMetrics {
  keywordRetention: number;
  entityRetention: number;
  vocabularyRatio: number;
  informationDensity: number;
  overallScore: number;
}

export class QualityMetricsCalculator {
  private static readonly STOP_WORDS = new Set([
    'the',
    'a',
    'an',
    'and',
    'or',
    'but',
    'in',
    'on',
    'at',
    'to',
    'for',
    'of',
    'with',
    'by',
    'from',
    'as',
    'is',
    'was',
    'are',
    'were',
    'be',
    'been',
    'being',
    'have',
    'has',
    'had',
    'do',
    'does',
    'did',
    'will',
    'would',
    'should',
    'could',
    'may',
    'might',
    'must',
    'can',
    'this',
    'that',
    'these',
    'those',
    'we',
    'they',
    'it',
  ]);

  static calculate(original: string, compressed: string): QualityMetrics {
    const origWords = this.tokenize(original);
    const compWords = this.tokenize(compressed);

    const origKeywords = this.extractKeywords(origWords);
    const compKeywords = this.extractKeywords(compWords);

    const origEntities = this.extractEntities(origWords);
    const compEntities = this.extractEntities(compWords);

    const keywordRetention = this.calculateRetention(origKeywords, compKeywords);
    const entityRetention = this.calculateRetention(origEntities, compEntities);

    const origVocab = new Set(origWords.map(w => w.toLowerCase()));
    const compVocab = new Set(compWords.map(w => w.toLowerCase()));
    const vocabularyRatio = compVocab.size / Math.max(1, origVocab.size);

    const informationDensity = compWords.length > 0 ? compVocab.size / compWords.length : 0;

    const overallScore =
      keywordRetention * 0.4 +
      entityRetention * 0.3 +
      vocabularyRatio * 0.2 +
      informationDensity * 0.1;

    return {
      keywordRetention,
      entityRetention,
      vocabularyRatio,
      informationDensity,
      overallScore,
    };
  }

  static format(metrics: QualityMetrics): string {
    return `Quality Metrics:
- Keyword Retention: ${(metrics.keywordRetention * 100).toFixed(1)}%
- Entity Retention: ${(metrics.entityRetention * 100).toFixed(1)}%
- Vocabulary Ratio: ${(metrics.vocabularyRatio * 100).toFixed(1)}%
- Info Density: ${metrics.informationDensity.toFixed(3)}
- Overall Score: ${(metrics.overallScore * 100).toFixed(1)}%`;
  }

  private static tokenize(text: string): string[] {
    return text.split(/\s+/).filter(w => w.length > 0);
  }

  private static extractKeywords(words: string[]): Set<string> {
    const keywords = new Set<string>();

    for (const word of words) {
      const lower = word.toLowerCase();
      if (
        !this.STOP_WORDS.has(lower) &&
        (word.length > 5 ||
          (word.length > 0 && word[0] === word[0].toUpperCase()) ||
          word.includes('-') ||
          word.includes('_'))
      ) {
        keywords.add(lower);
      }
    }

    return keywords;
  }

  private static extractEntities(words: string[]): Set<string> {
    const entities = new Set<string>();

    for (let i = 0; i < words.length; i++) {
      const word = words[i];

      if (word.includes('@') || word.startsWith('http')) {
        entities.add(word.toLowerCase());
      }

      if (word.length > 1 && word === word.toUpperCase()) {
        entities.add(word);
      }

      if (word.length > 2 && word[0] === word[0].toUpperCase()) {
        if (i + 1 < words.length && words[i + 1][0] === words[i + 1][0].toUpperCase()) {
          entities.add(`${word} ${words[i + 1]}`);
        }
        entities.add(word);
      }
    }

    return entities;
  }

  private static calculateRetention(original: Set<string>, compressed: Set<string>): number {
    if (original.size === 0) {
      return 1.0;
    }

    let preserved = 0;
    for (const item of original) {
      if (compressed.has(item)) {
        preserved++;
      }
    }

    return preserved / original.size;
  }
}
