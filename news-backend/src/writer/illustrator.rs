// PHASE4: ILLUSTRATOR - Image Extraction Module
// Extracts figures from PDFs and maps DeepSeek recommendations

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use regex::Regex;

/// Extrai todas as imagens de um PDF usando pdfimages (poppler)
/// 
/// Retorna: Vec de PathBufs apontando para imagens extraídas
pub async fn extract_figures_from_pdf(
    pdf_path: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    use std::process::Command;
    
    // Caminho para pdfimages já instalado localmente
    let pdfimages_path = 
        "G:/Hive-Hub/News-main/apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdfimages.exe";
    
    if !std::path::Path::new(pdfimages_path).exists() {
        return Err(anyhow::anyhow!("pdfimages.exe not found at: {}", pdfimages_path));
    }
    
    // Criar diretório temporário para extração
    let temp_extract_dir = output_dir.join("temp_images");
    tokio::fs::create_dir_all(&temp_extract_dir).await
        .context("Failed to create temp_images directory")?;
    
    // Executar: pdfimages -all <pdf> <output_prefix>
    // Resultado: img-000.png, img-001.png, img-002.png, ...
    let output_prefix = temp_extract_dir.join("img");
    
    let output = Command::new(pdfimages_path)
        .arg("-all")  // Todos os formatos de imagem
        .arg(pdf_path)
        .arg(&output_prefix)
        .output()
        .context("Failed to execute pdfimages")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pdfimages failed: {}", stderr));
    }
    
    // Listar arquivos extraídos e ordenar
    let mut extracted_files: Vec<PathBuf> = std::fs::read_dir(&temp_extract_dir)
        .context("Failed to read temp_images directory")?
        .filter_map(|entry| entry.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    
    extracted_files.sort();
    
    Ok(extracted_files)
}

/// Encontra a imagem correspondente à recomendação do DeepSeek
/// 
/// Exemplo: "figure_2.png" ou "figure 2" → img-001.png (segunda imagem, índice 1)
pub fn find_recommended_image(
    recommended_name: &str,
    extracted_images: &[PathBuf],
) -> Option<PathBuf> {
    // Extrair número da figura
    let figure_num = extract_figure_number(recommended_name)?;
    
    if figure_num == 0 {
        return None;
    }
    
    // pdfimages numera a partir de 000, então Figure 1 = img-000, Figure 2 = img-001
    let expected_index = figure_num - 1;
    
    if expected_index < extracted_images.len() {
        Some(extracted_images[expected_index].clone())
    } else {
        // Fallback: tentar buscar por nome que contenha o índice
        let expected_suffix = format!("{:03}", expected_index);
        extracted_images.iter()
            .find(|path| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.contains(&expected_suffix))
                    .unwrap_or(false)
            })
            .cloned()
    }
}

/// Extrai o número de uma referência de figura
/// 
/// Exemplos:
/// - "figure_2.png" → Some(2)
/// - "figure 2" → Some(2)
/// - "Fig. 3" → Some(3)
pub fn extract_figure_number(name: &str) -> Option<usize> {
    let re = Regex::new(r"(?i)(?:figure|fig)[\s_-]*(\d+)").ok()?;
    re.captures(name)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Extrai a legenda (caption) de uma figura específica do texto do PDF
/// 
/// Procura por padrões como:
/// - "Figure N: ... caption text ..."
/// - "Fig. N: ... caption text ..."
/// 
/// Retorna o texto da legenda completo
pub fn extract_figure_caption(text: &str, figure_num: usize) -> Option<String> {
    use regex::Regex;
    
    // Buscar Figure N ou Fig. N seguido de dois pontos ou apenas texto
    let patterns = vec![
        format!(r"(?i)(?:figure|fig)\.?\s*{}\s*[:\-]\s*([^\n]+(?:\n[^\n]+)*?)(?=\n(?:figure|fig)|[A-Z][a-z]+)", figure_num),
        format!(r"(?i)(?:figure|fig)\.?\s*{}\s+(.+?)(?=\n\n|\n(?:Figure|Fig|Abstract))", figure_num),
        format!(r"(?i)(?:figure|fig)\.?\s*{}\s*[:\-]\s*([^\n]+)", figure_num),
    ];
    
    for pattern in patterns {
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(captures) = re.captures(text) {
                if let Some(caption) = captures.get(1) {
                    let caption_text = caption.as_str().trim();
                    if !caption_text.is_empty() && caption_text.len() > 10 {
                        return Some(caption_text.to_string());
                    }
                }
            }
        }
    }
    
    None
}

/// MANTIDO POR COMPATIBILIDADE: Encontra referências textuais a figuras
pub fn find_figure_references(text: &str) -> Vec<String> {
    use regex::Regex;
    
    let mut figures = Vec::new();
    let re = Regex::new(r"(?i)(fig\.?\s+|figure\s+)\d+").unwrap();
    
    for cap in re.find_iter(text) {
        let figure_ref = cap.as_str().trim().to_string();
        let filename = figure_ref
            .replace(" ", "_")
            .replace(".", "")
            .to_lowercase() + ".png";
        figures.push(filename);
    }
    
    figures.sort();
    figures.dedup();
    figures
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_figure_number() {
        assert_eq!(extract_figure_number("figure_2.png"), Some(2));
        assert_eq!(extract_figure_number("figure 2"), Some(2));
        assert_eq!(extract_figure_number("Fig. 3"), Some(3));
        assert_eq!(extract_figure_number("Figure-10"), Some(10));
        assert_eq!(extract_figure_number("random text"), None);
    }
}
