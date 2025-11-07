"""Statistical token importance filtering (LLMLingua-inspired, model-free)

This module implements a compression strategy similar to LLMLingua but using
pure statistical heuristics instead of model-based perplexity scoring.

Enhanced with token-aware semantic preservation:
- Protects code blocks, JSON, paths, identifiers
- Contextual stopword filtering
- Preserves negations, comparators, domain terms
"""

import re
from dataclasses import dataclass, field
from enum import Enum
from typing import List, Dict, Set, Tuple, Optional
from collections import defaultdict
import math


class SpanType(Enum):
    """Type of protected span that should not be modified."""
    CODE_BLOCK = "code_block"
    JSON_BLOCK = "json_block"
    PATH = "path"
    IDENTIFIER = "identifier"
    HASH_OR_NUMBER = "hash_or_number"
    BRACKET = "bracket"


@dataclass
class ProtectedSpan:
    """A span of text that should be protected from modification."""
    start: int
    end: int
    span_type: SpanType


@dataclass
class WordImportance:
    """Importance score for a word based on statistical features."""
    position: int
    text: str
    score: float


@dataclass
class StatisticalFilterConfig:
    """Configuration for statistical filtering."""
    
    # Target compression ratio (0.0 to 1.0)
    compression_ratio: float = 0.5
    
    # Feature weights
    idf_weight: float = 0.3
    position_weight: float = 0.2
    pos_weight: float = 0.2
    entity_weight: float = 0.2
    entropy_weight: float = 0.1
    
    # Token-aware semantic preservation options
    enable_protection_masks: bool = True
    enable_contextual_stopwords: bool = True
    preserve_negations: bool = True
    preserve_comparators: bool = True
    
    # Domain-specific terms to always preserve
    domain_terms: List[str] = field(default_factory=lambda: [
        "Vectorizer", "Synap", "UMICP", "Graphs"
    ])
    
    # Minimum gap between critical tokens
    min_gap_between_critical: int = 3


class StatisticalFilter:
    """Statistical token filter (model-free alternative to LLMLingua)."""
    
    # Multilingual stopwords (10+ languages)
    STOP_WORDS = frozenset([
        # English
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
        "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has",
        "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
        "can", "shall", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we",
        "they", "what", "which", "who", "when", "where", "why", "how",
        # Spanish (Español)
        "el", "la", "los", "las", "un", "una", "unos", "unas", "y", "o", "pero", "en", "de",
        "del", "al", "para", "por", "con", "sin", "sobre", "entre", "hasta", "desde", "es",
        "son", "está", "están", "ser", "estar", "haber", "hacer", "tener", "decir", "ir",
        "ver", "dar", "saber", "querer", "poder", "poner", "este", "ese", "aquel", "mi",
        "tu", "su", "nuestro", "vuestro", "que", "quien", "cual", "cuando", "donde", "como",
        # Portuguese (Português)
        "o", "a", "os", "as", "um", "uma", "uns", "umas", "e", "ou", "mas", "em", "de",
        "do", "da", "dos", "das", "no", "na", "nos", "nas", "ao", "à", "aos", "às", "para",
        "por", "com", "sem", "sobre", "entre", "até", "desde", "é", "são", "está", "estão",
        "ser", "estar", "haver", "ter", "fazer", "dizer", "ir", "ver", "dar", "saber",
        "querer", "poder", "pôr", "este", "esse", "aquele", "meu", "teu", "seu", "nosso",
        "vosso", "que", "quem", "qual", "quando", "onde", "como",
        # French (Français)
        "le", "la", "les", "un", "une", "des", "et", "ou", "mais", "dans", "en", "de",
        "du", "au", "aux", "pour", "par", "avec", "sans", "sur", "sous", "entre", "vers",
        "chez", "est", "sont", "être", "avoir", "faire", "dire", "aller", "voir", "savoir",
        "pouvoir", "vouloir", "venir", "devoir", "prendre", "ce", "cet", "cette", "ces",
        "mon", "ton", "son", "notre", "votre", "leur", "que", "qui", "quoi", "dont", "où",
        "quand", "comment",
        # German (Deutsch)
        "der", "die", "das", "den", "dem", "des", "ein", "eine", "einer", "eines", "einem",
        "einen", "und", "oder", "aber", "in", "im", "an", "auf", "für", "von", "zu", "mit",
        "bei", "nach", "über", "unter", "ist", "sind", "war", "waren", "sein", "haben",
        "werden", "können", "müssen", "sollen", "wollen", "dieser", "jener", "mein", "dein",
        "sein", "unser", "euer", "ihr", "was", "wer", "wo", "wann", "wie", "warum",
        # Italian (Italiano)
        "il", "lo", "l", "i", "gli", "la", "le", "un", "uno", "una", "e", "o", "ma", "in",
        "di", "del", "dello", "della", "dei", "degli", "delle", "al", "allo", "alla", "ai",
        "agli", "alle", "per", "da", "dal", "dallo", "dalla", "dai", "dagli", "dalle", "con",
        "su", "sul", "sullo", "sulla", "sui", "sugli", "sulle", "è", "sono", "essere", "avere",
        "fare", "dire", "andare", "vedere", "sapere", "potere", "volere", "questo", "quello",
        "mio", "tuo", "suo", "nostro", "vostro", "loro", "che", "chi", "quale", "quando",
        "dove", "come", "perché",
        # Russian (Русский) - romanized
        "i", "v", "ne", "na", "ya", "on", "s", "eto", "kak", "po", "no", "oni", "vse",
        "tak", "ego", "za", "byl", "bylo", "tem", "chto", "eto", "esli", "mogu", "mozhet", "by",
        # Chinese (中文) - common particles
        "的", "了", "和", "是", "在", "我", "有", "他", "这", "中", "大", "来", "上",
        "国", "个", "到", "说", "们", "为", "子", "中", "你", "地", "出", "道", "也",
        "时", "年",
        # Japanese (日本語) - particles and common words
        "は", "が", "を", "に", "で", "と", "の", "も", "や", "から", "まで", "より",
        "か", "な", "ね", "よ", "わ", "さ", "だ", "です", "ます", "ある", "いる", "する",
        "なる", "これ", "それ", "あれ", "この", "その", "あの", "ここ", "そこ", "あそこ",
        # Arabic (العربية) - romanized common words
        "al", "wa", "fi", "min", "ila", "an", "ma", "la", "li", "bi", "qad", "lam",
        "kan", "fi", "ala", "hatha", "dhalika", "huwa", "hiya", "hum",
        # Hindi (हिन्दी) - romanized common words
        "ka", "ki", "ke", "se", "ne", "ko", "me", "par", "hai", "tha", "the", "thi",
        "aur", "ya", "to", "is", "wo", "ye", "kya", "kaise", "kab", "kahan", "kyun",
    ])
    
    NEGATIONS = frozenset([
        "not", "no", "never", "don't", "won't", "can't", "couldn't", "wouldn't",
        "shouldn't", "mustn't", "haven't", "hasn't", "hadn't", "isn't", "aren't",
        "wasn't", "weren't", "neither", "nor", "none"
    ])
    
    COMPARATORS = frozenset(["!=", "!==", "<=", ">=", "<", ">", "==", "===", "!"])
    
    MODALS = frozenset([
        "only", "except", "must", "should", "may", "might", "at", "least", "most"
    ])
    
    def __init__(self, config: Optional[StatisticalFilterConfig] = None):
        """Create a new statistical filter."""
        self.config = config or StatisticalFilterConfig()
        
        # Compile regex patterns once
        self.code_block_re = re.compile(r"```[\s\S]*?```")
        self.json_re = re.compile(r"\{[^}]*:[^}]*\}")
        self.path_re = re.compile(r"(?:[A-Za-z]+:)?//[^\s]+|[/\\][\w/\\.-]+\.[A-Za-z0-9]{1,5}\b")
        self.camel_re = re.compile(r"\b[A-Z][a-z0-9]+[A-Z][A-Za-z0-9]+\b")
        self.snake_re = re.compile(r"\b[a-z_][a-z0-9_]{2,}\b")
        self.upper_snake_re = re.compile(r"\b[A-Z][A-Z0-9_]+\b")
        self.hash_re = re.compile(r"\b[0-9a-f]{7,}\b|\b\d{3,}\b")
        self.bracket_re = re.compile(r"[\{\[\(][^\}\]\)]*[\}\]\)]")
    
    def detect_protected_spans(self, text: str) -> List[ProtectedSpan]:
        """Detect protected spans in text that should not be modified."""
        if not self.config.enable_protection_masks:
            return []
        
        spans = []
        
        # Code blocks
        for match in self.code_block_re.finditer(text):
            spans.append(ProtectedSpan(
                start=match.start(),
                end=match.end(),
                span_type=SpanType.CODE_BLOCK
            ))
        
        # JSON blocks
        for match in self.json_re.finditer(text):
            content = text[match.start():match.end()]
            if ':' in content:
                spans.append(ProtectedSpan(
                    start=match.start(),
                    end=match.end(),
                    span_type=SpanType.JSON_BLOCK
                ))
        
        # Paths and URLs
        for match in self.path_re.finditer(text):
            spans.append(ProtectedSpan(
                start=match.start(),
                end=match.end(),
                span_type=SpanType.PATH
            ))
        
        # CamelCase identifiers
        for match in self.camel_re.finditer(text):
            spans.append(ProtectedSpan(
                start=match.start(),
                end=match.end(),
                span_type=SpanType.IDENTIFIER
            ))
        
        # snake_case identifiers
        for match in self.snake_re.finditer(text):
            if '_' in match.group():
                spans.append(ProtectedSpan(
                    start=match.start(),
                    end=match.end(),
                    span_type=SpanType.IDENTIFIER
                ))
        
        # UPPER_SNAKE_CASE identifiers
        for match in self.upper_snake_re.finditer(text):
            if len(match.group()) > 1:
                spans.append(ProtectedSpan(
                    start=match.start(),
                    end=match.end(),
                    span_type=SpanType.IDENTIFIER
                ))
        
        # Hashes and large numbers
        for match in self.hash_re.finditer(text):
            spans.append(ProtectedSpan(
                start=match.start(),
                end=match.end(),
                span_type=SpanType.HASH_OR_NUMBER
            ))
        
        # Brackets content
        for match in self.bracket_re.finditer(text):
            spans.append(ProtectedSpan(
                start=match.start(),
                end=match.end(),
                span_type=SpanType.BRACKET
            ))
        
        return spans
    
    def is_word_protected(self, word_start: int, word_end: int, 
                         protected: List[ProtectedSpan]) -> bool:
        """Check if a word/token position overlaps with any protected span."""
        for span in protected:
            if word_start < span.end and word_end > span.start:
                return True
        return False
    
    def should_preserve_stopword(self, word: str, context_before: List[str], 
                                context_after: List[str]) -> bool:
        """Check if a stopword should be preserved based on context."""
        if not self.config.enable_contextual_stopwords:
            return False
        
        word_lower = word.lower()
        
        # "to" in infinitive/phrasal verbs
        if word_lower == "to":
            if context_before and context_before[-1].lower() in [
                "how", "steps", "need", "want", "try", "used", "able"
            ]:
                return True
        
        # "in/on/at" followed by paths or technical terms
        if word_lower in ["in", "on", "at"]:
            if context_after:
                next_word = context_after[0]
                if '/' in next_word or '\\' in next_word or '.' in next_word:
                    return True
                if next_word and (next_word[0].isupper() or '_' in next_word):
                    return True
        
        # "is/are/was/were" in assertions
        if word_lower in ["is", "are", "was", "were", "be"]:
            if context_before:
                prev = context_before[-1]
                if prev and (prev[0].isupper() or len(prev) > 6 or '_' in prev):
                    return True
        
        # "and/or" between important terms
        if word_lower in ["and", "or"]:
            prev_important = (context_before and 
                            (context_before[-1][0].isupper() or len(context_before[-1]) > 6))
            next_important = (context_after and 
                            (context_after[0][0].isupper() or len(context_after[0]) > 6))
            if prev_important and next_important:
                return True
        
        return False
    
    def is_critical_term(self, word: str) -> Optional[float]:
        """Check if a word is a critical term that must be preserved."""
        word_lower = word.lower()
        
        # Domain-specific terms
        for domain_term in self.config.domain_terms:
            if word.lower() == domain_term.lower():
                return float('inf')
        
        # Negations
        if self.config.preserve_negations and word_lower in self.NEGATIONS:
            return 10.0
        
        # Comparators
        if self.config.preserve_comparators and word in self.COMPARATORS:
            return 10.0
        
        # Modals
        if word_lower in self.MODALS:
            return 5.0
        
        return None
    
    def score_words(self, text: str) -> List[WordImportance]:
        """Calculate importance scores for all tokens."""
        words = text.split()
        
        if not words:
            return []
        
        # Detect protected spans
        protected_spans = self.detect_protected_spans(text)
        
        # Build word positions mapping
        word_positions = []
        char_idx = 0
        for word in words:
            # Skip whitespace
            while char_idx < len(text) and text[char_idx].isspace():
                char_idx += 1
            
            start = char_idx
            char_idx += len(word)
            end = char_idx
            
            word_positions.append((start, end))
        
        # Calculate various statistical features
        idf_scores = self._calculate_idf(words)
        position_scores = self._calculate_position_importance(words)
        pos_scores = self._calculate_pos_importance(words)
        entity_scores = self._calculate_entity_importance(words)
        entropy_scores = self._calculate_local_entropy(words)
        
        # Combine scores for each word
        importances = []
        for idx, word in enumerate(words):
            # Check if word is critical or protected
            critical_score = self.is_critical_term(word)
            
            if critical_score is not None:
                final_score = critical_score
            else:
                # Check if word is in a protected span
                start, end = word_positions[idx]
                is_protected = self.is_word_protected(start, end, protected_spans)
                
                if is_protected:
                    final_score = float('inf')
                else:
                    # Calculate normal combined score
                    idf = idf_scores.get(word, 0.0)
                    pos_score = position_scores[idx]
                    pos_tag_score = pos_scores[idx]
                    entity_score = entity_scores[idx]
                    entropy = entropy_scores[idx]
                    
                    final_score = (
                        idf * self.config.idf_weight +
                        pos_score * self.config.position_weight +
                        pos_tag_score * self.config.pos_weight +
                        entity_score * self.config.entity_weight +
                        entropy * self.config.entropy_weight
                    )
            
            importances.append(WordImportance(
                position=idx,
                text=word,
                score=final_score
            ))
        
        return importances
    
    def compress(self, text: str) -> str:
        """Filter text keeping only high-importance words."""
        importances = self.score_words(text)
        
        if not importances:
            return text
        
        # Sort by score (descending)
        sorted_importances = sorted(importances, key=lambda x: x.score, reverse=True)
        
        # Keep top N% based on compression ratio
        keep_count = max(1, int(len(importances) * self.config.compression_ratio))
        keep_count = min(keep_count, len(importances))
        
        # Get indices of tokens to keep
        keep_indices = {imp.position for imp in sorted_importances[:keep_count]}
        
        # Fill gaps between critical tokens
        critical_threshold = 0.8
        critical_positions = sorted([
            imp.position for imp in importances
            if imp.score > critical_threshold and imp.position in keep_indices
        ])
        
        # Check for large gaps between critical tokens
        for i in range(len(critical_positions) - 1):
            gap_size = critical_positions[i + 1] - critical_positions[i]
            if gap_size > self.config.min_gap_between_critical:
                # Find highest-scored token in the gap
                gap_candidates = [
                    imp for imp in importances
                    if (critical_positions[i] < imp.position < critical_positions[i + 1] and
                        imp.position not in keep_indices)
                ]
                
                if gap_candidates:
                    best = max(gap_candidates, key=lambda x: x.score)
                    keep_indices.add(best.position)
        
        # Reconstruct text with kept tokens in original order
        words = text.split()
        kept_words = [words[idx] for idx in sorted(keep_indices)]
        
        return ' '.join(kept_words)
    
    def _calculate_idf(self, words: List[str]) -> Dict[str, float]:
        """Calculate IDF scores."""
        freq_map = defaultdict(int)
        for word in words:
            freq_map[word] += 1
        
        total = len(words)
        return {word: math.log(total / count) for word, count in freq_map.items()}
    
    def _calculate_position_importance(self, words: List[str]) -> List[float]:
        """Calculate position importance (U-shaped: start and end are important)."""
        length = len(words)
        scores = []
        
        for idx in range(length):
            normalized = idx / length if length > 0 else 0
            if normalized < 0.1 or normalized > 0.9:
                score = 1.0
            elif normalized < 0.2 or normalized > 0.8:
                score = 0.7
            else:
                score = 0.3
            scores.append(score)
        
        return scores
    
    def _calculate_pos_importance(self, words: List[str]) -> List[float]:
        """Calculate POS importance using stop word heuristics."""
        scores = []
        
        for idx, word in enumerate(words):
            word_lower = word.lower()
            
            # Check if it's a stopword
            if word_lower in self.STOP_WORDS:
                # Check contextual preservation
                context_before = words[max(0, idx-3):idx]
                context_after = words[idx+1:min(len(words), idx+4)]
                
                if self.should_preserve_stopword(word, context_before, context_after):
                    score = 0.7  # Contextually important stopword
                else:
                    score = 0.1  # Regular stopword
            elif word and word[0].isupper():
                score = 1.0  # Proper noun
            elif len(word) > 6:
                score = 0.7  # Long word
            else:
                score = 0.5  # Regular word
            
            scores.append(score)
        
        return scores
    
    def _calculate_entity_importance(self, words: List[str]) -> List[float]:
        """Detect named entities using simple patterns."""
        scores = []
        
        for idx, word in enumerate(words):
            score = 0.0
            
            if word and word[0].isupper():
                score += 0.3
            
            if idx > 0:
                prev = words[idx - 1].lower()
                if prev.startswith("mr.") or prev.startswith("dr."):
                    score += 0.5
            
            if '@' in word or word.startswith("http"):
                score += 0.6
            
            if len(word) > 1 and word.isupper():
                score += 0.4
            
            scores.append(min(score, 1.0))
        
        return scores
    
    def _calculate_local_entropy(self, words: List[str]) -> List[float]:
        """Calculate local entropy (vocabulary diversity)."""
        WINDOW = 10
        scores = []
        
        for idx in range(len(words)):
            start = max(0, idx - WINDOW // 2)
            end = min(len(words), idx + WINDOW // 2)
            window = words[start:end]
            
            unique = len(set(window))
            total = len(window)
            
            scores.append(unique / total if total > 0 else 0.0)
        
        return scores

