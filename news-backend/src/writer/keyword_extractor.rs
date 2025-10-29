// Keyword Extractor
// Extracts relevant keywords from article title and content for image search

use std::collections::HashSet;
use regex::Regex;

/// Extrai keywords relevantes do artigo para busca de imagens
/// 
/// Estratégia:
/// 1. Extrai palavras do título (noun phrases)
/// 2. Extrai termos técnicos importantes do conteúdo
/// 3. Remove stop words comuns
/// 4. Limita a 3-5 keywords mais relevantes
pub fn extract_keywords(title: &str, article_text: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    
    // 1. Extrair do título (prioridade alta)
    let title_keywords = extract_from_title(title);
    keywords.extend(title_keywords);
    
    // 2. Extrair termos técnicos chave do conteúdo
    let content_keywords = extract_from_content(article_text);
    keywords.extend(content_keywords);
    
    // 3. Remover duplicatas e normalizar
    let mut unique_keywords: HashSet<String> = HashSet::new();
    for keyword in keywords {
        let normalized = keyword.to_lowercase().trim().to_string();
        if normalized.len() > 3 && !is_stop_word(&normalized) {
            unique_keywords.insert(normalized);
        }
    }
    
    // 4. Converter para Vec e limitar
    let mut result: Vec<String> = unique_keywords.into_iter().collect();
    result.sort();
    result.truncate(5); // Max 5 keywords
    
    result
}

/// Extrai keywords do título
fn extract_from_title(title: &str) -> Vec<String> {
    let re = Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap();
    
    re.find_iter(title)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Extrai termos técnicos importantes do conteúdo
fn extract_from_content(text: &str) -> Vec<String> {
    // Procurar por termos chave que aparecem frequentemente
    let re = Regex::new(r"\b[a-zA-Z]{4,}\b").unwrap();
    
    let mut word_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for cap in re.find_iter(text) {
        let word = cap.as_str().to_lowercase();
        if !is_stop_word(&word) && word.len() > 4 {
            *word_count.entry(word).or_insert(0) += 1;
        }
    }
    
    // Retornar as 10 palavras mais frequentes
    let mut sorted_words: Vec<(String, usize)> = word_count.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
    sorted_words.into_iter().take(10).map(|(word, _)| word).collect()
}

/// Verifica se é uma stop word (palavras muito comuns sem significado)
fn is_stop_word(word: &str) -> bool {
    let stop_words = vec![
        "this", "that", "these", "those", "the", "a", "an", "is", "are", "was", "were",
        "be", "been", "being", "have", "has", "had", "do", "does", "did", "will", "would",
        "should", "could", "may", "might", "must", "can", "from", "for", "with", "about",
        "into", "onto", "over", "under", "above", "below", "through", "during", "while",
        "which", "what", "when", "where", "why", "how", "how", "and", "or", "but", "nor",
        "so", "yet", "very", "more", "most", "much", "many", "some", "any", "all", "each",
        "every", "both", "few", "little", "other", "another", "such", "same", "only", "also",
        "after", "before", "since", "until", "up", "down", "out", "off", "on", "in", "at",
        "to", "of", "by", "as", "if", "else", "than", "then", "there", "their", "they",
        "them", "its", "it", "its", "our", "we", "us", "your", "you", "his", "him", "her",
        "him", "she", "he", "me", "my", "mine", "who", "whom", "whose", "here", "there",
    ];
    
    stop_words.contains(&word.to_lowercase().as_str())
}

/// Combina keywords em uma string para busca no Pixabay
pub fn keywords_to_search_query(keywords: &[String]) -> String {
    // Pegar as 3 primeiras keywords e combinar com "+"
    let top_keywords: Vec<String> = keywords.iter().take(3).cloned().collect();
    top_keywords.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_keywords() {
        let title = "Artificial Intelligence Breakthrough: New Neural Networks";
        let content = "Machine learning algorithms have revolutionized the field of artificial intelligence. Deep neural networks are now capable of understanding complex patterns in data.";
        
        let keywords = extract_keywords(title, content);
        println!("Extracted keywords: {:?}", keywords);
        assert!(!keywords.is_empty());
    }
}

