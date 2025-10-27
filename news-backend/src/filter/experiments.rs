use crate::filter::parser::ParsedPdf;

// Keywords para detectar testes e resultados empíricos
const TEST_KEYWORDS: &[&str] = &[
    "test", "tested", "testing", "tests", "test suite",
    "accuracy", "precision", "recall", "f1", "f1-score",
    "auc", "roc", "confusion matrix", 
    "evaluation", "evaluate", "evaluated",
    "results show", "experimental results", "our results",
    "benchmark", "baseline", "comparison",
    "error", "errors", "improvement", "performance",
    "table", "fig", "figure", "plot", "chart"
];

// Keywords para resultados numéricos (indicador de testes reais)
const NUMERIC_RESULT_PATTERNS: &[&str] = &[
    "accuracy", "%", 
    "precision", "recall",
    "f1", "auc", "roc",
    "mse", "rmse", "mae",
    "bleu", "rouge", "meteor"
];

pub fn has_tests_in_results(parsed: &ParsedPdf) -> bool {
    let text_lower = parsed.text.to_lowercase();
    
    // Verificar presença de keywords de testes
    let test_keyword_count = TEST_KEYWORDS.iter()
        .filter(|&&kw| text_lower.contains(kw))
        .count();
    
    // Verificar resultados numéricos simples (ex: "accuracy", "precision")
    let has_numeric_keywords = NUMERIC_RESULT_PATTERNS.iter()
        .any(|&pattern| text_lower.contains(pattern));
    
    // Aprovar se tiver pelo menos 3 keywords de teste OU termos numéricos
    test_keyword_count >= 3 || has_numeric_keywords
}

// Função legada mantida para compatibilidade (deprecada)
pub fn has_experimental_sections(parsed: &ParsedPdf) -> bool {
    has_tests_in_results(parsed)
}

