use compression_prompt::{StatisticalFilter, StatisticalFilterConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Statistical Compression Pipeline\n");

    // Read test input - using a larger benchmark file
    let test_file = "../large_test_input.txt";
    let text = fs::read_to_string(test_file)?;

    println!("📝 Original text ({} chars):", text.len());
    println!("{}\n", &text[..200.min(text.len())]);

    let sep = "=".repeat(80);

    // Test 1: Default configuration (50% compression)
    println!("{}", sep);
    println!("Test 1: Default Configuration (50% compression)");
    println!("{}", sep);

    let filter1 = StatisticalFilter::default();
    let compressed1 = filter1.compress(&text);

    let original_words = text.split_whitespace().count();
    let compressed_words = compressed1.split_whitespace().count();
    let ratio = compressed_words as f32 / original_words as f32;

    println!("Compression ratio: {:.3}", ratio);
    println!("Words saved: {:.1}%", (1.0 - ratio) * 100.0);
    println!("Original words: {}", original_words);
    println!("Compressed words: {}", compressed_words);
    println!("\nFirst 200 chars of compressed text:");
    println!("{}\n", &compressed1[..200.min(compressed1.len())]);

    // Test 2: Aggressive compression (30%)
    println!("{}", sep);
    println!("Test 2: Aggressive Compression (30%)");
    println!("{}", sep);

    let config2 = StatisticalFilterConfig {
        compression_ratio: 0.3,
        ..Default::default()
    };
    let filter2 = StatisticalFilter::new(config2);
    let compressed2 = filter2.compress(&text);

    let compressed_words2 = compressed2.split_whitespace().count();
    let ratio2 = compressed_words2 as f32 / original_words as f32;

    println!("Compression ratio: {:.3}", ratio2);
    println!("Words saved: {:.1}%", (1.0 - ratio2) * 100.0);
    println!("Original words: {}", original_words);
    println!("Compressed words: {}", compressed_words2);
    println!("\nFirst 200 chars of compressed text:");
    println!("{}\n", &compressed2[..200.min(compressed2.len())]);

    // Test 3: Light compression (70%)
    println!("{}", sep);
    println!("Test 3: Light Compression (70%)");
    println!("{}", sep);

    let config3 = StatisticalFilterConfig {
        compression_ratio: 0.7,
        ..Default::default()
    };
    let filter3 = StatisticalFilter::new(config3);
    let compressed3 = filter3.compress(&text);

    let compressed_words3 = compressed3.split_whitespace().count();
    let ratio3 = compressed_words3 as f32 / original_words as f32;

    println!("Compression ratio: {:.3}", ratio3);
    println!("Words saved: {:.1}%", (1.0 - ratio3) * 100.0);
    println!("Original words: {}", original_words);
    println!("Compressed words: {}", compressed_words3);
    println!("\nFirst 200 chars of compressed text:");
    println!("{}\n", &compressed3[..200.min(compressed3.len())]);

    println!("{}", sep);
    println!("✅ All compression tests completed!");
    println!("{}", sep);

    Ok(())
}
