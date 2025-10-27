use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use std::fs;

use crate::filter::parser::ParsedPdf;
use crate::filter::source_detector::{SourceType, detect_source_type};
use crate::filter::experiments::has_experimental_sections;
use crate::filter::fake_detector::calculate_fake_penalty;
use crate::filter::validator::validate_dois;
use crate::filter::authors::validate_authors;
use crate::filter::scorer::{FilterResult, calculate_score};
use crate::filter::categorizer::categorize;

#[derive(Default)]
pub struct FilterStats {
    pub total: usize,
    pub approved: usize,
    pub rejected: usize,
    pub skipped: usize,
}

pub async fn run_filter_pipeline(download_dir: &Path) -> Result<FilterStats> {
    let pdfs = discover_unfiltered_pdfs(download_dir)?;
    
    if pdfs.is_empty() {
        println!("   No unfiltered PDFs found");
        return Ok(FilterStats {
            total: 0,
            approved: 0,
            rejected: 0,
            skipped: 0,
        });
    }
    
    println!("   Found {} unfiltered PDFs", pdfs.len());
    
    // Processar cada PDF sequencialmente por enquanto
    // TODO: Implementar pipeline paralelo com rayon + tokio
    
    let mut stats = FilterStats::default();
    stats.total = pdfs.len();
    
    for pdf_path in pdfs {
        // Parse do PDF
        let parsed = match crate::filter::parser::parse_pdf(&pdf_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("   Failed to parse {}: {}", pdf_path.display(), e);
                stats.rejected += 1;
                continue;
            }
        };
        
        // Detectar tipo de fonte
        let source_type = detect_source_type(&parsed);
        
        if source_type == SourceType::NonScientific {
            println!("   Skipping non-scientific source: {}", pdf_path.display());
            stats.skipped += 1;
            continue;
        }
        
        // Filtros rápidos
        let has_tests = has_experimental_sections(&parsed); // Agora verifica testes nos resultados
        let fake_penalty = calculate_fake_penalty(&parsed.text);
        
        if !has_tests || fake_penalty > 0.5 {
            println!("   Rejected (no tests in results or high fake penalty): {}", parsed.title);
            stats.rejected += 1;
            continue;
        }
        
        // Validação via APIs
        let doi_ratio = validate_dois(&parsed.dois).await;
        let author_ratio = validate_authors(&parsed.authors).await;
        
        let result = FilterResult {
            doc: parsed,
            doi_ratio,
            author_ratio,
            has_exp: has_tests, // Usar has_tests para has_exp
            fake_penalty,
        };
        
        let score = calculate_score(&result);
        
        if score >= 0.5 { // Threshold reduzido de 0.7 para 0.5
            let category = categorize(&result.doc);
            println!("   Approved (score: {:.2}): {} → {}", score, result.doc.title, category);
            stats.approved += 1;
            
            // Mover para /filtered/<category>/
            move_to_category(&pdf_path, &category, download_dir)?;
        } else {
            println!("   Rejected (score: {:.2}): {}", score, result.doc.title);
            stats.rejected += 1;
            
            // Mover para /rejected/
            move_to_rejected(&pdf_path, download_dir)?;
        }
    }
    
    Ok(stats)
}

fn discover_unfiltered_pdfs(download_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut pdfs = Vec::new();
    
    if !download_dir.exists() {
        return Ok(pdfs);
    }
    
    // Buscar PDFs recursivamente
    fn find_pdfs(dir: &Path, pdfs: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursão para subdiretórios
                find_pdfs(&path, pdfs)?;
            } else if let Some(ext) = path.extension() {
                if ext == "pdf" {
                    pdfs.push(path);
                }
            }
        }
        Ok(())
    }
    
    find_pdfs(download_dir, &mut pdfs)?;
    Ok(pdfs)
}

fn move_to_category(pdf_path: &Path, category: &str, base_dir: &Path) -> Result<()> {
    let category_dir = base_dir.join("filtered").join(category);
    
    // Criar diretório se não existir
    if !category_dir.exists() {
        fs::create_dir_all(&category_dir)?;
    }
    
    // Obter nome do arquivo
    let filename = pdf_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf");
    
    let dest_path = category_dir.join(filename);
    
    // Mover arquivo
    fs::rename(pdf_path, &dest_path)?;
    
    Ok(())
}

fn move_to_rejected(pdf_path: &Path, base_dir: &Path) -> Result<()> {
    let rejected_dir = base_dir.join("rejected");
    
    // Criar diretório se não existir
    if !rejected_dir.exists() {
        fs::create_dir_all(&rejected_dir)?;
    }
    
    // Obter nome do arquivo
    let filename = pdf_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf");
    
    let dest_path = rejected_dir.join(filename);
    
    // Mover arquivo
    fs::rename(pdf_path, &dest_path)?;
    
    Ok(())
}

 