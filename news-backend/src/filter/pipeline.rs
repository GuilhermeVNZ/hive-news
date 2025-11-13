use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::filter::authors::validate_authors;
use crate::filter::categorizer::categorize;
use crate::filter::experiments::has_experimental_sections;
use crate::filter::fake_detector::calculate_fake_penalty;
use crate::filter::scorer::{FilterResult, calculate_score};
use crate::filter::source_detector::{SourceType, detect_source_type};
use crate::filter::validator::validate_dois;
use crate::utils::article_registry::{ArticleStatus, RegistryManager};
use crate::utils::path_resolver::resolve_workspace_path;

#[derive(Default)]
pub struct FilterStats {
    pub total: usize,
    pub approved: usize,
    pub rejected: usize,
    pub skipped: usize,
}

pub async fn run_filter_pipeline(download_dir: &Path) -> Result<FilterStats> {
    // Threshold para aprovação: score >= 0.4
    const FILTER_THRESHOLD: f32 = 0.4;

    // Inicializar registry
    let registry_path = resolve_workspace_path("articles_registry.json");
    let registry = RegistryManager::new(&registry_path)?;

    let pdfs = discover_unfiltered_pdfs(download_dir, &registry)?;

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

    let mut stats = FilterStats {
        total: pdfs.len(),
        ..FilterStats::default()
    };

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

        // Extrair article_id do caminho do PDF
        let article_id = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // Verificar se o artigo existe no registry antes de tentar atualizar
        // Se não existir, criar uma entrada básica primeiro
        if !registry.is_article_registered(article_id) {
            // Criar entrada básica no registry se não existir
            // Isso pode acontecer se o PDF foi descoberto antes do registro ser concluído
            let pdf_url = pdf_path.to_string_lossy().to_string();
            let arxiv_url = if article_id.contains(".") {
                format!("https://arxiv.org/abs/{}", article_id)
            } else {
                pdf_url.clone()
            };

            if let Err(e) = registry.register_collected(
                article_id.to_string(),
                result.doc.title.clone(),
                arxiv_url,
                pdf_url,
            ) {
                eprintln!(
                    "   ⚠️  Failed to create registry entry for article {}: {}",
                    article_id, e
                );
                stats.rejected += 1;
                continue;
            }
        }

        // Arredondar score para 2 casas decimais para evitar problemas de precisão float
        // Se arredondado for >= 0.40, aprovar
        let rounded_score = (score * 100.0).round() / 100.0;

        // Threshold: aprovar se score arredondado >= 0.4
        if rounded_score >= FILTER_THRESHOLD {
            let category = categorize(&result.doc);
            println!(
                "   Approved (score: {:.2}): {} → {}",
                score, result.doc.title, category
            );
            stats.approved += 1;

            // Mover para /filtered/<category>/ (ainda temporário - será deletado após writer)
            move_to_category(&pdf_path, &category, download_dir)?;

            // Registrar no registry como filtered
            if let Err(e) = registry.register_filtered(article_id, score as f64, category.clone()) {
                eprintln!("   ⚠️  Failed to register filtered article: {}", e);
            }

            // Nota: PDF será deletado após writer processar (não deletar aqui ainda)
        } else {
            println!("   Rejected (score: {:.2}): {}", score, result.doc.title);
            stats.rejected += 1;

            // Registrar no registry como rejected ANTES de mover/deletar
            let reason = format!("Score {:.2} below threshold {:.2}", score, FILTER_THRESHOLD);
            if let Err(e) = registry.register_rejected(article_id, score as f64, reason.clone()) {
                eprintln!("   ⚠️  Failed to register rejected article: {}", e);
                // Se falhou porque o artigo não existe, já foi tratado acima
            }

            // Verificar se o arquivo ainda existe antes de tentar mover
            if !pdf_path.exists() {
                println!("   ⚠️  PDF already removed: {}", pdf_path.display());
                continue;
            }

            // Mover para /rejected/ (para debug/logging, mas será deletado)
            let rejected_path = match move_to_rejected(&pdf_path, download_dir) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("   ⚠️  Failed to move rejected PDF: {}", e);
                    // Tentar deletar diretamente do local original se mover falhou
                    if let Err(del_err) = fs::remove_file(&pdf_path) {
                        eprintln!(
                            "   ⚠️  Failed to delete rejected PDF from original location: {}",
                            del_err
                        );
                    } else {
                        println!(
                            "   🗑️  Rejected PDF deleted from original location: {}",
                            pdf_path.display()
                        );
                    }
                    continue;
                }
            };

            // Deletar PDF rejeitado imediatamente do destino (rejected/)
            if let Err(e) = fs::remove_file(&rejected_path) {
                eprintln!(
                    "   ⚠️  Failed to delete rejected PDF from {}: {}",
                    rejected_path.display(),
                    e
                );
            } else {
                println!("   🗑️  Rejected PDF deleted: {}", rejected_path.display());
            }
        }
    }

    Ok(stats)
}

pub(crate) fn discover_unfiltered_pdfs(
    download_dir: &Path,
    registry: &RegistryManager,
) -> Result<Vec<PathBuf>> {
    let mut pdfs = Vec::new();

    if !download_dir.exists() {
        return Ok(pdfs);
    }

    // Buscar PDFs recursivamente de downloads/ (ONLY arxiv/, skip filtered/ e rejected/)
    fn find_pdfs(
        dir: &Path,
        pdfs: &mut Vec<PathBuf>,
        registry: &RegistryManager,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                // Skip filtered, rejected, cache subdirectories
                if dir_name == "filtered"
                    || dir_name == "rejected"
                    || dir_name == "cache"
                    || dir_name == "temp"
                {
                    continue;
                }

                // Recursão para subdiretórios
                find_pdfs(&path, pdfs, registry)?;
            } else if path.extension().is_some_and(|ext| ext == "pdf") {
                // Extrair article_id do caminho
                let article_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Processar se:
                // 1. Nunca processado
                // 2. Status Collected (baixado mas nÃ£o filtrado ainda)
                // 3. Status Rejected mas ainda estÃ¡ em arxiv/ (pode ter sido rejeitado com threshold antigo)
                //    e o PDF ainda existe (nÃ£o foi deletado)
                let metadata = registry.get_metadata(article_id);
                let should_process = match metadata {
                    None => true, // Nunca processado - processar
                    Some(meta) => match meta.status {
                        ArticleStatus::Collected => true, // Baixado mas nÃ£o filtrado ainda
                        ArticleStatus::Rejected => path.exists(), // Reprocessar rejeitados presentes em disco
                        ArticleStatus::Filtered | ArticleStatus::Published => false, // JÃ¡ processado completamente
                    },
                };

                if should_process {
                    pdfs.push(path);
                }
            }
        }
        Ok(())
    }

    // Search in downloads/ (mainly from arxiv/, excluding filtered/, rejected/, cache/)
    find_pdfs(download_dir, &mut pdfs, registry)?;

    Ok(pdfs)
}

pub(crate) fn move_to_category(pdf_path: &Path, category: &str, base_dir: &Path) -> Result<()> {
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

pub(crate) fn move_to_rejected(pdf_path: &Path, base_dir: &Path) -> Result<PathBuf> {
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

    Ok(dest_path)
}
