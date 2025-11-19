use anyhow::Result;
use lopdf::Document;
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::utils::path_resolver::resolve_workspace_path;

#[allow(dead_code)]
pub struct ParsedPdf {
    pub file_path: String,
    pub raw_doc_id: i32,
    pub source_name: String,
    pub source_url: String,
    pub title: String,
    pub authors: Vec<String>,
    pub text: String,
    pub dois: Vec<String>,
    pub sections: Vec<String>,
}

pub fn parse_pdf(path: &Path) -> Result<ParsedPdf> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Extrair metadados do path
    let raw_doc_id = 1;
    let source_name = "arxiv".to_string();
    let source_url = format!("https://arxiv.org/abs/{}", file_name);

    // Tentar parser real com lopdf (avisos de encoding são esperados e não impedem extração)
    let text = match parse_pdf_text(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("⚠️  Failed to parse PDF {}: {}", path.display(), e);
            // Fallback: retornar vazio - será rejeitado pelo filter se não tiver conteúdo suficiente
            String::new()
        }
    };

    // Extrair informações do texto real
    let dois = extract_dois(&text);
    let sections = extract_sections(&text);

    // Tentar extrair título (primeira linha com texto)
    let title = extract_title(&text, file_name);

    Ok(ParsedPdf {
        file_path: path.to_string_lossy().to_string(),
        raw_doc_id,
        source_name,
        source_url,
        title,
        authors: vec![], // TODO: Extrair autores
        text,
        dois,
        sections,
    })
}

fn parse_pdf_text(path: &Path) -> Result<String> {
    use std::process::Command;

    // Caminho para pdftotext instalado localmente
    let pdftotext_path: PathBuf = std::env::var("PDFTOTEXT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            resolve_workspace_path(
                "apps/Release-25.07.0-0/poppler-25.07.0/Library/bin/pdftotext.exe",
            )
        });

    // Verificar se pdftotext existe
    if pdftotext_path.exists() {
        let output = Command::new(&pdftotext_path)
            .arg(path.as_os_str())
            .arg("-") // output para stdout
            .output()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.len() > 100 {
                return Ok(text.to_string());
            }
        }
    }

    // Estratégia 2: Tentar lopdf (silenciando avisos de encoding)
    if let Ok(doc) = Document::load(path) {
        let mut full_text = String::new();
        let pages = doc.get_pages();

        if !pages.is_empty() {
            for (page_id, _) in pages.iter() {
                // lopdf pode gerar avisos de encoding - ignora-los silenciosamente
                if let Ok(text) = doc.extract_text(&[*page_id]) {
                    full_text.push_str(&text);
                    full_text.push('\n');
                }
            }

            if full_text.len() > 100 {
                return Ok(full_text);
            }
        }
    }

    // Estratégia 3: Leitura direta de bytes brutos
    if let Ok(bytes) = std::fs::read(path) {
        let text = extract_text_from_bytes(&bytes);
        if text.len() > 100 {
            return Ok(text);
        }
    }

    // Fallback: retornar string vazia (log será colapsado no pipeline)
    Ok(String::new())
}

fn extract_text_from_bytes(bytes: &[u8]) -> String {
    use std::str;

    let mut text = String::new();
    let pdf_str = str::from_utf8(bytes).unwrap_or("");

    // Procurar por padrões comuns de texto em PDFs
    let mut cursor = 0;

    while let Some(start) = pdf_str[cursor..].find("BT") {
        cursor += start + 2;

        if let Some(end) = pdf_str[cursor..].find("ET") {
            let text_block = &pdf_str[cursor..cursor + end];

            // Extrair texto entre parênteses (comum em PDFs)
            for line in text_block.lines() {
                let mut current = line;
                while let Some(open) = current.find('(') {
                    if let Some(close) = current[open..].find(')') {
                        let text_content = &current[open + 1..open + close];
                        text.push_str(text_content);
                        text.push(' ');
                        current = &current[open + close + 1..];
                    } else {
                        break;
                    }
                }
            }

            cursor += end;
        } else {
            break;
        }
    }

    // Limpar caracteres de controle e normalizar espaços
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
        .collect::<String>()
        .replace("  ", " ")
        .trim()
        .to_string()
}

pub fn extract_dois(text: &str) -> Vec<String> {
    let doi_regex = Regex::new(r"10\.\d{4,}/[^\s]+").unwrap();
    doi_regex
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn extract_title(text: &str, fallback: &str) -> String {
    // Tentar encontrar primeira linha substancial (mais de 5 palavras)
    for line in text.lines() {
        let line = line.trim();
        if line.len() > 30 && line.split_whitespace().count() > 5 {
            return line.to_string();
        }
    }

    // Fallback para nome do arquivo
    fallback.to_string()
}

#[allow(dead_code)]
pub fn extract_authors(_text: &str) -> Vec<String> {
    // TODO: Extrair autores do PDF
    vec![]
}

pub fn extract_sections(text: &str) -> Vec<String> {
    let sections = vec!["Abstract", "Introduction", "Method", "Results"];
    sections
        .into_iter()
        .filter(|&s| text.contains(s))
        .map(|s| s.to_string())
        .collect()
}
