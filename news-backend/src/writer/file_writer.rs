// File Writer Module
// Handles saving generated content to disk
use std::path::Path;
use anyhow::Result;
use tokio::fs;
use regex::Regex;

/// Remove formatação markdown indesejada como **Label:** do texto
fn clean_markdown_formatting(content: &str) -> String {
    let mut cleaned = content.to_string();
    
    // Remove padrões **Label:** do início de linhas
    let label_pattern = Regex::new(r"(?m)^\*\*[^:]+:\*\*\s*").unwrap();
    cleaned = label_pattern.replace_all(&cleaned, "").to_string();
    
    // Remove **Label:** no meio do texto (com cuidado para não remover markdown legítimo)
    let inline_label_pattern = Regex::new(r"\*\*([^:]+):\*\*\s+").unwrap();
    cleaned = inline_label_pattern.replace_all(&cleaned, "").to_string();
    
    // Limpar espaços extras entre parágrafos (mais de 2 quebras de linha)
    let extra_newlines = Regex::new(r"\n{3,}").unwrap();
    cleaned = extra_newlines.replace_all(&cleaned, "\n\n").to_string();
    
    // Trim no início e fim
    cleaned.trim().to_string()
}

pub async fn save_article(
    output_dir: &Path,
    content: &str,
) -> Result<()> {
    // Limpar formatação markdown indesejada antes de salvar
    let cleaned_content = clean_markdown_formatting(content);
    fs::write(output_dir.join("article.md"), cleaned_content).await?;
    Ok(())
}

pub async fn save_title(
    output_dir: &Path,
    title: &str,
) -> Result<()> {
    fs::write(output_dir.join("title.txt"), title).await?;
    Ok(())
}

pub async fn save_subtitle(
    output_dir: &Path,
    subtitle: &str,
) -> Result<()> {
    fs::write(output_dir.join("subtitle.txt"), subtitle).await?;
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

#[allow(dead_code)]
pub async fn save_metadata(
    output_dir: &Path,
    metadata: &serde_json::Value,
) -> Result<()> {
    let metadata_str = serde_json::to_string_pretty(metadata)?;
    fs::write(output_dir.join("metadata.json"), metadata_str).await?;
    Ok(())
}

pub async fn save_image_categories(
    output_dir: &Path,
    categories: &[String],
) -> Result<()> {
    let content = categories.join("\n");
    fs::write(output_dir.join("image_categories.txt"), content).await?;
    Ok(())
}

pub async fn save_source(
    output_dir: &Path,
    source: &str,
) -> Result<()> {
    fs::write(output_dir.join("source.txt"), source).await?;
    Ok(())
}
