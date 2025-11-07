"""Quality metrics for compression evaluation (model-free)

Measures how well compressed text preserves important information
"""

from dataclasses import dataclass
from typing import List, Set


@dataclass
class QualityMetrics:
    """Quality assessment for compressed text."""
    
    keyword_retention: float
    entity_retention: float
    vocabulary_ratio: float
    information_density: float
    overall_score: float
    
    @staticmethod
    def calculate(original: str, compressed: str) -> 'QualityMetrics':
        """Calculate comprehensive quality metrics."""
        orig_words = QualityMetrics._tokenize(original)
        comp_words = QualityMetrics._tokenize(compressed)
        
        # Extract important elements
        orig_keywords = QualityMetrics._extract_keywords(orig_words)
        comp_keywords = QualityMetrics._extract_keywords(comp_words)
        
        orig_entities = QualityMetrics._extract_entities(orig_words)
        comp_entities = QualityMetrics._extract_entities(comp_words)
        
        # Calculate retention rates
        keyword_retention = QualityMetrics._calculate_retention(orig_keywords, comp_keywords)
        entity_retention = QualityMetrics._calculate_retention(orig_entities, comp_entities)
        
        # Vocabulary analysis
        orig_vocab = {word.lower() for word in orig_words}
        comp_vocab = {word.lower() for word in comp_words}
        vocabulary_ratio = len(comp_vocab) / max(1, len(orig_vocab))
        
        # Information density
        information_density = (len(comp_vocab) / len(comp_words)) if comp_words else 0.0
        
        # Overall score (weighted average)
        overall_score = (
            keyword_retention * 0.4 +
            entity_retention * 0.3 +
            vocabulary_ratio * 0.2 +
            information_density * 0.1
        )
        
        return QualityMetrics(
            keyword_retention=keyword_retention,
            entity_retention=entity_retention,
            vocabulary_ratio=vocabulary_ratio,
            information_density=information_density,
            overall_score=overall_score
        )
    
    @staticmethod
    def _tokenize(text: str) -> List[str]:
        """Tokenize text into words."""
        return text.split()
    
    @staticmethod
    def _extract_keywords(words: List[str]) -> Set[str]:
        """Extract important keywords (long words, capitalized, technical terms)."""
        STOP_WORDS = frozenset([
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
            "can", "this", "that", "these", "those", "we", "they", "it"
        ])
        
        keywords = set()
        for word in words:
            lower = word.lower()
            # Keep if: not a stop word AND (long OR capitalized OR contains special chars)
            if (lower not in STOP_WORDS and
                (len(word) > 5 or (word and word[0].isupper()) or '-' in word or '_' in word)):
                keywords.add(lower)
        
        return keywords
    
    @staticmethod
    def _extract_entities(words: List[str]) -> Set[str]:
        """Extract named entities (capitalized sequences, emails, URLs, acronyms)."""
        entities = set()
        
        for idx, word in enumerate(words):
            # Emails and URLs
            if '@' in word or word.startswith("http"):
                entities.add(word.lower())
            
            # Acronyms (2+ uppercase letters)
            if len(word) > 1 and all(c.isupper() or not c.isalpha() for c in word):
                entities.add(word)
            
            # Capitalized words (potential proper nouns)
            if word and word[0].isupper() and len(word) > 2:
                # Multi-word entities
                if idx + 1 < len(words) and words[idx + 1] and words[idx + 1][0].isupper():
                    entity = f"{word} {words[idx + 1]}"
                    entities.add(entity)
                entities.add(word)
        
        return entities
    
    @staticmethod
    def _calculate_retention(original: Set[str], compressed: Set[str]) -> float:
        """Calculate retention rate between two sets."""
        if not original:
            return 1.0
        
        preserved = len(original & compressed)
        return preserved / len(original)
    
    def format(self) -> str:
        """Format metrics as human-readable string."""
        return f"""Quality Metrics:
- Keyword Retention: {self.keyword_retention * 100:.1f}%
- Entity Retention: {self.entity_retention * 100:.1f}%
- Vocabulary Ratio: {self.vocabulary_ratio * 100:.1f}%
- Info Density: {self.information_density:.3f}
- Overall Score: {self.overall_score * 100:.1f}%"""

