/**
 * Statistical token importance filtering (LLMLingua-inspired, model-free)
 */

enum SpanType {
  CODE_BLOCK = 'code_block',
  JSON_BLOCK = 'json_block',
  PATH = 'path',
  IDENTIFIER = 'identifier',
  HASH_OR_NUMBER = 'hash_or_number',
  BRACKET = 'bracket',
}

interface ProtectedSpan {
  start: number;
  end: number;
  spanType: SpanType;
}

export interface WordImportance {
  position: number;
  text: string;
  score: number;
}

export interface StatisticalFilterConfig {
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

export class StatisticalFilter {
  /**
   * Default configuration - 50% compression with 89% quality retention
   * Validated on 20 real papers: 92% keyword retention, 90% entity retention
   * Speed: <0.2ms average
   * Token-aware enhancements: Protects code, contextual stopwords, preserves semantics
   */
  static readonly DEFAULT_CONFIG: StatisticalFilterConfig = {
    compressionRatio: 0.5, // Keep 50% of tokens (recommended)
    idfWeight: 0.3,
    positionWeight: 0.2,
    posWeight: 0.2,
    entityWeight: 0.2,
    entropyWeight: 0.1,
    // Token-aware semantic preservation (all enabled by default)
    enableProtectionMasks: true,
    enableContextualStopwords: true,
    preserveNegations: true,
    preserveComparators: true,
    domainTerms: ['Vectorizer', 'Synap', 'UMICP', 'Graphs'],
    minGapBetweenCritical: 3,
  };

  private static readonly STOP_WORDS = new Set([
    // English
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
    'shall',
    'this',
    'that',
    'these',
    'those',
    'i',
    'you',
    'he',
    'she',
    'it',
    'we',
    'they',
    'what',
    'which',
    'who',
    'when',
    'where',
    'why',
    'how',
    // Spanish (Español)
    'el',
    'la',
    'los',
    'las',
    'un',
    'una',
    'unos',
    'unas',
    'y',
    'o',
    'pero',
    'en',
    'de',
    'del',
    'al',
    'para',
    'por',
    'con',
    'sin',
    'sobre',
    'entre',
    'hasta',
    'desde',
    'es',
    'son',
    'está',
    'están',
    'ser',
    'estar',
    'haber',
    'hacer',
    'tener',
    'decir',
    'ir',
    'ver',
    'dar',
    'saber',
    'querer',
    'poder',
    'poner',
    'este',
    'ese',
    'aquel',
    'mi',
    'tu',
    'su',
    'nuestro',
    'vuestro',
    'que',
    'quien',
    'cual',
    'cuando',
    'donde',
    'como',
    // Portuguese (Português)
    'o',
    'a',
    'os',
    'as',
    'um',
    'uma',
    'uns',
    'umas',
    'e',
    'ou',
    'mas',
    'em',
    'de',
    'do',
    'da',
    'dos',
    'das',
    'no',
    'na',
    'nos',
    'nas',
    'ao',
    'à',
    'aos',
    'às',
    'para',
    'por',
    'com',
    'sem',
    'sobre',
    'entre',
    'até',
    'desde',
    'é',
    'são',
    'está',
    'estão',
    'ser',
    'estar',
    'haver',
    'ter',
    'fazer',
    'dizer',
    'ir',
    'ver',
    'dar',
    'saber',
    'querer',
    'poder',
    'pôr',
    'este',
    'esse',
    'aquele',
    'meu',
    'teu',
    'seu',
    'nosso',
    'vosso',
    'que',
    'quem',
    'qual',
    'quando',
    'onde',
    'como',
    // French (Français)
    'le',
    'la',
    'les',
    'un',
    'une',
    'des',
    'et',
    'ou',
    'mais',
    'dans',
    'en',
    'de',
    'du',
    'au',
    'aux',
    'pour',
    'par',
    'avec',
    'sans',
    'sur',
    'sous',
    'entre',
    'vers',
    'chez',
    'est',
    'sont',
    'être',
    'avoir',
    'faire',
    'dire',
    'aller',
    'voir',
    'savoir',
    'pouvoir',
    'vouloir',
    'venir',
    'devoir',
    'prendre',
    'ce',
    'cet',
    'cette',
    'ces',
    'mon',
    'ton',
    'son',
    'notre',
    'votre',
    'leur',
    'que',
    'qui',
    'quoi',
    'dont',
    'où',
    'quand',
    'comment',
    // German (Deutsch)
    'der',
    'die',
    'das',
    'den',
    'dem',
    'des',
    'ein',
    'eine',
    'einer',
    'eines',
    'einem',
    'einen',
    'und',
    'oder',
    'aber',
    'in',
    'im',
    'an',
    'auf',
    'für',
    'von',
    'zu',
    'mit',
    'bei',
    'nach',
    'über',
    'unter',
    'ist',
    'sind',
    'war',
    'waren',
    'sein',
    'haben',
    'werden',
    'können',
    'müssen',
    'sollen',
    'wollen',
    'dieser',
    'jener',
    'mein',
    'dein',
    'sein',
    'unser',
    'euer',
    'ihr',
    'was',
    'wer',
    'wo',
    'wann',
    'wie',
    'warum',
    // Italian (Italiano)
    'il',
    'lo',
    'l',
    'i',
    'gli',
    'la',
    'le',
    'un',
    'uno',
    'una',
    'e',
    'o',
    'ma',
    'in',
    'di',
    'del',
    'dello',
    'della',
    'dei',
    'degli',
    'delle',
    'al',
    'allo',
    'alla',
    'ai',
    'agli',
    'alle',
    'per',
    'da',
    'dal',
    'dallo',
    'dalla',
    'dai',
    'dagli',
    'dalle',
    'con',
    'su',
    'sul',
    'sullo',
    'sulla',
    'sui',
    'sugli',
    'sulle',
    'è',
    'sono',
    'essere',
    'avere',
    'fare',
    'dire',
    'andare',
    'vedere',
    'sapere',
    'potere',
    'volere',
    'questo',
    'quello',
    'mio',
    'tuo',
    'suo',
    'nostro',
    'vostro',
    'loro',
    'che',
    'chi',
    'quale',
    'quando',
    'dove',
    'come',
    'perché',
    // Russian (Русский) - romanized
    'i',
    'v',
    'ne',
    'na',
    'ya',
    'on',
    's',
    'eto',
    'kak',
    'po',
    'no',
    'oni',
    'vse',
    'tak',
    'ego',
    'za',
    'byl',
    'bylo',
    'tem',
    'chto',
    'eto',
    'esli',
    'mogu',
    'mozhet',
    'by',
    // Chinese (中文) - common particles
    '的',
    '了',
    '和',
    '是',
    '在',
    '我',
    '有',
    '他',
    '这',
    '中',
    '大',
    '来',
    '上',
    '国',
    '个',
    '到',
    '说',
    '们',
    '为',
    '子',
    '中',
    '你',
    '地',
    '出',
    '道',
    '也',
    '时',
    '年',
    // Japanese (日本語) - particles and common words
    'は',
    'が',
    'を',
    'に',
    'で',
    'と',
    'の',
    'も',
    'や',
    'から',
    'まで',
    'より',
    'か',
    'な',
    'ね',
    'よ',
    'わ',
    'さ',
    'だ',
    'です',
    'ます',
    'ある',
    'いる',
    'する',
    'なる',
    'これ',
    'それ',
    'あれ',
    'この',
    'その',
    'あの',
    'ここ',
    'そこ',
    'あそこ',
    // Arabic (العربية) - romanized common words
    'al',
    'wa',
    'fi',
    'min',
    'ila',
    'an',
    'ma',
    'la',
    'li',
    'bi',
    'qad',
    'lam',
    'kan',
    'fi',
    'ala',
    'hatha',
    'dhalika',
    'huwa',
    'hiya',
    'hum',
    // Hindi (हिन्दी) - romanized common words
    'ka',
    'ki',
    'ke',
    'se',
    'ne',
    'ko',
    'me',
    'par',
    'hai',
    'tha',
    'the',
    'thi',
    'aur',
    'ya',
    'to',
    'is',
    'wo',
    'ye',
    'kya',
    'kaise',
    'kab',
    'kahan',
    'kyun',
  ]);

  private static readonly NEGATIONS = new Set([
    'not',
    'no',
    'never',
    "don't",
    "won't",
    "can't",
    "couldn't",
    "wouldn't",
    "shouldn't",
    "mustn't",
    "haven't",
    "hasn't",
    "hadn't",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
    'neither',
    'nor',
    'none',
  ]);

  private static readonly COMPARATORS = new Set([
    '!=',
    '!==',
    '<=',
    '>=',
    '<',
    '>',
    '==',
    '===',
    '!',
  ]);

  private static readonly MODALS = new Set([
    'only',
    'except',
    'must',
    'should',
    'may',
    'might',
    'at',
    'least',
    'most',
  ]);

  private config: StatisticalFilterConfig;

  // Regex patterns
  private codeBlockRe = /```[\s\S]*?```/g;
  private jsonRe = /\{[^}]*:[^}]*\}/g;
  private pathRe = /(?:[A-Za-z]+:)?\/\/[^\s]+|[/\\][\w/\\.-]+\.[A-Za-z0-9]{1,5}\b/g;
  private camelRe = /\b[A-Z][a-z0-9]+[A-Z][A-Za-z0-9]+\b/g;
  private snakeRe = /\b[a-z_][a-z0-9_]{2,}\b/g;
  private upperSnakeRe = /\b[A-Z][A-Z0-9_]+\b/g;
  private hashRe = /\b[0-9a-f]{7,}\b|\b\d{3,}\b/g;
  private bracketRe = /[{[(][^\]})*]*[\]})]/g;

  constructor(config: Partial<StatisticalFilterConfig> = {}) {
    this.config = { ...StatisticalFilter.DEFAULT_CONFIG, ...config };
  }

  /**
   * Create filter with default configuration
   */
  static default(): StatisticalFilter {
    return new StatisticalFilter(StatisticalFilter.DEFAULT_CONFIG);
  }

  /**
   * Filter text keeping only high-importance words
   *
   * @param text - Input text to compress
   * @returns Compressed text with low-importance words removed
   */
  compress(text: string): string {
    const importances = this.scoreWords(text);

    if (importances.length === 0) {
      return text;
    }

    // Sort by score (descending)
    const sorted = [...importances].sort((a, b) => b.score - a.score);

    // Keep top N% based on compression ratio
    const keepCount = Math.max(1, Math.floor(importances.length * this.config.compressionRatio));

    // Get indices of tokens to keep
    const keepIndices = new Set(sorted.slice(0, keepCount).map(imp => imp.position));

    // Fill gaps between critical tokens
    const criticalThreshold = 0.8;
    const criticalPositions = importances
      .filter(imp => imp.score > criticalThreshold && keepIndices.has(imp.position))
      .map(imp => imp.position)
      .sort((a, b) => a - b);

    for (let i = 0; i < criticalPositions.length - 1; i++) {
      const gapSize = criticalPositions[i + 1] - criticalPositions[i];
      if (gapSize > this.config.minGapBetweenCritical) {
        const gapCandidates = importances.filter(
          imp =>
            imp.position > criticalPositions[i] &&
            imp.position < criticalPositions[i + 1] &&
            !keepIndices.has(imp.position)
        );

        if (gapCandidates.length > 0) {
          const best = gapCandidates.reduce((prev, curr) =>
            curr.score > prev.score ? curr : prev
          );
          keepIndices.add(best.position);
        }
      }
    }

    // Reconstruct text with kept tokens
    const words = text.split(/\s+/);
    const kept = Array.from(keepIndices)
      .sort((a, b) => a - b)
      .map(idx => words[idx]);

    return kept.join(' ');
  }

  /**
   * Compress text and return detailed metrics
   *
   * @param text - Input text to compress
   * @returns Compression result with metrics
   */
  compressWithMetrics(text: string): {
    compressed: string;
    originalTokens: number;
    compressedTokens: number;
    compressionRatio: number;
    tokensRemoved: number;
  } {
    // Perform statistical compression
    const compressed = this.compress(text);

    // Calculate token counts (rough estimation: words)
    const originalTokens = text.split(/\s+/).filter(w => w.length > 0).length;
    const compressedTokens = compressed.split(/\s+/).filter(w => w.length > 0).length;
    const compressionRatio = originalTokens > 0 ? compressedTokens / originalTokens : 1.0;
    const tokensRemoved = originalTokens - compressedTokens;

    return {
      compressed,
      originalTokens,
      compressedTokens,
      compressionRatio,
      tokensRemoved,
    };
  }

  /**
   * Calculate importance scores for all tokens
   * Score words in text by importance
   *
   * @param text - Input text to score
   * @returns Array of word importance scores
   */
  scoreWords(text: string): WordImportance[] {
    const words = text.split(/\s+/);

    if (words.length === 0) {
      return [];
    }

    const protectedSpans = this.detectProtectedSpans(text);
    const wordPositions = this.buildWordPositions(text, words);

    const idfScores = this.calculateIdf(words);
    const positionScores = this.calculatePositionImportance(words);
    const posScores = this.calculatePosImportance(words);
    const entityScores = this.calculateEntityImportance(words);
    const entropyScores = this.calculateLocalEntropy(words);

    return words.map((word, idx) => {
      const criticalScore = this.isCriticalTerm(word);

      let finalScore: number;
      if (criticalScore !== null) {
        finalScore = criticalScore;
      } else {
        const [start, end] = wordPositions[idx];
        const isProtected = this.isWordProtected(start, end, protectedSpans);

        if (isProtected) {
          finalScore = Infinity;
        } else {
          const idf = idfScores.get(word) || 0;
          finalScore =
            idf * this.config.idfWeight +
            positionScores[idx] * this.config.positionWeight +
            posScores[idx] * this.config.posWeight +
            entityScores[idx] * this.config.entityWeight +
            entropyScores[idx] * this.config.entropyWeight;
        }
      }

      return { position: idx, text: word, score: finalScore };
    });
  }

  private detectProtectedSpans(text: string): ProtectedSpan[] {
    if (!this.config.enableProtectionMasks) {
      return [];
    }

    const spans: ProtectedSpan[] = [];

    // Code blocks
    for (const match of text.matchAll(this.codeBlockRe)) {
      if (match.index !== undefined) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.CODE_BLOCK,
        });
      }
    }

    // JSON blocks
    for (const match of text.matchAll(this.jsonRe)) {
      if (match.index !== undefined && match[0].includes(':')) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.JSON_BLOCK,
        });
      }
    }

    // Paths
    for (const match of text.matchAll(this.pathRe)) {
      if (match.index !== undefined) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.PATH,
        });
      }
    }

    // Identifiers
    for (const match of text.matchAll(this.camelRe)) {
      if (match.index !== undefined) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.IDENTIFIER,
        });
      }
    }

    for (const match of text.matchAll(this.snakeRe)) {
      if (match.index !== undefined && match[0].includes('_')) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.IDENTIFIER,
        });
      }
    }

    // UPPER_SNAKE_CASE identifiers
    for (const match of text.matchAll(this.upperSnakeRe)) {
      if (match.index !== undefined && match[0].length > 1) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.IDENTIFIER,
        });
      }
    }

    // Hashes and large numbers
    for (const match of text.matchAll(this.hashRe)) {
      if (match.index !== undefined) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.HASH_OR_NUMBER,
        });
      }
    }

    // Brackets, braces, parens content
    for (const match of text.matchAll(this.bracketRe)) {
      if (match.index !== undefined) {
        spans.push({
          start: match.index,
          end: match.index + match[0].length,
          spanType: SpanType.BRACKET,
        });
      }
    }

    return spans;
  }

  private buildWordPositions(text: string, words: string[]): Array<[number, number]> {
    const positions: Array<[number, number]> = [];
    let charIdx = 0;

    for (const word of words) {
      while (charIdx < text.length && /\s/.test(text[charIdx])) {
        charIdx++;
      }

      const start = charIdx;
      charIdx += word.length;
      const end = charIdx;

      positions.push([start, end]);
    }

    return positions;
  }

  private isWordProtected(start: number, end: number, spans: ProtectedSpan[]): boolean {
    return spans.some(span => start < span.end && end > span.start);
  }

  private isCriticalTerm(word: string): number | null {
    const lower = word.toLowerCase();

    for (const term of this.config.domainTerms) {
      if (word.toLowerCase() === term.toLowerCase()) {
        return Infinity;
      }
    }

    if (this.config.preserveNegations && StatisticalFilter.NEGATIONS.has(lower)) {
      return 10.0;
    }

    if (this.config.preserveComparators && StatisticalFilter.COMPARATORS.has(word)) {
      return 10.0;
    }

    if (StatisticalFilter.MODALS.has(lower)) {
      return 5.0;
    }

    return null;
  }

  private calculateIdf(words: string[]): Map<string, number> {
    const freqMap = new Map<string, number>();
    for (const word of words) {
      freqMap.set(word, (freqMap.get(word) || 0) + 1);
    }

    const total = words.length;
    const idfMap = new Map<string, number>();
    for (const [word, count] of freqMap.entries()) {
      idfMap.set(word, Math.log(total / count));
    }

    return idfMap;
  }

  private calculatePositionImportance(words: string[]): number[] {
    const len = words.length;
    return words.map((_, idx) => {
      const normalized = idx / len;
      if (normalized < 0.1 || normalized > 0.9) return 1.0;
      if (normalized < 0.2 || normalized > 0.8) return 0.7;
      return 0.3;
    });
  }

  private shouldPreserveStopword(
    word: string,
    contextBefore: string[],
    contextAfter: string[]
  ): boolean {
    if (!this.config.enableContextualStopwords) {
      return false;
    }

    const wordLower = word.toLowerCase();

    // "to" in infinitive/phrasal verbs: "how to", "steps to", "need to"
    if (wordLower === 'to') {
      if (contextBefore.length > 0) {
        const prev = contextBefore[contextBefore.length - 1].toLowerCase();
        if (['how', 'steps', 'need', 'want', 'try', 'used', 'able'].includes(prev)) {
          return true;
        }
      }
    }

    // "in/on/at" followed by paths or technical terms
    if (['in', 'on', 'at'].includes(wordLower)) {
      if (contextAfter.length > 0) {
        const next = contextAfter[0];
        // Check if next word looks like a path component
        if (next.includes('/') || next.includes('\\') || next.includes('.')) {
          return true;
        }
        // Check if next word is technical (starts with uppercase or contains _)
        if (next.length > 0 && (next[0] === next[0].toUpperCase() || next.includes('_'))) {
          return true;
        }
      }
    }

    // "is/are/was/were" in assertions (follows important term)
    if (['is', 'are', 'was', 'were', 'be'].includes(wordLower)) {
      if (contextBefore.length > 0) {
        const prev = contextBefore[contextBefore.length - 1];
        // If previous word is capitalized or technical, keep the verb
        if (
          (prev.length > 0 && prev[0] === prev[0].toUpperCase()) ||
          prev.length > 6 ||
          prev.includes('_')
        ) {
          return true;
        }
      }
    }

    // "and/or" between important terms
    if (['and', 'or'].includes(wordLower)) {
      const prevImportant =
        contextBefore.length > 0 &&
        ((contextBefore[contextBefore.length - 1].length > 0 &&
          contextBefore[contextBefore.length - 1][0] ===
            contextBefore[contextBefore.length - 1][0].toUpperCase()) ||
          contextBefore[contextBefore.length - 1].length > 6);
      const nextImportant =
        contextAfter.length > 0 &&
        ((contextAfter[0].length > 0 && contextAfter[0][0] === contextAfter[0][0].toUpperCase()) ||
          contextAfter[0].length > 6);
      if (prevImportant && nextImportant) {
        return true;
      }
    }

    return false;
  }

  private calculatePosImportance(words: string[]): number[] {
    return words.map((word, idx) => {
      const lower = word.toLowerCase();

      // Check if it's a stopword
      if (StatisticalFilter.STOP_WORDS.has(lower)) {
        // Check contextual preservation
        const contextBefore =
          idx > 0
            ? words
                .slice(Math.max(0, idx - 3), idx)
                .reverse()
                .reverse()
            : [];
        const contextAfter =
          idx + 1 < words.length ? words.slice(idx + 1, Math.min(words.length, idx + 4)) : [];

        if (this.shouldPreserveStopword(word, contextBefore, contextAfter)) {
          return 0.7; // Contextually important stopword
        } else {
          return 0.1; // Regular stopword - low importance
        }
      } else if (word.length > 0 && word[0] === word[0].toUpperCase()) {
        return 1.0; // Proper noun - high importance
      } else if (word.length > 6) {
        return 0.7; // Long word - medium-high importance
      } else {
        return 0.5; // Regular word - medium importance
      }
    });
  }

  private calculateEntityImportance(words: string[]): number[] {
    return words.map((word, idx) => {
      let score = 0;

      if (word.length > 0 && word[0] === word[0].toUpperCase()) {
        score += 0.3;
      }

      if (idx > 0) {
        const prev = words[idx - 1].toLowerCase();
        if (prev.startsWith('mr.') || prev.startsWith('dr.')) {
          score += 0.5;
        }
      }

      if (word.includes('@') || word.startsWith('http')) {
        score += 0.6;
      }

      if (word.length > 1 && word === word.toUpperCase()) {
        score += 0.4;
      }

      return Math.min(score, 1.0);
    });
  }

  private calculateLocalEntropy(words: string[]): number[] {
    const WINDOW = 10;
    return words.map((_, idx) => {
      const start = Math.max(0, idx - Math.floor(WINDOW / 2));
      const end = Math.min(words.length, idx + Math.floor(WINDOW / 2));
      const window = words.slice(start, end);
      const unique = new Set(window).size;
      return unique / window.length;
    });
  }
}
