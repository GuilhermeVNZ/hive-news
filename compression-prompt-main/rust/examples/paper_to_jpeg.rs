use compression_prompt::{
    ImageRenderer, ImageRendererConfig, StatisticalFilter, StatisticalFilterConfig,
};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Converting Paper to Compressed JPEG Images\n");

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

    let sep = "=".repeat(80);

    // Configuration for different compression levels
    let configs = vec![
        (0.3, "aggressive", "30% compression", 85),
        (0.5, "balanced", "50% compression (default)", 85),
        (0.7, "light", "70% compression", 85),
    ];

    let renderer = ImageRenderer::new(ImageRendererConfig::default());
    let mut total_png_size = 0usize;
    let mut total_jpeg_size = 0usize;

    for (ratio, label, description, jpeg_quality) in configs {
        println!("{}", sep);
        println!("Processing: {} - {}", label.to_uppercase(), description);
        println!("{}", sep);

        let config = StatisticalFilterConfig {
            compression_ratio: ratio,
            ..Default::default()
        };

        let filter = StatisticalFilter::new(config);
        let compressed = filter.compress(&text);

        let original_tokens = text.split_whitespace().count();
        let compressed_tokens = compressed.split_whitespace().count();
        let compression_ratio_actual = compressed_tokens as f32 / original_tokens as f32;

        println!("✅ Compression complete:");
        println!("  Original tokens: {}", original_tokens);
        println!("  Compressed tokens: {}", compressed_tokens);
        println!(
            "  Compression ratio: {:.1}%",
            compression_ratio_actual * 100.0
        );
        println!("  Tokens saved: {}", original_tokens - compressed_tokens);
        println!(
            "  Token savings: {:.1}%",
            (1.0 - compression_ratio_actual) * 100.0
        );

        // Save compressed text
        let text_filename = format!("rnn_paper_{}_compressed.txt", label);
        fs::write(&text_filename, &compressed)?;
        println!("  📝 Text saved: {}", text_filename);

        // Generate PNG (for comparison)
        let png_data = renderer.render_to_png(&compressed)?;
        let png_filename = format!("rnn_paper_{}_compressed.png", label);
        fs::write(&png_filename, &png_data)?;
        total_png_size += png_data.len();

        println!("  🖼️  PNG saved: {}", png_filename);
        println!(
            "     Size: {:.2} MB ({} bytes)",
            png_data.len() as f32 / 1_048_576.0,
            png_data.len()
        );

        // Generate JPEG
        let jpeg_data = renderer.render_to_jpeg(&compressed, jpeg_quality)?;
        let jpeg_filename = format!("rnn_paper_{}_compressed_q{}.jpg", label, jpeg_quality);
        fs::write(&jpeg_filename, &jpeg_data)?;
        total_jpeg_size += jpeg_data.len();

        let reduction = (1.0 - (jpeg_data.len() as f32 / png_data.len() as f32)) * 100.0;

        println!("  📷 JPEG saved: {}", jpeg_filename);
        println!(
            "     Size: {:.2} MB ({} bytes)",
            jpeg_data.len() as f32 / 1_048_576.0,
            jpeg_data.len()
        );
        println!("     Quality: {}", jpeg_quality);
        println!(
            "     vs PNG: -{:.1}% ({:.2} MB saved)",
            reduction,
            (png_data.len() - jpeg_data.len()) as f32 / 1_048_576.0
        );

        // Verify JPEG signature
        if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
            println!("     ✓ Valid JPEG signature");
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
    println!();
    println!(
        "  Total PNG size: {:.2} MB",
        total_png_size as f32 / 1_048_576.0
    );
    println!(
        "  Total JPEG size: {:.2} MB",
        total_jpeg_size as f32 / 1_048_576.0
    );
    println!(
        "  Total saved: {:.2} MB ({:.1}% reduction)",
        (total_png_size - total_jpeg_size) as f32 / 1_048_576.0,
        (1.0 - (total_jpeg_size as f32 / total_png_size as f32)) * 100.0
    );
    println!();
    println!("📁 Generated files (PNG + JPEG):");
    println!("   - rnn_paper_aggressive_compressed.txt + .png + _q85.jpg");
    println!("   - rnn_paper_balanced_compressed.txt + .png + _q85.jpg");
    println!("   - rnn_paper_light_compressed.txt + .png + _q85.jpg");
    println!();
    println!("💡 Use JPEG images for:");
    println!("   ✓ Menor tamanho de arquivo (~66% de redução vs PNG)");
    println!("   ✓ Upload mais rápido para APIs de Vision models");
    println!("   ✓ Ainda perfeitamente legível para OCR");
    println!("   ✓ Ideal para produção em larga escala");

    Ok(())
}
