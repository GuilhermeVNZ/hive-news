// Prompt Compression Module
// Integrates with compression-prompt tool to reduce API costs
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};

pub struct PromptCompressor {
    compression_tool_path: PathBuf,
}

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
        let compression_tool_path = PathBuf::from(
            "G:/Hive-Hub/compression-prompt-main/rust/target/release/compress.exe"
        );
        
        if !compression_tool_path.exists() {
            // Try to build if not found
            Self::build_compression_tool()?;
        }
        
        Ok(Self { compression_tool_path })
    }
    
    fn build_compression_tool() -> Result<()> {
        println!("📦 Building compression-prompt tool...");
        
        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir("G:/Hive-Hub/compression-prompt-main/rust")
            .output()
            .context("Failed to execute cargo build")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Build failed: {}", stderr);
            return Err(anyhow::anyhow!("Failed to build compression-prompt"));
        }
        
        println!("✅ Compression tool built successfully");
        Ok(())
    }
    
    pub fn compress(&self, text: &str) -> Result<CompressedPrompt> {
        let original_tokens = self.count_tokens(text);
        
        // Create temp file with prompt
        let temp_dir = std::env::temp_dir();
        let temp_input = temp_dir.join(format!("prompt_input_{}.txt", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()));
        
        std::fs::write(&temp_input, text)
            .context("Failed to write temp file")?;
        
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
            // Add a fixed JSON instruction at the end
            compressed_text.push_str("\n\n## JSON OUTPUT REQUIRED:\nYou MUST return valid JSON only:\n{{\"title\": \"...\", \"article_text\": \"...\"}}");
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
