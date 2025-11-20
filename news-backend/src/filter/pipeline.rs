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
    
    // Contadores para logs colapsados
    let mut extraction_failures = 0;
    let mut parse_errors = 0;
    let mut non_scientific = 0;
    let mut rejected_count = 0;

    for pdf_path in pdfs {
        // CRITICAL: Log each PDF being processed
        let article_id = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
        println!("   🔍 Processing PDF: {} ({})", article_id, pdf_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"));
        
        // Parse do PDF
        let parsed = match crate::filter::parser::parse_pdf(&pdf_path) {
            Ok(p) => {
                // Verificar se extração de texto falhou (texto vazio)
                if p.text.is_empty() || p.text.len() < 100 {
                    println!("   ⚠️  PDF {}: Text extraction failed or very short ({} chars)", article_id, p.text.len());
                    extraction_failures += 1;
                } else {
                    println!("   ✅ PDF {}: Text extracted successfully ({} chars)", article_id, p.text.len());
                }
                p
            },
            Err(e) => {
                println!("   ❌ PDF {}: Parse error - {}", article_id, e);
                parse_errors += 1;
                stats.rejected += 1;
                continue;
            }
        };

        // Detectar tipo de fonte
        let source_type = detect_source_type(&parsed);

        if source_type == SourceType::NonScientific {
            println!("   ⏭️  PDF {}: Non-scientific source, skipping", article_id);
            non_scientific += 1;
            stats.skipped += 1;
            continue;
        }

        // Se extração de texto falhou, tentar buscar metadados do arXiv como fallback
        let mut parsed_with_text = parsed;
        let used_fallback = parsed_with_text.text.is_empty() || parsed_with_text.text.len() < 100;
        if used_fallback {
            // Tentar buscar abstract do arXiv via API
            println!("   🔄 PDF {}: Attempting arXiv abstract fallback...", article_id);
            if let Some(arxiv_id) = pdf_path.file_stem().and_then(|s| s.to_str()) {
                match fetch_arxiv_abstract(arxiv_id).await {
                    Ok(abstract_text) if !abstract_text.is_empty() => {
                        println!("   ✅ PDF {}: arXiv abstract retrieved ({} chars)", article_id, abstract_text.len());
                        // Atualizar o texto do parsed usando uma nova instância
                        parsed_with_text.text = abstract_text;
                    }
                    Ok(_) => {
                        println!("   ⚠️  PDF {}: arXiv abstract is empty", article_id);
                    }
                    Err(e) => {
                        println!("   ⚠️  PDF {}: Failed to fetch arXiv abstract: {}", article_id, e);
                    }
                }
            } else {
                println!("   ⚠️  PDF {}: Cannot extract arxiv_id from filename", article_id);
            }
        }

        // Se ainda não tem texto suficiente, pular mas não rejeitar ainda
        // (pode ser um PDF válido que precisa de melhor extração)
        if parsed_with_text.text.is_empty() || parsed_with_text.text.len() < 50 {
            println!("   ⏭️  PDF {}: Insufficient text after fallback ({} chars), skipping", article_id, parsed_with_text.text.len());
            extraction_failures += 1;
            stats.skipped += 1;
            continue;
        }

        // Filtros rápidos
        let has_tests = has_experimental_sections(&parsed_with_text) || parsed_with_text.text.len() > 500; // Se tem texto suficiente, assumir que pode ter testes
        let fake_penalty = calculate_fake_penalty(&parsed_with_text.text);

        println!("   🔍 PDF {}: Filter checks - has_tests={}, fake_penalty={:.2}", article_id, has_tests, fake_penalty);

        if !has_tests || fake_penalty > 0.5 {
            println!("   ❌ PDF {}: REJECTED - has_tests={}, fake_penalty={:.2}", article_id, has_tests, fake_penalty);
            rejected_count += 1;
            stats.rejected += 1;
            continue;
        }
        
        println!("   ✅ PDF {}: Passed initial filters, proceeding to validation...", article_id);

        // Validação via APIs
        println!("   🔍 PDF {}: Validating via APIs (DOI, authors)...", article_id);
        let doi_ratio = validate_dois(&parsed_with_text.dois).await;
        let author_ratio = validate_authors(&parsed_with_text.authors).await;
        println!("   📊 PDF {}: Validation results - DOI ratio: {:.2}, Author ratio: {:.2}", article_id, doi_ratio, author_ratio);

        let result = FilterResult {
            doc: parsed_with_text,
            doi_ratio,
            author_ratio,
            has_exp: has_tests, // Usar has_tests para has_exp
            fake_penalty,
        };

        let score = calculate_score(&result);
        println!("   📊 PDF {}: Calculated filter score: {:.2} (threshold: {:.2})", article_id, score, FILTER_THRESHOLD);

        // Article ID já extraído acima no início do loop

        // Verificar se o artigo existe no registry antes de tentar atualizar
        // Se não existir, criar uma entrada básica primeiro
        if !registry.is_article_registered(article_id) {
            println!("   📋 PDF {}: Not in registry, creating entry...", article_id);
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
                println!("   ❌ PDF {}: Failed to create registry entry: {}", article_id, e);
                eprintln!(
                    "   ⚠️  Failed to create registry entry for article {}: {}",
                    article_id, e
                );
                stats.rejected += 1;
                continue;
            } else {
                println!("   ✅ PDF {}: Registry entry created", article_id);
            }
        } else {
            println!("   ✅ PDF {}: Already in registry", article_id);
        }

        // Arredondar score para 2 casas decimais para evitar problemas de precisão float
        // Se arredondado for >= 0.40, aprovar
        let rounded_score = (score * 100.0).round() / 100.0;

        // Threshold: aprovar se score arredondado >= 0.4
        if rounded_score >= FILTER_THRESHOLD {
            let category = categorize(&result.doc);
            println!(
                "   ✅ PDF {}: APPROVED (score: {:.2} >= {:.2}) - {} → {}",
                article_id, rounded_score, FILTER_THRESHOLD, result.doc.title, category
            );
            stats.approved += 1;

            // Mover para /filtered/<category>/ (ainda temporário - será deletado após writer)
            println!("   📁 PDF {}: Moving to filtered/{}...", article_id, category);
            move_to_category(&pdf_path, &category, download_dir)?;
            println!("   ✅ PDF {}: Moved to filtered/{}/", article_id, category);

            // Registrar no registry como filtered
            if let Err(e) = registry.register_filtered(article_id, score as f64, category.clone()) {
                eprintln!("   ⚠️  PDF {}: Failed to register filtered article: {}", article_id, e);
            } else {
                println!("   ✅ PDF {}: Registered as filtered in registry", article_id);
            }

            // Nota: PDF será deletado após writer processar (não deletar aqui ainda)
        } else {
            println!("   ❌ PDF {}: REJECTED (score: {:.2} < {:.2}) - {}", article_id, rounded_score, FILTER_THRESHOLD, result.doc.title);
            stats.rejected += 1;

            // Registrar no registry como rejected ANTES de mover/deletar
            let reason = format!("Score {:.2} below threshold {:.2}", score, FILTER_THRESHOLD);
            if let Err(e) = registry.register_rejected(article_id, score as f64, reason.clone()) {
                println!("   ⚠️  PDF {}: Failed to register rejected article: {}", article_id, e);
                eprintln!("   ⚠️  Failed to register rejected article: {}", e);
                // Se falhou porque o artigo não existe, já foi tratado acima
            } else {
                println!("   ✅ PDF {}: Registered as rejected in registry", article_id);
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
    
    // Log resumido ao invés de linha por linha
    if extraction_failures > 0 || parse_errors > 0 || rejected_count > 0 || non_scientific > 0 {
        println!("\n📊 Filter summary: {} approved, {} rejected ({} no tests/fake), {} extraction failures, {} parse errors, {} non-scientific", 
            stats.approved, stats.rejected, rejected_count, extraction_failures, parse_errors, non_scientific);
    }

    Ok(stats)
}

// Função auxiliar para buscar abstract do arXiv quando extração de PDF falha
async fn fetch_arxiv_abstract(arxiv_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    use reqwest;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    
    let url = format!("https://export.arxiv.org/api/query?id_list={}", arxiv_id);
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        return Err("HTTP error".into());
    }
    
    let xml = response.text().await?;
    
    // Extrair abstract do XML
    if let Some(abstract_start) = xml.find("<summary>") {
        if let Some(abstract_end) = xml[abstract_start..].find("</summary>") {
            let abstract_text = &xml[abstract_start + 9..abstract_start + abstract_end];
            return Ok(abstract_text.trim().to_string());
        }
    }
    
    Err("No abstract found".into())
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
                // 2. Status Collected (baixado mas não filtrado ainda)
                let metadata = registry.get_metadata(article_id);
                let should_process = match metadata {
                    None => true, // Nunca processado - processar
                    Some(meta) => match meta.status {
                        ArticleStatus::Collected => true, // Baixado mas não filtrado ainda
                        ArticleStatus::Rejected
                        | ArticleStatus::Filtered
                        | ArticleStatus::Published => false, // Já processado - não reprocessar
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
