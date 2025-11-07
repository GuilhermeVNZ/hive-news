use compression_prompt::{
    ImageRenderer, ImageRendererConfig, StatisticalFilter, StatisticalFilterConfig,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Comparing PNG vs JPEG Image Formats\n");

    // Read the RNN paper
    let paper_path = "../benchmarks/datasets/arxiv_markdown/1211.5063.md";
    let text = fs::read_to_string(paper_path)?;

    println!("📝 Original paper: {} chars\n", text.len());

    // Compress with balanced ratio (50%)
    let filter = StatisticalFilter::new(StatisticalFilterConfig {
        compression_ratio: 0.5,
        ..Default::default()
    });
    let compressed = filter.compress(&text);

    println!(
        "✅ Compressed: {} chars ({:.1}% of original)\n",
        compressed.len(),
        (compressed.len() as f32 / text.len() as f32) * 100.0
    );

    let renderer = ImageRenderer::new(ImageRendererConfig::default());

    let sep = "=".repeat(80);

    // Test PNG
    println!("{}", sep);
    println!("PNG Format");
    println!("{}", sep);

    let png_data = renderer.render_to_png(&compressed)?;
    fs::write("format_test.png", &png_data)?;

    println!("✅ PNG generated:");
    println!("  File: format_test.png");
    println!(
        "  Size: {:.2} MB ({} bytes)",
        png_data.len() as f32 / 1_048_576.0,
        png_data.len()
    );
    println!("  Compression: Lossless");
    println!();

    // Test JPEG with different quality levels
    let jpeg_qualities = vec![
        (95, "Máxima (95)"),
        (90, "Alta (90)"),
        (85, "Boa (85)"),
        (80, "Média-Alta (80)"),
        (75, "Média (75)"),
        (70, "Aceitável (70)"),
        (60, "Baixa (60)"),
    ];

    println!("{}", sep);
    println!("JPEG Format - Quality Comparison");
    println!("{}", sep);

    let mut best_quality = 0;
    let mut best_size = 0;
    let mut best_ratio = 0.0f32;

    for (quality, label) in &jpeg_qualities {
        let jpeg_data = renderer.render_to_jpeg(&compressed, *quality)?;
        let filename = format!("format_test_q{}.jpg", quality);
        fs::write(&filename, &jpeg_data)?;

        let size_mb = jpeg_data.len() as f32 / 1_048_576.0;
        let reduction = (1.0 - (jpeg_data.len() as f32 / png_data.len() as f32)) * 100.0;

        println!("✅ JPEG Quality {} - {}:", quality, label);
        println!("  File: {}", filename);
        println!("  Size: {:.2} MB ({} bytes)", size_mb, jpeg_data.len());
        println!(
            "  vs PNG: -{:.1}% ({:.2} MB saved)",
            reduction,
            (png_data.len() - jpeg_data.len()) as f32 / 1_048_576.0
        );

        // Track best balance (quality 85 is usually the sweet spot)
        if *quality == 85 {
            best_quality = *quality;
            best_size = jpeg_data.len();
            best_ratio = reduction;
        }

        println!();
    }

    println!("{}", sep);
    println!("📊 Summary & Recommendation");
    println!("{}", sep);
    println!();
    println!("PNG:");
    println!("  ✓ Lossless (texto perfeitamente legível)");
    println!(
        "  ✗ Arquivo maior: {:.2} MB",
        png_data.len() as f32 / 1_048_576.0
    );
    println!();
    println!("JPEG Quality {} (RECOMENDADO):", best_quality);
    println!("  ✓ Redução de {:.1}% no tamanho", best_ratio);
    println!(
        "  ✓ Economia de {:.2} MB",
        (png_data.len() - best_size) as f32 / 1_048_576.0
    );
    println!("  ✓ Ainda legível para OCR/Vision models");
    println!(
        "  ✓ Tamanho final: {:.2} MB",
        best_size as f32 / 1_048_576.0
    );
    println!();
    println!("💡 Para máxima economia com qualidade aceitável:");
    println!("   - Use JPEG quality 85 para documentos técnicos");
    println!("   - Use JPEG quality 90 para máxima legibilidade");
    println!("   - Use JPEG quality 75-80 se tamanho for crítico");
    println!();
    println!("📁 Arquivos gerados:");
    println!("   - format_test.png (PNG)");
    for (quality, _) in jpeg_qualities {
        println!(
            "   - format_test_q{}.jpg (JPEG quality {})",
            quality, quality
        );
    }

    Ok(())
}
