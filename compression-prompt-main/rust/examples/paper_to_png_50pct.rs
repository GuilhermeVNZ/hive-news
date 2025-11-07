use compression_prompt::{ImageRenderer, StatisticalFilter, StatisticalFilterConfig};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Converting Paper to PNG (50% compression only)\n");

    // Read original paper
    let paper_path = "../benchmarks/datasets/arxiv_markdown/1211.5063.md";

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

    // Compress with 50% ratio (balanced)
    let filter = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    });

    let compressed = filter.compress(&text);

    let original_tokens = text.split_whitespace().count();
    let compressed_tokens = compressed.split_whitespace().count();
    let compression_ratio = compressed_tokens as f32 / original_tokens as f32;

    println!("✅ Compression complete:");
    println!("  Original tokens: {}", original_tokens);
    println!("  Compressed tokens: {}", compressed_tokens);
    println!("  Compression ratio: {:.1}%", compression_ratio * 100.0);
    println!(
        "  Tokens saved: {} ({:.1}%)",
        original_tokens - compressed_tokens,
        (1.0 - compression_ratio) * 100.0
    );
    println!();

    // Save compressed text
    let text_filename = "rnn_paper_compressed.txt";
    fs::write(text_filename, &compressed)?;
    println!("📝 Text saved: {}", text_filename);
    println!();

    // Generate PNG image(s)
    let renderer = ImageRenderer::default();

    println!("🖼️  Generating PNG image(s)...");
    println!();

    // Try to render in one image first
    match renderer.render_to_png(&compressed) {
        Ok(png_data) => {
            // Success! Fits in one image
            let png_filename = "rnn_paper_compressed.png";
            fs::write(png_filename, &png_data)?;

            println!("✅ PNG image generated (1 page):");
            println!("  File: {}", png_filename);
            println!(
                "  Size: {:.2} MB ({} bytes)",
                png_data.len() as f32 / 1_048_576.0,
                png_data.len()
            );
            println!("  Dimensions: 1024x1024");
            println!("  Font size: 12.5pt");

            if png_data.len() >= 8 && png_data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
                println!("  ✓ Valid PNG signature");
            }

            println!();
            println!("✅ Done!");
            println!();
            println!("📊 Summary:");
            println!("  Paper: 'On the difficulty of training Recurrent Neural Networks'");
            println!("  Compression: 50% (balanced)");
            println!("  Output: 1 page");
            println!("  Files:");
            println!("    - {} (compressed text)", text_filename);
            println!("    - {} (PNG image)", png_filename);
        }
        Err(_) => {
            // Text too large, split into multiple pages
            println!("⚠️  Text doesn't fit in 1 page, splitting into multiple...");
            println!();

            // Split text into chunks
            let words: Vec<&str> = compressed.split_whitespace().collect();
            let chunk_size = 2000; // words per page (optimized to fill better)
            let chunks: Vec<String> = words
                .chunks(chunk_size)
                .map(|chunk| chunk.join(" "))
                .collect();

            println!("📄 Generating {} pages...", chunks.len());
            println!();

            let mut total_size = 0usize;

            for (i, chunk) in chunks.iter().enumerate() {
                let page_num = i + 1;
                let filename = format!("rnn_paper_compressed_page{}.png", page_num);

                let png_data = renderer.render_to_png(chunk)?;
                fs::write(&filename, &png_data)?;
                total_size += png_data.len();

                println!("✅ Page {}/{}:", page_num, chunks.len());
                println!("  File: {}", filename);
                println!("  Size: {:.2} MB", png_data.len() as f32 / 1_048_576.0);
                println!("  Words: {}", chunk.split_whitespace().count());
            }

            println!();
            println!("✅ Done!");
            println!();
            println!("📊 Summary:");
            println!("  Paper: 'On the difficulty of training Recurrent Neural Networks'");
            println!("  Compression: 50% (balanced)");
            println!("  Output: {} pages", chunks.len());
            println!("  Total size: {:.2} MB", total_size as f32 / 1_048_576.0);
            println!("  Font size: 12.5pt (improved readability)");
            println!();
            println!("  Files:");
            println!("    - {} (compressed text)", text_filename);
            for i in 1..=chunks.len() {
                println!("    - rnn_paper_compressed_page{}.png", i);
            }
        }
    }

    Ok(())
}
