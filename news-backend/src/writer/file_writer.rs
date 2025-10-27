// File Writer Module
// Handles saving generated content to disk
use std::path::Path;
use anyhow::Result;
use tokio::fs;

pub async fn save_article(
    output_dir: &Path,
    content: &str,
) -> Result<()> {
    fs::write(output_dir.join("article.md"), content).await?;
    Ok(())
}

pub async fn save_linkedin(
    output_dir: &Path,
    content: &str,
) -> Result<()> {
    fs::write(output_dir.join("linkedin.txt"), content).await?;
    Ok(())
}

pub async fn save_x(
    output_dir: &Path,
    content: &str,
) -> Result<()> {
    fs::write(output_dir.join("x.txt"), content).await?;
    Ok(())
}

pub async fn save_shorts_script(
    output_dir: &Path,
    content: &str,
) -> Result<()> {
    fs::write(output_dir.join("shorts_script.txt"), content).await?;
    Ok(())
}

pub async fn save_metadata(
    output_dir: &Path,
    metadata: &serde_json::Value,
) -> Result<()> {
    let metadata_str = serde_json::to_string_pretty(metadata)?;
    fs::write(output_dir.join("metadata.json"), metadata_str).await?;
    Ok(())
}
