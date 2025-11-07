//! CLI tool for compressing text using compression-prompt

use compression_prompt::{Compressor, CompressorConfig, OutputFormat, StatisticalFilterConfig};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

fn print_usage() {
    eprintln!("Usage: compress [OPTIONS] [INPUT_FILE]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -r, --ratio <RATIO>      Compression ratio (0.0-1.0, default: 0.5)");
    eprintln!("  -o, --output <FILE>      Output file (default: stdout)");
    eprintln!("  -f, --format <FORMAT>    Output format: text, png, jpeg (default: text)");
    eprintln!("  -q, --quality <QUALITY>  JPEG quality 1-100 (default: 85, only for jpeg)");
    eprintln!("  -s, --stats              Show compression statistics");
    eprintln!("  -h, --help               Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  compress input.txt                        # Compress to stdout");
    eprintln!("  compress -r 0.7 input.txt                 # Conservative (70%)");
    eprintln!("  compress -r 0.3 input.txt                 # Aggressive (30%)");
    eprintln!("  compress -f png -o out.png input.txt      # Save as PNG image");
    eprintln!("  compress -f jpeg -q 90 -o out.jpg input.txt  # Save as JPEG");
    eprintln!("  cat input.txt | compress                  # Read from stdin");
}

enum OutputFormatType {
    Text,
    Png,
    Jpeg,
}

struct Config {
    input_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    compression_ratio: f32,
    show_stats: bool,
    output_format: OutputFormatType,
    jpeg_quality: u8,
}

impl Config {
    fn parse_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();

        let mut config = Config {
            input_file: None,
            output_file: None,
            compression_ratio: 0.5,
            show_stats: false,
            output_format: OutputFormatType::Text,
            jpeg_quality: 85,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    print_usage();
                    process::exit(0);
                }
                "-r" | "--ratio" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --ratio".to_string());
                    }
                    config.compression_ratio = args[i]
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid ratio: {}", args[i]))?;

                    if !(0.0..=1.0).contains(&config.compression_ratio) {
                        return Err("Ratio must be between 0.0 and 1.0".to_string());
                    }
                }
                "-o" | "--output" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --output".to_string());
                    }
                    config.output_file = Some(PathBuf::from(&args[i]));
                }
                "-f" | "--format" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --format".to_string());
                    }
                    config.output_format = match args[i].to_lowercase().as_str() {
                        "text" => OutputFormatType::Text,
                        "png" => OutputFormatType::Png,
                        "jpeg" | "jpg" => OutputFormatType::Jpeg,
                        _ => {
                            return Err(format!(
                                "Invalid format: {} (use: text, png, jpeg)",
                                args[i]
                            ));
                        }
                    };
                }
                "-q" | "--quality" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --quality".to_string());
                    }
                    config.jpeg_quality = args[i]
                        .parse::<u8>()
                        .map_err(|_| format!("Invalid quality: {}", args[i]))?;

                    if !(1..=100).contains(&config.jpeg_quality) {
                        return Err("Quality must be between 1 and 100".to_string());
                    }
                }
                "-s" | "--stats" => {
                    config.show_stats = true;
                }
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                }
                arg => {
                    config.input_file = Some(PathBuf::from(arg));
                }
            }
            i += 1;
        }

        Ok(config)
    }
}

fn read_input(config: &Config) -> io::Result<String> {
    if let Some(ref input_file) = config.input_file {
        fs::read_to_string(input_file)
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

fn write_output(config: &Config, text: &str) -> io::Result<()> {
    if let Some(ref output_file) = config.output_file {
        fs::write(output_file, text)
    } else {
        io::stdout().write_all(text.as_bytes())?;
        io::stdout().flush()
    }
}

fn main() {
    let config = match Config::parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    };

    // Read input
    let input = match read_input(&config) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    // Configure compressor
    let compressor_config = CompressorConfig {
        target_ratio: config.compression_ratio,
        min_input_bytes: 100, // Lower threshold for CLI
        min_input_tokens: 10,
    };

    let filter_config = StatisticalFilterConfig {
        compression_ratio: config.compression_ratio,
        ..Default::default()
    };

    let compressor = Compressor::with_filter_config(compressor_config, filter_config);

    // Determine output format
    let output_format = match config.output_format {
        OutputFormatType::Text => OutputFormat::Text,
        OutputFormatType::Png | OutputFormatType::Jpeg => {
            #[cfg(not(feature = "image"))]
            {
                eprintln!("Error: Image output requires the 'image' feature");
                eprintln!("Rebuild with: cargo build --features image");
                process::exit(1);
            }
            #[cfg(feature = "image")]
            OutputFormat::Image
        }
    };

    // Compress
    let result = match compressor.compress_with_format(&input, output_format) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Compression error: {}", e);
            process::exit(1);
        }
    };

    // Show stats if requested
    if config.show_stats {
        eprintln!("Compression Statistics:");
        eprintln!("  Original tokens:   {}", result.original_tokens);
        eprintln!("  Compressed tokens: {}", result.compressed_tokens);
        eprintln!("  Tokens removed:    {}", result.tokens_removed);
        eprintln!(
            "  Compression ratio: {:.1}%",
            (1.0 - result.compression_ratio) * 100.0
        );
        eprintln!(
            "  Target ratio:      {:.1}%",
            (1.0 - config.compression_ratio) * 100.0
        );
        eprintln!();
    }

    // Write output based on format
    match config.output_format {
        OutputFormatType::Text => {
            if let Err(e) = write_output(&config, &result.compressed) {
                eprintln!("Error writing output: {}", e);
                process::exit(1);
            }
        }
        OutputFormatType::Png => {
            #[cfg(feature = "image")]
            {
                if let Some(image_data) = result.image_data {
                    let output_path = config
                        .output_file
                        .as_ref()
                        .map(|p| p.to_owned())
                        .unwrap_or_else(|| PathBuf::from("compressed.png"));

                    if let Err(e) = fs::write(&output_path, image_data) {
                        eprintln!("Error writing PNG: {}", e);
                        process::exit(1);
                    }

                    if config.show_stats {
                        eprintln!("  Output saved to: {}", output_path.display());
                    }
                } else {
                    eprintln!("Error: Failed to generate PNG image");
                    process::exit(1);
                }
            }
        }
        OutputFormatType::Jpeg => {
            #[cfg(feature = "image")]
            {
                use compression_prompt::ImageRenderer;

                let renderer = ImageRenderer::default();
                match renderer.render_to_jpeg(&result.compressed, config.jpeg_quality) {
                    Ok(jpeg_data) => {
                        let output_path = config
                            .output_file
                            .as_ref()
                            .map(|p| p.to_owned())
                            .unwrap_or_else(|| PathBuf::from("compressed.jpg"));

                        if let Err(e) = fs::write(&output_path, jpeg_data) {
                            eprintln!("Error writing JPEG: {}", e);
                            process::exit(1);
                        }

                        if config.show_stats {
                            eprintln!("  Output saved to: {}", output_path.display());
                        }
                    }
                    Err(e) => {
                        eprintln!("Error generating JPEG: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }
}
