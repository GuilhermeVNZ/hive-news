use crate::filter::parser::ParsedPdf;

const CATEGORIES: &[(&str, &[&str])] = &[
    ("machine-learning", &["machine learning", "neural network", "deep learning"]),
    ("nlp", &["natural language", "nlp", "text processing", "language model"]),
    ("computer-vision", &["computer vision", "image processing", "object detection"]),
    ("robotics", &["robot", "robotics", "autonomous"]),
    ("theory", &["theoretical", "complexity", "algorithm analysis"]),
    ("security", &["security", "cryptography", "privacy"]),
];

pub fn categorize(parsed: &ParsedPdf) -> String {
    let text_lower = format!("{} {}", parsed.title, parsed.text).to_lowercase();
    
    for (category, keywords) in CATEGORIES {
        let matches = keywords.iter()
            .filter(|&&kw| text_lower.contains(kw))
            .count();
        
        if matches >= 2 {
            return category.to_string();
        }
    }
    
    "general".to_string()
}


