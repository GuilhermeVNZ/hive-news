// File Writer Module
// Handles saving generated content to disk
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use tokio::fs;

/// Remove formatação markdown indesejada como **Label:** do texto
/// Também remove "Opening Hook" e outros labels que indicam conteúdo gerado por IA
fn clean_markdown_formatting(content: &str) -> String {
    let mut cleaned = content.to_string();

    // Remove padrões **Label:** do início de linhas
    let label_pattern = Regex::new(r"(?m)^\*\*[^:]+:\*\*\s*").unwrap();
    cleaned = label_pattern.replace_all(&cleaned, "").to_string();

    // Remove **Label:** no meio do texto (com cuidado para não remover markdown legítimo)
    let inline_label_pattern = Regex::new(r"\*\*([^:]+):\*\*\s+").unwrap();
    cleaned = inline_label_pattern.replace_all(&cleaned, "").to_string();

    // CRITICAL: Remove TODOS os labels estruturais que indicam conteúdo gerado por IA
    // Lista completa de labels comuns gerados pelo DeepSeek
    // Usamos (?i) para case-insensitive, então não precisamos de todas as variações
    let ai_label_patterns = vec![
        // Opening Hook
        r"(?i)^\s*\*\*Opening Hook\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Opening Hook\s*[—–-]?\s*",
        // The Challenge / Challenge
        r"(?i)^\s*\*\*The Challenge\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Challenge\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Challenge\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Challenge\s*[—–-]?\s*",
        // The Method / Method / Methodology
        r"(?i)^\s*\*\*The Method\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Method\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Method\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Method\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Methodology\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Methodology\s*[—–-]?\s*",
        // The Discovery / Discovery
        r"(?i)^\s*\*\*The Discovery\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Discovery\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Discovery\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Discovery\s*[—–-]?\s*",
        // The Findings / Findings
        r"(?i)^\s*\*\*The Findings\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Findings\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Findings\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Findings\s*[—–-]?\s*",
        // The Results / Results
        r"(?i)^\s*\*\*The Results\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Results\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Results\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Results\s*[—–-]?\s*",
        // The Implications / Implications
        r"(?i)^\s*\*\*The Implications\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Implications\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Implications\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Implications\s*[—–-]?\s*",
        // The Significance / Significance
        r"(?i)^\s*\*\*The Significance\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Significance\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Significance\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Significance\s*[—–-]?\s*",
        // Key Finding / Key Findings
        r"(?i)^\s*\*\*Key Finding\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Key Finding\s*[—–-]?\s*",
        r"(?i)^\s*\*\*Key Findings\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Key Findings\s*[—–-]?\s*",
        // Context
        r"(?i)^\s*\*\*Context\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Context\s*[—–-]?\s*",
        // Limitations / The Limitations
        r"(?i)^\s*\*\*Limitations\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Limitations\s*[—–-]?\s*",
        r"(?i)^\s*\*\*The Limitations\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Limitations\s*[—–-]?\s*",
        // Background / The Background
        r"(?i)^\s*\*\*Background\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Background\s*[—–-]?\s*",
        r"(?i)^\s*\*\*The Background\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Background\s*[—–-]?\s*",
        // Analysis / The Analysis
        r"(?i)^\s*\*\*Analysis\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Analysis\s*[—–-]?\s*",
        r"(?i)^\s*\*\*The Analysis\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Analysis\s*[—–-]?\s*",
        // Conclusion / The Conclusion
        r"(?i)^\s*\*\*Conclusion\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Conclusion\s*[—–-]?\s*",
        r"(?i)^\s*\*\*The Conclusion\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Conclusion\s*[—–-]?\s*",
        // Discussion / The Discussion
        r"(?i)^\s*\*\*Discussion\*\*\s*[—–-]?\s*",
        r"(?i)^\s*Discussion\s*[—–-]?\s*",
        r"(?i)^\s*\*\*The Discussion\*\*\s*[—–-]?\s*",
        r"(?i)^\s*The Discussion\s*[—–-]?\s*",
    ];
    
    // Remove cada padrão no início de linha
    for pattern in &ai_label_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }
    }
    
    // Remove também no meio do texto (se aparecer) - versões inline
    let ai_label_inline_patterns = vec![
        r"(?i)\*\*Opening Hook\*\*\s*[—–-]?\s*",
        r"(?i)Opening Hook\s*[—–-]?\s*",
        r"(?i)\*\*The Challenge\*\*\s*[—–-]?\s*",
        r"(?i)The Challenge\s*[—–-]?\s*",
        r"(?i)\*\*Challenge\*\*\s*[—–-]?\s*",
        r"(?i)Challenge\s*[—–-]?\s*",
        r"(?i)\*\*The Method\*\*\s*[—–-]?\s*",
        r"(?i)The Method\s*[—–-]?\s*",
        r"(?i)\*\*Method\*\*\s*[—–-]?\s*",
        r"(?i)Method\s*[—–-]?\s*",
        r"(?i)\*\*Methodology\*\*\s*[—–-]?\s*",
        r"(?i)Methodology\s*[—–-]?\s*",
        r"(?i)\*\*The Discovery\*\*\s*[—–-]?\s*",
        r"(?i)The Discovery\s*[—–-]?\s*",
        r"(?i)\*\*Discovery\*\*\s*[—–-]?\s*",
        r"(?i)Discovery\s*[—–-]?\s*",
        r"(?i)\*\*The Findings\*\*\s*[—–-]?\s*",
        r"(?i)The Findings\s*[—–-]?\s*",
        r"(?i)\*\*Findings\*\*\s*[—–-]?\s*",
        r"(?i)Findings\s*[—–-]?\s*",
        r"(?i)\*\*The Results\*\*\s*[—–-]?\s*",
        r"(?i)The Results\s*[—–-]?\s*",
        r"(?i)\*\*Results\*\*\s*[—–-]?\s*",
        r"(?i)Results\s*[—–-]?\s*",
        r"(?i)\*\*The Implications\*\*\s*[—–-]?\s*",
        r"(?i)The Implications\s*[—–-]?\s*",
        r"(?i)\*\*Implications\*\*\s*[—–-]?\s*",
        r"(?i)Implications\s*[—–-]?\s*",
    ];
    
    for pattern in &ai_label_inline_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }
    }

    // Limpar espaços extras entre parágrafos (mais de 2 quebras de linha)
    let extra_newlines = Regex::new(r"\n{3,}").unwrap();
    cleaned = extra_newlines.replace_all(&cleaned, "\n\n").to_string();

    // Trim no início e fim
    cleaned.trim().to_string()
}

/// Limpa referências duplicadas e garante apenas uma referência científica ao final do artigo
/// Remove múltiplas seções de "References", "Bibliography", "Works Cited", etc.
/// Mantém apenas a primeira referência encontrada (assumindo que é a do artigo fonte)
fn clean_duplicate_references(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut cleaned_lines: Vec<&str> = Vec::new();
    let mut seen_reference_sections = 0;
    let mut in_reference_section = false;
    let mut reference_section_start = 0;
    
    // Padrão para detectar início de seção de referências
    let section_header_pattern = Regex::new(r"(?i)^(?:##?\s*)?(?:References|Bibliography|Works Cited|Sources|Fontes|Referências|Source|Fonte)\s*$").unwrap();
    
    // Padrão para detectar linhas de referência (números, colchetes, parênteses)
    let citation_pattern = Regex::new(r"^\s*(\d+\.|\[.*?\]|\(.*?\)|\-|\*)\s*").unwrap();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Detectar início de seção de referências
        if section_header_pattern.is_match(trimmed) {
            seen_reference_sections += 1;
            
            // Se já vimos uma seção de referências, pular esta e todas as seguintes
            if seen_reference_sections > 1 {
                in_reference_section = true;
                continue;
            }
            
            // Primeira seção de referências - manter
            in_reference_section = true;
            reference_section_start = cleaned_lines.len();
            cleaned_lines.push(line);
            continue;
        }
        
        // Se estamos em uma seção de referências duplicada, pular todas as linhas
        if in_reference_section && seen_reference_sections > 1 {
            // Verificar se esta linha parece ser o fim da seção (linha vazia seguida de conteúdo não-referência)
            if trimmed.is_empty() {
                // Verificar próxima linha para ver se ainda estamos na seção
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if !citation_pattern.is_match(next_line) && !section_header_pattern.is_match(next_line) && !next_line.is_empty() {
                        // Próxima linha não é referência, sair da seção
                        in_reference_section = false;
                    }
                }
            }
            continue;
        }
        
        // Se estamos na primeira seção de referências, verificar duplicatas dentro da seção
        if in_reference_section && seen_reference_sections == 1 {
            // Se a linha parece ser uma referência, verificar se é duplicata
            if citation_pattern.is_match(trimmed) {
                // Normalizar para comparação (remover marcadores de citação)
                let normalized = citation_pattern
                    .replace(trimmed, "")
                    .trim()
                    .to_lowercase();
                
                // Verificar se já vimos esta referência antes nesta seção
                let mut is_duplicate = false;
                for prev_line in cleaned_lines.iter().skip(reference_section_start) {
                    let prev_trimmed = prev_line.trim();
                    if citation_pattern.is_match(prev_trimmed) {
                        let prev_normalized = citation_pattern
                            .replace(prev_trimmed, "")
                            .trim()
                            .to_lowercase();
                        
                        // Comparar referências normalizadas (ignorando diferenças de formatação)
                        if !normalized.is_empty() && !prev_normalized.is_empty() {
                            // Comparação simples: se o conteúdo principal é muito similar, considerar duplicata
                            let similarity = normalized.chars().filter(|c| prev_normalized.contains(*c)).count();
                            if similarity as f32 / normalized.len().max(1) as f32 > 0.8 {
                                is_duplicate = true;
                                break;
                            }
                        }
                    }
                }
                
                if is_duplicate {
                    continue;
                }
            }
            
            // Verificar se saímos da seção de referências
            if trimmed.is_empty() && i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if !citation_pattern.is_match(next_line) && !section_header_pattern.is_match(next_line) && !next_line.is_empty() {
                    in_reference_section = false;
                }
            }
        }
        
        cleaned_lines.push(line);
    }
    
    cleaned_lines.join("\n")
}

/// Garante que cada parágrafo comece com letra maiúscula
/// Processa quebras de linha duplas como separadores de parágrafo
fn capitalize_paragraphs(content: &str) -> String {
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut capitalized_paragraphs = Vec::new();
    
    for paragraph in paragraphs {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        // Encontrar a primeira letra alfabética e capitalizá-la
        let mut chars: Vec<char> = trimmed.chars().collect();
        let mut found_letter = false;
        
        for i in 0..chars.len() {
            if chars[i].is_alphabetic() {
                chars[i] = chars[i].to_uppercase().next().unwrap_or(chars[i]);
                found_letter = true;
                break;
            }
        }
        
        if found_letter {
            capitalized_paragraphs.push(chars.into_iter().collect::<String>());
        } else {
            // Se não encontrou letra alfabética, manter original
            capitalized_paragraphs.push(trimmed.to_string());
        }
    }
    
    capitalized_paragraphs.join("\n\n")
}

pub async fn save_article(output_dir: &Path, content: &str) -> Result<()> {
    // Limpar formatação markdown indesejada antes de salvar
    let mut cleaned_content = clean_markdown_formatting(content);
    
    // Limpar referências duplicadas e garantir apenas uma referência científica
    cleaned_content = clean_duplicate_references(&cleaned_content);
    
    // Garantir que cada parágrafo comece com letra maiúscula
    cleaned_content = capitalize_paragraphs(&cleaned_content);
    
    fs::write(output_dir.join("article.md"), cleaned_content).await?;
    Ok(())
}

pub async fn save_title(output_dir: &Path, title: &str) -> Result<()> {
    fs::write(output_dir.join("title.txt"), title).await?;
    Ok(())
}

pub async fn save_subtitle(output_dir: &Path, subtitle: &str) -> Result<()> {
    fs::write(output_dir.join("subtitle.txt"), subtitle).await?;
    Ok(())
}

pub async fn save_linkedin(output_dir: &Path, content: &str) -> Result<()> {
    fs::write(output_dir.join("linkedin.txt"), content).await?;
    Ok(())
}

pub async fn save_x(output_dir: &Path, content: &str) -> Result<()> {
    fs::write(output_dir.join("x.txt"), content).await?;
    Ok(())
}

pub async fn save_shorts_script(output_dir: &Path, content: &str) -> Result<()> {
    fs::write(output_dir.join("shorts_script.txt"), content).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn save_metadata(output_dir: &Path, metadata: &serde_json::Value) -> Result<()> {
    let metadata_str = serde_json::to_string_pretty(metadata)?;
    fs::write(output_dir.join("metadata.json"), metadata_str).await?;
    Ok(())
}

pub async fn save_image_categories(output_dir: &Path, categories: &[String]) -> Result<()> {
    let content = categories.join("\n");
    fs::write(output_dir.join("image_categories.txt"), content).await?;
    Ok(())
}

pub async fn save_source(output_dir: &Path, source: &str) -> Result<()> {
    fs::write(output_dir.join("source.txt"), source).await?;
    Ok(())
}

pub async fn save_slug(output_dir: &Path, slug: &str) -> Result<()> {
    fs::write(output_dir.join("slug.txt"), slug).await?;
    Ok(())
}
