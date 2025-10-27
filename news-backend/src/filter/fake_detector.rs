const FAKE_PATTERNS: &[&str] = &[
    "in this study, we propose",
    "the rest of this paper is organized as follows",
    "to the best of our knowledge",
    "experimental results show that",
];

pub fn calculate_fake_penalty(text: &str) -> f32 {
    let mut penalty = 0.0;
    
    // Padrões genéricos
    for pattern in FAKE_PATTERNS {
        if text.to_lowercase().contains(pattern) {
            penalty += 0.15;
        }
    }
    
    // Repetições via n-grams (simplificado)
    let ngrams = extract_trigrams(text);
    let repetition_rate = calculate_repetition(&ngrams);
    penalty += repetition_rate * 0.3;
    
    penalty.min(1.0)
}

fn extract_trigrams(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    words.windows(3)
        .map(|w| w.join(" "))
        .collect()
}

fn calculate_repetition(ngrams: &[String]) -> f32 {
    let mut counts = std::collections::HashMap::new();
    for ng in ngrams {
        *counts.entry(ng.clone()).or_insert(0) += 1;
    }
    
    let max_count = counts.values().max().unwrap_or(&1);
    (*max_count as f32 - 1.0) / ngrams.len().max(1) as f32
}


