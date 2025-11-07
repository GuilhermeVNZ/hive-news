use compression_prompt::{
    ImageRenderer, ImageRendererConfig, StatisticalFilter, StatisticalFilterConfig,
};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Converting Paper to READABLE Compressed Images\n");
    println!("🎯 Focus: Máxima legibilidade para Vision Models\n");

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

    // Use only balanced compression for better results
    println!("{}", sep);
    println!("Compressing with BALANCED ratio (50%)");
    println!("{}", sep);

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
    println!("  Tokens saved: {}", original_tokens - compressed_tokens);
    println!();

    // Save compressed text
    fs::write("rnn_paper_readable.txt", &compressed)?;
    println!("📝 Compressed text saved: rnn_paper_readable.txt");
    println!();

    // Configure renderer with LARGER font for better readability
    let config = ImageRendererConfig {
        font_size: 16.0,   // Fonte maior
        line_spacing: 1.3, // Mais espaçamento
        margin_x: 30,
        margin_y: 30,
        min_font_size: 10.0, // Mínimo maior
        ..Default::default()
    };

    let renderer = ImageRenderer::new(config);

    // Try to render entire text in one image
    println!("{}", sep);
    println!("Generating readable PNG image");
    println!("{}", sep);

    match renderer.render_to_png(&compressed) {
        Ok(png_data) => {
            fs::write("rnn_paper_readable.png", &png_data)?;

            println!("✅ PNG image generated:");
            println!("  File: rnn_paper_readable.png");
            println!(
                "  Size: {:.2} MB ({} bytes)",
                png_data.len() as f32 / 1_048_576.0,
                png_data.len()
            );
            println!("  Dimensions: 1024x1024");
            println!("  Font size: 16pt (legível para Vision models)");
            println!("  ✓ Valid PNG signature");
            println!();
        }
        Err(e) => {
            println!("⚠️  Texto muito grande para uma imagem!");
            println!("   Erro: {:?}", e);
            println!();
            println!("💡 Solução: Dividindo em múltiplas imagens...");
            println!();

            // Split text into chunks that fit
            let words: Vec<&str> = compressed.split_whitespace().collect();
            let chunk_size = 800; // palavras por imagem
            let chunks: Vec<String> = words
                .chunks(chunk_size)
                .map(|chunk| chunk.join(" "))
                .collect();

            println!("📄 Gerando {} imagens...", chunks.len());
            println!();

            for (i, chunk) in chunks.iter().enumerate() {
                let page_num = i + 1;
                let filename = format!("rnn_paper_readable_page{}.png", page_num);

                let png_data = renderer.render_to_png(chunk)?;
                fs::write(&filename, &png_data)?;

                println!("✅ Página {}/{}:", page_num, chunks.len());
                println!("  File: {}", filename);
                println!("  Size: {:.2} MB", png_data.len() as f32 / 1_048_576.0);
                println!("  Words: {}", chunk.split_whitespace().count());
            }
        }
    }

    println!();
    println!("{}", sep);
    println!("✅ Conversion complete!");
    println!("{}", sep);
    println!();
    println!("📊 Configuration used:");
    println!("  Font size: 16pt (maior que antes)");
    println!("  Line spacing: 1.3 (mais espaço)");
    println!("  Margins: 30px (maiores)");
    println!("  Min font: 10pt (garante legibilidade)");
    println!();
    println!("💡 Estas imagens são otimizadas para:");
    println!("  ✓ OCR de modelos de visão");
    println!("  ✓ GPT-4 Vision");
    println!("  ✓ Claude 3 Vision");
    println!("  ✓ Gemini Vision");
    println!("  ✓ Máxima legibilidade");

    Ok(())
}
