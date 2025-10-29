use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

use crate::filter::authors::validate_authors;
use crate::filter::categorizer::categorize;
use crate::filter::experiments::has_experimental_sections;
use crate::filter::fake_detector::calculate_fake_penalty;
use crate::filter::parser::ParsedPdf;
use crate::filter::scorer::{FilterResult, calculate_score};
use crate::filter::source_detector::{SourceType, detect_source_type};
use crate::filter::validator::validate_dois;

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
            println!(
                "   Rejected (no tests in results or high fake penalty): {}",
                parsed.title
            );
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

        if score >= 0.5 {
            // Threshold reduzido de 0.7 para 0.5
            let category = categorize(&result.doc);
            println!(
                "   Approved (score: {:.2}): {} → {}",
                score, result.doc.title, category
            );
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

    // Check if article already has writer output
    fn article_already_processed(pdf_path: &Path, download_dir: &Path) -> bool {
        // Extract article ID from PDF path (e.g., "2510.21610.pdf" -> "2510.21610")
        let article_id = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // Check multiple possible output locations
        let output_locations = vec![
            download_dir.parent().map(|p| p.join("output").join("AIResearch").join(article_id)),
            Some(download_dir.parent().unwrap_or(download_dir).join("output").join("AIResearch").join(article_id)),
            Some(PathBuf::from("G:/Hive-Hub/News-main/output/AIResearch").join(article_id)),
            Some(PathBuf::from("G:/Hive-Hub/News-main/output/ScienceAI").join(article_id)),
        ];
        
        for opt_path in output_locations {
            if let Some(output_dir) = opt_path {
                let article_file = output_dir.join("article.md");
                if article_file.exists() {
                    return true;
                }
            }
        }
        
        false
    }

    // Buscar PDFs recursivamente de downloads/ (ONLY arxiv/, skip filtered/ e rejected/)
    fn find_pdfs(dir: &Path, pdfs: &mut Vec<PathBuf>, download_dir: &Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                
                // Skip filtered, rejected, cache subdirectories
                if dir_name == "filtered" || dir_name == "rejected" || dir_name == "cache" || dir_name == "temp" {
                    continue;
                }
                
                // Recursão para subdiretórios
                find_pdfs(&path, pdfs, download_dir)?;
            } else if let Some(ext) = path.extension() {
                if ext == "pdf" {
                    // Check if article already processed by writer
                    if !article_already_processed(&path, download_dir) {
                        pdfs.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    // Search in downloads/ (mainly from arxiv/, excluding filtered/, rejected/, cache/)
    find_pdfs(download_dir, &mut pdfs, download_dir)?;
    
    Ok(pdfs)
}

fn move_to_category(pdf_path: &Path, category: &str, base_dir: &Path) -> Result<()> {
    let category_dir = base_dir.join("filtered").join(category);

    // Criar diretório se não existir
    if !category_dir.exists() {
        fs::create_dir_all(&category_dir)?;
    }

    // Obter nome do arquivo
    let filename = pdf_path
        .file_name()
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
    let filename = pdf_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.pdf");

    let dest_path = rejected_dir.join(filename);

    // Mover arquivo
    fs::rename(pdf_path, &dest_path)?;

    Ok(())
}
