// Prompt Compression Module
// Integrates with compression-prompt tool to reduce API costs
use crate::utils::path_resolver::resolve_workspace_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PromptCompressor {
    compression_tool_path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompressedPrompt {
    pub original_text: String,
    pub compressed_text: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub compression_ratio: f32,
}

impl PromptCompressor {
    pub fn new() -> Result<Self> {
        let compression_tool_path = Self::locate_compress_binary()?;

        println!(
            "✅ Found compress binary at: {}",
            compression_tool_path.display()
        );

        Ok(Self {
            compression_tool_path,
        })
    }

    fn locate_compress_binary() -> Result<PathBuf> {
        let binary_name = if cfg!(target_os = "windows") {
            "compress.exe"
        } else {
            "compress"
        };

        // Priority 1: Check system PATH (Docker: /usr/local/bin/compress)
        if let Ok(path_result) = Command::new("which").arg(binary_name).output()
            && path_result.status.success()
        {
            let path_str = String::from_utf8_lossy(&path_result.stdout)
                .trim()
                .to_string();
            if !path_str.is_empty() {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Priority 2: Check common Docker location
        let docker_path = PathBuf::from("/usr/local/bin").join(binary_name);
        if docker_path.exists() {
            return Ok(docker_path);
        }

        // Priority 3: Try to locate project directory and pre-built binary
        if let Ok(project_dir) = Self::locate_project_dir() {
            let binary_path = project_dir.join("target").join("release").join(binary_name);
            if binary_path.exists() {
                return Ok(binary_path);
            }

            // Try to build if cargo is available
            if Command::new("cargo").arg("--version").output().is_ok() {
                println!("📦 Building compression-prompt tool...");
                if Self::build_compression_tool(&project_dir).is_ok() && binary_path.exists() {
                    return Ok(binary_path);
                }
            }
        }

        Err(anyhow::anyhow!(
            "compress binary not found. Checked:\n\
             - System PATH (which {})\n\
             - Docker location (/usr/local/bin/{})\n\
             - Project build directory (compression-prompt-main/rust/target/release/{})",
            binary_name,
            binary_name,
            binary_name
        ))
    }

    fn locate_project_dir() -> Result<PathBuf> {
        let candidates = [
            PathBuf::from("compression-prompt-main/rust"),
            PathBuf::from("../compression-prompt-main/rust"),
            PathBuf::from("../../compression-prompt-main/rust"),
            resolve_workspace_path("compression-prompt-main/rust"),
        ];

        for candidate in candidates.iter() {
            if candidate.exists() {
                return Ok(candidate.clone());
            }
        }

        Err(anyhow::anyhow!(
            "compression-prompt project directory not found"
        ))
    }

    fn build_compression_tool(project_dir: &Path) -> Result<()> {
        let output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(project_dir)
            .output()
            .context("Failed to execute cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to build compression-prompt: {}",
                stderr
            ));
        }

        println!("✅ Compression tool built successfully");
        Ok(())
    }

    pub fn compress(&self, text: &str) -> Result<CompressedPrompt> {
        let original_tokens = self.count_tokens(text);

        // Create temp file with prompt
        let temp_dir = std::env::temp_dir();
        let temp_input = temp_dir.join(format!(
            "prompt_input_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        std::fs::write(&temp_input, text).context("Failed to write temp file")?;

        // Run compression tool
        // Use -r 0.5 for 50% compression ratio (default quality ~89%)
        let output = Command::new(&self.compression_tool_path)
            .arg(&temp_input)
            .arg("-r")
            .arg("0.5")
            .output()
            .context("Failed to run compression tool")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Compression failed: {}", stderr);

            // Clean up
            let _ = std::fs::remove_file(&temp_input);

            return Err(anyhow::anyhow!("Compression process failed"));
        }

        let mut compressed_text = String::from_utf8_lossy(&output.stdout).to_string();

        // Ensure the compressed prompt contains the JSON instruction
        let has_json_original = text.to_lowercase().contains("json");
        let has_json_compressed = compressed_text.to_lowercase().contains("json");

        if has_json_original && !has_json_compressed {
            // Add a fixed JSON instruction at the end with explicit format requirements
            compressed_text.push_str("\n\n## CRITICAL: JSON OUTPUT REQUIRED - FOLLOW THIS EXACT FORMAT:\n{{\"title\": \"...\", \"article_text\": \"...\", \"image_categories\": [...]}}\n⚠️ \"article_text\" MUST be a STRING field at root level (NOT nested in an \"article\" object)");
        }

        let compressed_tokens = self.count_tokens(&compressed_text);
        let compression_ratio = if original_tokens > 0 {
            1.0 - (compressed_tokens as f32 / original_tokens as f32)
        } else {
            0.0
        };

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_input);

        Ok(CompressedPrompt {
            original_text: text.to_string(),
            compressed_text,
            original_tokens,
            compressed_tokens,
            compression_ratio,
        })
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Simple approximation: ~4 chars per token
        // For production, consider using tiktoken or similar
        (text.len() as f32 / 4.0).ceil() as usize
    }
}
