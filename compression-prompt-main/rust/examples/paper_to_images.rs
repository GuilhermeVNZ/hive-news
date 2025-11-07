use compression_prompt::{OutputFormat, StatisticalFilter, StatisticalFilterConfig};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Converting Paper to Compressed Images\n");

    // Read original paper
    let paper_path = "../benchmarks/datasets/arxiv_markdown/1505.07818.md";

    if !Path::new(paper_path).exists() {
        eprintln!("❌ Paper file not found: {}", paper_path);
        return Err("Paper file not found".into());
    }

    let text = fs::read_to_string(paper_path)?;

    println!("📝 Original paper:");
    println!("  File: {}", paper_path);
    println!("  Size: {} chars", text.len());
    println!("  Words: {}", text.split_whitespace().count());
    println!();

    let sep = "=".repeat(80);

    // Configuration for different compression levels
    let configs = vec![
        (0.3, "aggressive", "30% compression"),
        (0.5, "balanced", "50% compression (default)"),
        (0.7, "light", "70% compression"),
    ];

    for (ratio, label, description) in configs {
        println!("{}", sep);
        println!("Processing: {} - {}", label.to_uppercase(), description);
        println!("{}", sep);

        let config = StatisticalFilterConfig {
            compression_ratio: ratio,
            ..Default::default()
        };

        let filter = StatisticalFilter::new(config);
        let result = filter.compress_with_format(&text, OutputFormat::Image)?;

        println!("✅ Compression complete:");
        println!("  Original tokens: {}", result.original_tokens);
        println!("  Compressed tokens: {}", result.compressed_tokens);
        println!(
            "  Compression ratio: {:.1}%",
            result.compression_ratio * 100.0
        );
        println!("  Tokens saved: {}", result.tokens_removed);
        println!(
            "  Token savings: {:.1}%",
            (1.0 - result.compression_ratio) * 100.0
        );

        // Save compressed text
        let text_filename = format!("rnn_paper_{}_compressed.txt", label);
        fs::write(&text_filename, &result.compressed)?;
        println!("  📝 Text saved: {}", text_filename);

        // Save image
        if let Some(img_data) = &result.image_data {
            let img_filename = format!("rnn_paper_{}_compressed.png", label);
            fs::write(&img_filename, img_data)?;

            println!("  🖼️  Image saved: {}", img_filename);
            println!(
                "     Size: {} KB ({} bytes)",
                img_data.len() / 1024,
                img_data.len()
            );
            println!("     Dimensions: 1024x1024 PNG");

            // Verify PNG signature
            if img_data.len() >= 8 && img_data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
                println!("     ✓ Valid PNG signature");
            }

            // Calculate compression efficiency
            let chars_per_kb = result.compressed.len() as f32 / (img_data.len() as f32 / 1024.0);
            println!("     Density: {:.1} chars/KB", chars_per_kb);
        }

        println!();
    }

    println!("{}", sep);
    println!("✅ Paper conversion complete!");
    println!("{}", sep);
    println!();
    println!("📊 Summary:");
    println!("  Input: {}", paper_path);
    println!("  Paper: 'On the difficulty of training Recurrent Neural Networks'");
    println!("  Generated files:");
    println!("    - rnn_paper_aggressive_compressed.txt + .png (30% compression)");
    println!("    - rnn_paper_balanced_compressed.txt + .png (50% compression)");
    println!("    - rnn_paper_light_compressed.txt + .png (70% compression)");
    println!();
    println!("💡 Use these PNG images with vision models for optical context compression!");

    Ok(())
}
