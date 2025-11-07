//! Quality metrics for compression evaluation (model-free)
//!
//! Measures how well compressed text preserves important information

use std::collections::HashSet;

/// Quality assessment for compressed text
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Percentage of important keywords preserved (0.0-1.0)
    pub keyword_retention: f64,

    /// Percentage of named entities preserved (0.0-1.0)
    pub entity_retention: f64,

    /// Vocabulary diversity ratio (compressed/original)
    pub vocabulary_ratio: f64,

    /// Information density (unique_words/total_words)
    pub information_density: f64,

    /// Overall quality score (0.0-1.0)
    pub overall_score: f64,
}

impl QualityMetrics {
    /// Calculate comprehensive quality metrics
    pub fn calculate(original: &str, compressed: &str) -> Self {
        let orig_words = Self::tokenize(original);
        let comp_words = Self::tokenize(compressed);

        // Extract important elements
        let orig_keywords = Self::extract_keywords(&orig_words);
        let comp_keywords = Self::extract_keywords(&comp_words);

        let orig_entities = Self::extract_entities(&orig_words);
        let comp_entities = Self::extract_entities(&comp_words);

        // Calculate retention rates
        let keyword_retention = Self::calculate_retention(&orig_keywords, &comp_keywords);
        let entity_retention = Self::calculate_retention(&orig_entities, &comp_entities);

        // Vocabulary analysis
        let orig_vocab: HashSet<_> = orig_words.iter().map(|s| s.to_lowercase()).collect();
        let comp_vocab: HashSet<_> = comp_words.iter().map(|s| s.to_lowercase()).collect();
        let vocabulary_ratio = comp_vocab.len() as f64 / orig_vocab.len().max(1) as f64;

        // Information density
        let information_density = if comp_words.is_empty() {
            0.0
        } else {
            comp_vocab.len() as f64 / comp_words.len() as f64
        };

        // Overall score (weighted average)
        let overall_score = keyword_retention * 0.4
            + entity_retention * 0.3
            + vocabulary_ratio * 0.2
            + information_density * 0.1;

        Self {
            keyword_retention,
            entity_retention,
            vocabulary_ratio,
            information_density,
            overall_score,
        }
    }

    /// Tokenize text into words
    fn tokenize(text: &str) -> Vec<&str> {
        text.split_whitespace().collect()
    }

    /// Extract important keywords (long words, capitalized, technical terms)
    fn extract_keywords(words: &[&str]) -> HashSet<String> {
        const STOP_WORDS: &[&str] = &[
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
            "can", "this", "that", "these", "those", "we", "they", "it",
        ];

        words
            .iter()
            .filter_map(|word| {
                let lower = word.to_lowercase();
                // Keep if: not a stop word AND (long OR capitalized OR contains special chars)
                if !STOP_WORDS.contains(&lower.as_str())
                    && (word.len() > 5
                        || word.chars().next().is_some_and(|c| c.is_uppercase())
                        || word.contains('-')
                        || word.contains('_'))
                {
                    Some(lower)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract named entities (capitalized sequences, emails, URLs, acronyms)
    fn extract_entities(words: &[&str]) -> HashSet<String> {
        let mut entities = HashSet::new();

        for (i, word) in words.iter().enumerate() {
            // Emails and URLs
            if word.contains('@') || word.starts_with("http") {
                entities.insert(word.to_lowercase());
            }

            // Acronyms (2+ uppercase letters)
            if word.len() > 1 && word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
                entities.insert(word.to_string());
            }

            // Capitalized words (potential proper nouns)
            if word.chars().next().is_some_and(|c| c.is_uppercase()) && word.len() > 2 {
                // Multi-word entities (e.g., "John Smith")
                if i + 1 < words.len()
                    && words[i + 1]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_uppercase())
                {
                    let entity = format!("{} {}", word, words[i + 1]);
                    entities.insert(entity);
                }
                entities.insert(word.to_string());
            }
        }

        entities
    }

    /// Calculate retention rate between two sets
    fn calculate_retention(original: &HashSet<String>, compressed: &HashSet<String>) -> f64 {
        if original.is_empty() {
            return 1.0;
        }

        let preserved = original.intersection(compressed).count();
        preserved as f64 / original.len() as f64
    }

    /// Format metrics as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Quality Metrics:\n\
             - Keyword Retention: {:.1}%\n\
             - Entity Retention: {:.1}%\n\
             - Vocabulary Ratio: {:.1}%\n\
             - Info Density: {:.3}\n\
             - Overall Score: {:.1}%",
            self.keyword_retention * 100.0,
            self.entity_retention * 100.0,
            self.vocabulary_ratio * 100.0,
            self.information_density,
            self.overall_score * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_preservation() {
        let text = "Machine Learning is a subset of Artificial Intelligence";
        let metrics = QualityMetrics::calculate(text, text);

        assert_eq!(metrics.keyword_retention, 1.0);
        assert_eq!(metrics.entity_retention, 1.0);
        assert_eq!(metrics.vocabulary_ratio, 1.0);
    }

    #[test]
    fn test_lossy_compression() {
        let original = "Machine Learning is a powerful subset of Artificial Intelligence";
        let compressed = "Machine Learning subset Artificial Intelligence";
        let metrics = QualityMetrics::calculate(original, compressed);

        // Should retain important keywords
        assert!(metrics.keyword_retention > 0.7);
        assert!(metrics.entity_retention > 0.7);
        assert!(metrics.overall_score > 0.5);
    }

    #[test]
    fn test_entity_extraction() {
        let text = "Dr. John Smith works at IBM and uses john@example.com";
        let words: Vec<&str> = text.split_whitespace().collect();
        let entities = QualityMetrics::extract_entities(&words);

        assert!(entities.contains("IBM"));
        assert!(entities.contains("john@example.com"));
    }
}
