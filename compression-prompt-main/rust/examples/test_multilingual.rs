// Simple multilingual compression test
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};

fn main() {
    println!("🌍 Testing Multilingual Statistical Compression\n");

    let config = StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    };
    let filter = StatisticalFilter::new(config);

    let tests = vec![
        (
            "English",
            "The quick brown fox jumps over the lazy dog. This is a test of statistical compression.",
        ),
        (
            "Spanish",
            "El rápido zorro marrón salta sobre el perro perezoso. Esta es una prueba de compresión estadística.",
        ),
        (
            "French",
            "Le rapide renard brun saute par-dessus le chien paresseux. Ceci est un test de compression statistique.",
        ),
        (
            "German",
            "Der schnelle braune Fuchs springt über den faulen Hund. Dies ist ein Test der statistischen Kompression.",
        ),
        (
            "Portuguese",
            "A rápida raposa marrom pula sobre o cão preguiçoso. Este é um teste de compressão estatística.",
        ),
    ];

    for (lang, text) in tests {
        println!("Language: {}", lang);
        println!("Original: {}", text);

        let compressed = filter.compress(text);

        let original_words = text.split_whitespace().count();
        let compressed_words = compressed.split_whitespace().count();
        let ratio = compressed_words as f32 / original_words as f32;

        println!("Compressed: {}", compressed);
        println!(
            "Ratio: {:.2} ({} -> {} words)\n",
            ratio, original_words, compressed_words
        );
    }

    println!("✅ Multilingual compression test completed!");
}
