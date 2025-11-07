use compression_prompt::{OutputFormat, StatisticalFilter, StatisticalFilterConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  Testing Text-to-Image Compression Pipeline\n");

    // Read test input
    let test_file = "../large_test_input.txt";
    let text = fs::read_to_string(test_file)?;

    println!(
        "📝 Original text ({} chars, {} words)",
        text.len(),
        text.split_whitespace().count()
    );

    let sep = "=".repeat(80);

    // Test 1: Compress to text (default)
    println!("\n{}", sep);
    println!("Test 1: Compress to Text (default)");
    println!("{}", sep);

    let filter = StatisticalFilter::default();
    let result_text = filter.compress_with_format(&text, OutputFormat::Text)?;

    println!("✅ Compressed to text:");
    println!("  Original tokens: {}", result_text.original_tokens);
    println!("  Compressed tokens: {}", result_text.compressed_tokens);
    println!(
        "  Compression ratio: {:.1}%",
        result_text.compression_ratio * 100.0
    );
    println!("  Tokens saved: {}", result_text.tokens_removed);
    println!("  Format: {:?}", result_text.format);

    // Save compressed text
    fs::write("output_compressed.txt", &result_text.compressed)?;
    println!("  Saved: output_compressed.txt");

    // Test 2: Compress to image (1024x1024 PNG)
    println!("\n{}", sep);
    println!("Test 2: Compress to Image (1024x1024 PNG)");
    println!("{}", sep);

    let result_image = filter.compress_with_format(&text, OutputFormat::Image)?;

    println!("✅ Compressed to image:");
    println!("  Original tokens: {}", result_image.original_tokens);
    println!("  Compressed tokens: {}", result_image.compressed_tokens);
    println!(
        "  Compression ratio: {:.1}%",
        result_image.compression_ratio * 100.0
    );
    println!("  Tokens saved: {}", result_image.tokens_removed);
    println!("  Format: {:?}", result_image.format);

    if let Some(img_data) = &result_image.image_data {
        println!(
            "  Image size: {} bytes ({:.1} KB)",
            img_data.len(),
            img_data.len() as f32 / 1024.0
        );

        fs::write("output_compressed.png", img_data)?;
        println!("  Saved: output_compressed.png");

        // Verify PNG signature
        if img_data.len() >= 8 && img_data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
            println!("  ✓ Valid PNG signature");
        }
    }

    // Test 3: Aggressive compression to image (30%)
    println!("\n{}", sep);
    println!("Test 3: Aggressive Compression to Image (30%)");
    println!("{}", sep);

    let config_aggressive = StatisticalFilterConfig {
        compression_ratio: 0.3,
        ..Default::default()
    };
    let filter_aggressive = StatisticalFilter::new(config_aggressive);
    let result_aggressive = filter_aggressive.compress_with_format(&text, OutputFormat::Image)?;

    println!("✅ Aggressive compression:");
    println!(
        "  Compression ratio: {:.1}%",
        result_aggressive.compression_ratio * 100.0
    );
    println!("  Tokens saved: {}", result_aggressive.tokens_removed);

    if let Some(img_data) = &result_aggressive.image_data {
        println!(
            "  Image size: {} bytes ({:.1} KB)",
            img_data.len(),
            img_data.len() as f32 / 1024.0
        );

        fs::write("output_compressed_30pct.png", img_data)?;
        println!("  Saved: output_compressed_30pct.png");
    }

    // Test 4: Light compression to image (70%)
    println!("\n{}", sep);
    println!("Test 4: Light Compression to Image (70%)");
    println!("{}", sep);

    let config_light = StatisticalFilterConfig {
        compression_ratio: 0.7,
        ..Default::default()
    };
    let filter_light = StatisticalFilter::new(config_light);
    let result_light = filter_light.compress_with_format(&text, OutputFormat::Image)?;

    println!("✅ Light compression:");
    println!(
        "  Compression ratio: {:.1}%",
        result_light.compression_ratio * 100.0
    );
    println!("  Tokens saved: {}", result_light.tokens_removed);

    if let Some(img_data) = &result_light.image_data {
        println!(
            "  Image size: {} bytes ({:.1} KB)",
            img_data.len(),
            img_data.len() as f32 / 1024.0
        );

        fs::write("output_compressed_70pct.png", img_data)?;
        println!("  Saved: output_compressed_70pct.png");
    }

    println!("\n{}", sep);
    println!("✅ All image compression tests completed!");
    println!("{}", sep);
    println!("\n📊 Summary:");
    println!("  - Text output: output_compressed.txt");
    println!("  - Image output (50%): output_compressed.png");
    println!("  - Image output (30%): output_compressed_30pct.png");
    println!("  - Image output (70%): output_compressed_70pct.png");
    println!("\n💡 Use these images with vision models for optical context compression!");

    Ok(())
}
