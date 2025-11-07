//! Generate A/B test files for LLM comparison
//!
//! Creates matched pairs of original vs compressed prompts
//! for testing with actual language models

use compression_prompt::compressor::{Compressor, CompressorConfig};
use compression_prompt::statistical_filter::{StatisticalFilter, StatisticalFilterConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ABTest {
    test_id: String,
    technique: String,
    original_prompt: String,
    compressed_prompt: String,
    original_tokens: usize,
    compressed_tokens: usize,
    compression_ratio: f64,
    metadata: TestMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestMetadata {
    paper_id: Option<String>,
    dictionary_entries: Option<usize>,
    substitutions: Option<usize>,
    processing_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ABTestSuite {
    version: String,
    created_at: String,
    tests: Vec<ABTest>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Generating A/B Test Suite for LLM Evaluation\n");

    let mut test_suite = ABTestSuite {
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        tests: Vec::new(),
    };

    // Load datasets
    let benchmark_files = vec![
        (
            "benchmark_100",
            "../benchmarks/datasets/prompts/benchmark_100_papers.txt",
        ),
        (
            "benchmark_200",
            "../benchmarks/datasets/prompts/benchmark_200_papers.txt",
        ),
    ];

    for (name, path) in benchmark_files {
        if let Ok(content) = fs::read_to_string(path) {
            println!("📄 Processing: {}", name);

            // Split into individual papers
            let papers: Vec<&str> = content
                .split("\n\n")
                .filter(|p| p.trim().len() > 500)
                .take(20) // Take first 20 papers for A/B testing
                .collect();

            println!("   Found {} papers\n", papers.len());

            for (idx, paper) in papers.iter().enumerate() {
                // Test 1: Dictionary compression
                test_dictionary_compression(
                    &mut test_suite,
                    paper,
                    &format!("{}_paper_{}", name, idx + 1),
                )?;

                // Test 2: Statistical filtering (50%)
                test_statistical_filtering(
                    &mut test_suite,
                    paper,
                    &format!("{}_paper_{}", name, idx + 1),
                    0.5,
                )?;

                // Test 3: Statistical filtering (70%)
                test_statistical_filtering(
                    &mut test_suite,
                    paper,
                    &format!("{}_paper_{}", name, idx + 1),
                    0.7,
                )?;

                // Test 4: Hybrid approach
                test_hybrid_compression(
                    &mut test_suite,
                    paper,
                    &format!("{}_paper_{}", name, idx + 1),
                )?;
            }
        }
    }

    // Save test suite
    let output_dir = PathBuf::from("../benchmarks/ab_tests");
    fs::create_dir_all(&output_dir)?;

    let output_file = output_dir.join("ab_test_suite.json");
    let json = serde_json::to_string_pretty(&test_suite)?;
    fs::write(&output_file, json)?;

    println!("\n✅ Generated {} A/B tests", test_suite.tests.len());
    println!("📁 Saved to: {}", output_file.display());

    // Also save individual test files for easier LLM testing
    save_individual_tests(&test_suite, &output_dir)?;

    // Generate markdown comparison files
    generate_markdown_comparison(&test_suite, &output_dir)?;

    Ok(())
}

fn test_dictionary_compression(
    suite: &mut ABTestSuite,
    text: &str,
    paper_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = CompressorConfig::default();
    let compressor = Compressor::new(config);

    let start = std::time::Instant::now();
    match compressor.compress(text) {
        Ok(result) => {
            let duration = start.elapsed();

            suite.tests.push(ABTest {
                test_id: format!("{}_dictionary", paper_id),
                technique: "dictionary".to_string(),
                original_prompt: text.to_string(),
                compressed_prompt: result.compressed.clone(),
                original_tokens: result.original_tokens,
                compressed_tokens: result.compressed_tokens,
                compression_ratio: result.compression_ratio as f64,
                metadata: TestMetadata {
                    paper_id: Some(paper_id.to_string()),
                    dictionary_entries: None,
                    substitutions: None,
                    processing_time_ms: duration.as_secs_f64() * 1000.0,
                },
            });
        }
        Err(_) => {
            // Skip if compression fails (text too small, etc.)
        }
    }

    Ok(())
}

fn test_statistical_filtering(
    suite: &mut ABTestSuite,
    text: &str,
    paper_id: &str,
    ratio: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = StatisticalFilterConfig {
        compression_ratio: ratio,
        ..Default::default()
    };
    let filter = StatisticalFilter::new(config);

    let start = std::time::Instant::now();
    let compressed = filter.compress(text);
    let duration = start.elapsed();

    let original_tokens = text.split_whitespace().count();
    let compressed_tokens = compressed.split_whitespace().count();
    let actual_ratio = compressed_tokens as f64 / original_tokens as f64;

    suite.tests.push(ABTest {
        test_id: format!("{}_statistical_{:.0}", paper_id, ratio * 100.0),
        technique: format!("statistical_{:.0}%", ratio * 100.0),
        original_prompt: text.to_string(),
        compressed_prompt: compressed,
        original_tokens,
        compressed_tokens,
        compression_ratio: actual_ratio,
        metadata: TestMetadata {
            paper_id: Some(paper_id.to_string()),
            dictionary_entries: None,
            substitutions: None,
            processing_time_ms: duration.as_secs_f64() * 1000.0,
        },
    });

    Ok(())
}

fn test_hybrid_compression(
    suite: &mut ABTestSuite,
    text: &str,
    paper_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dict_config = CompressorConfig::default();
    let dict_compressor = Compressor::new(dict_config);

    let start = std::time::Instant::now();
    match dict_compressor.compress(text) {
        Ok(dict_result) => {
            // Apply statistical filtering to dictionary result
            let stat_config = StatisticalFilterConfig {
                compression_ratio: 0.5,
                ..Default::default()
            };
            let stat_filter = StatisticalFilter::new(stat_config);
            let final_compressed = stat_filter.compress(&dict_result.compressed);
            let duration = start.elapsed();

            let original_tokens = text.split_whitespace().count();
            let compressed_tokens = final_compressed.split_whitespace().count();
            let compression_ratio = compressed_tokens as f64 / original_tokens as f64;

            suite.tests.push(ABTest {
                test_id: format!("{}_hybrid", paper_id),
                technique: "hybrid".to_string(),
                original_prompt: text.to_string(),
                compressed_prompt: final_compressed,
                original_tokens,
                compressed_tokens,
                compression_ratio,
                metadata: TestMetadata {
                    paper_id: Some(paper_id.to_string()),
                    dictionary_entries: None,
                    substitutions: None,
                    processing_time_ms: duration.as_secs_f64() * 1000.0,
                },
            });
        }
        Err(_) => {
            // Skip if dictionary compression fails
        }
    }

    Ok(())
}

fn save_individual_tests(
    suite: &ABTestSuite,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let individual_dir = output_dir.join("individual_tests");
    fs::create_dir_all(&individual_dir)?;

    for test in &suite.tests {
        let test_file = individual_dir.join(format!("{}.json", test.test_id));
        let json = serde_json::to_string_pretty(test)?;
        fs::write(&test_file, json)?;
    }

    println!("💾 Saved {} individual test files", suite.tests.len());

    Ok(())
}

fn generate_markdown_comparison(
    suite: &ABTestSuite,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();

    markdown.push_str("# A/B Test Suite - Compression Techniques Comparison\n\n");
    markdown.push_str(&format!("Generated: {}\n\n", suite.created_at));
    markdown.push_str(&format!("Total tests: {}\n\n", suite.tests.len()));

    // Summary table
    markdown.push_str("## Summary Statistics\n\n");
    markdown.push_str("| Technique | Avg Compression Ratio | Avg Token Savings | Tests |\n");
    markdown.push_str("|-----------|----------------------|-------------------|-------|\n");

    let techniques = vec!["dictionary", "statistical_50%", "statistical_70%", "hybrid"];

    for technique in techniques {
        let tests: Vec<_> = suite
            .tests
            .iter()
            .filter(|t| t.technique == technique)
            .collect();

        if !tests.is_empty() {
            let avg_ratio: f64 =
                tests.iter().map(|t| t.compression_ratio).sum::<f64>() / tests.len() as f64;
            let avg_savings: f64 = tests
                .iter()
                .map(|t| ((1.0 - t.compression_ratio) * 100.0))
                .sum::<f64>()
                / tests.len() as f64;

            markdown.push_str(&format!(
                "| {} | {:.3} | {:.1}% | {} |\n",
                technique,
                avg_ratio,
                avg_savings,
                tests.len()
            ));
        }
    }

    markdown.push_str("\n## Individual Test Results\n\n");

    for test in &suite.tests {
        markdown.push_str(&format!("### {}\n\n", test.test_id));
        markdown.push_str(&format!("**Technique:** {}\n\n", test.technique));
        markdown.push_str(&format!("- Original tokens: {}\n", test.original_tokens));
        markdown.push_str(&format!(
            "- Compressed tokens: {}\n",
            test.compressed_tokens
        ));
        markdown.push_str(&format!(
            "- Compression ratio: {:.3}\n",
            test.compression_ratio
        ));
        markdown.push_str(&format!(
            "- Token savings: {:.1}%\n",
            (1.0 - test.compression_ratio) * 100.0
        ));
        markdown.push_str(&format!(
            "- Processing time: {:.2}ms\n",
            test.metadata.processing_time_ms
        ));

        if let Some(entries) = test.metadata.dictionary_entries {
            markdown.push_str(&format!("- Dictionary entries: {}\n", entries));
        }
        if let Some(subs) = test.metadata.substitutions {
            markdown.push_str(&format!("- Substitutions: {}\n", subs));
        }

        markdown.push_str("\n---\n\n");
    }

    let markdown_file = output_dir.join("ab_test_comparison.md");
    fs::write(&markdown_file, markdown)?;

    println!(
        "📊 Generated comparison report: {}",
        markdown_file.display()
    );

    Ok(())
}
