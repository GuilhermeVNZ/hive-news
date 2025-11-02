use axum::{
    Router,
    extract::Extension,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

mod collectors;
mod config;
mod db;
mod filter;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
mod writer;

use db::connection::Database;
use std::path::Path;
use crate::utils::article_registry::RegistryManager;

#[allow(dead_code)]
fn file_already_downloaded(paper_id: &str, base_dir: &Path) -> bool {
    let filename = format!("{}.pdf", paper_id);

    // 1. Verificar em downloads/arxiv/ (todas as subpastas de data)
    let arxiv_dir = base_dir.join("arxiv");
    if arxiv_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&arxiv_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let date_dir = entry.path();
                    if date_dir.is_dir() {
                        let file_path = date_dir.join(&filename);
                        if file_path.exists() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 2. Verificar em downloads/filtered/<categoria>/
    let filtered_dir = base_dir.join("filtered");
    if filtered_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&filtered_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let category_dir = entry.path();
                    if category_dir.is_dir() {
                        let file_path = category_dir.join(&filename);
                        if file_path.exists() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 3. Verificar em downloads/rejected/
    let rejected_dir = base_dir.join("rejected");
    if rejected_dir.exists() {
        let file_path = rejected_dir.join(&filename);
        if file_path.exists() {
            return true;
        }
    }

    false
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    // Try multiple locations for .env file
    let env_paths = vec![
        std::path::PathBuf::from(".env"),
        std::path::PathBuf::from("news-backend/.env"),
        std::path::PathBuf::from("G:/Hive-Hub/News-main/news-backend/.env"),
    ];
    
    for path in &env_paths {
        if path.exists() {
            if let Err(e) = dotenv::from_path(path) {
                eprintln!("Warning: Failed to load .env from {}: {}", path.display(), e);
            } else {
                eprintln!("[OK] Loaded .env from {}", path.display());
            }
            break;
        }
    }
    
    // Fallback: try default location
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check if we should run a collection test, filter, writer, migration, or enrichment
    let args: Vec<String> = std::env::args().collect();
    let test_collector = args.len() > 1 && args[1] == "collect";
    let collect_pmc = args.len() > 1 && args[1] == "collect-pmc";
    let collect_ss = args.len() > 1 && args[1] == "collect-ss";
    let collect_enabled = args.len() > 1 && args[1] == "collect-enabled";
    let test_rss = args.len() > 1 && args[1] == "test-rss";
    let test_html = args.len() > 1 && args[1] == "test-html";
    let test_filter = args.len() > 1 && args[1] == "filter";
    let test_writer = args.len() > 1 && args[1] == "write";
    let write_news = args.len() > 1 && args[1] == "write-news";
    let cleanup_news = args.len() > 1 && args[1] == "cleanup-news";
    let run_pipeline = args.len() > 1 && args[1] == "pipeline";
    let run_pipeline_debug = args.len() > 1 && args[1] == "pipeline-debug";
    let migrate_registry = args.len() > 1 && args[1] == "migrate-registry";
    let enrich_registry = args.len() > 1 && args[1] == "enrich-registry";

    if test_collector {
        println!("🔬 Test Collector - arXiv");
        println!("=====================================\n");

        println!("📥 Starting collection from arXiv...");
        println!("   Category: cs.AI");
        println!("   Papers: 10\n");

        // Coleta direta SEM banco de dados
        run_arxiv_collection_direct().await?;

        return Ok(());
    }

    if collect_pmc {
        println!("🔬 Test Collector - PubMed Central (PMC)");
        println!("=====================================\n");
        run_pmc_collection_direct().await?;
        return Ok(());
    }

    if collect_ss {
        println!("🔬 Test Collector - Semantic Scholar");
        println!("=====================================\n");
        run_semantic_scholar_collection_direct().await?;
        return Ok(());
    }

    if collect_enabled {
        println!("🔬 Collector - Enabled Sources from system_config.json");
        println!("====================================================\n");
        run_collect_enabled_from_config().await?;
        return Ok(());
    }

    if test_rss {
        println!("📡 Test RSS Collector");
        println!("=====================================\n");
        run_rss_collector_test().await?;
        return Ok(());
    }

    if test_html {
        println!("🌐 Test HTML Collector");
        println!("=====================================\n");
        run_html_collector_test().await?;
        return Ok(());
    }

    if test_filter {
        println!("🔬 Scientific Filter - PDF Validation");
        println!("=====================================\n");
        
        // Run filter pipeline
        let filter_result = filter::pipeline::run_filter_pipeline(Path::new("G:/Hive-Hub/News-main/downloads")).await?;
        
        println!("\n✅ Filter completed!");
        println!("   Approved: {}", filter_result.approved);
        println!("   Rejected: {}", filter_result.rejected);
        println!("   Skipped (non-scientific): {}", filter_result.skipped);
        println!("   Total processed: {}", filter_result.total);
        
        return Ok(());
    }

    if cleanup_news {
        println!("🧹 News Cleanup - Verify and Clean Processed Articles");
        println!("====================================================\n");
        run_cleanup_news().await?;
        return Ok(());
    }

    if write_news {
        println!("📰 News Writer - Generate Articles from Collected News");
        println!("====================================================\n");
        run_news_writer().await?;
        return Ok(());
    }

    if run_pipeline {
        println!("🔄 News Pipeline - Complete Processing Flow");
        println!("==========================================\n");
        run_news_pipeline().await?;
        return Ok(());
    }

    if run_pipeline_debug {
        println!("🔍 News Pipeline - DEBUG MODE (Ultra-Detailed Logging)");
        println!("====================================================\n");
        run_news_pipeline_debug().await?;
        return Ok(());
    }

    if test_writer {
        println!("✍️  DeepSeek Writer - Content Generation");
        println!("=====================================\n");
        println!("   Style: Nature/Science magazine editorial");
        
        // Run writer pipeline
        run_writer_pipeline().await?;
        
        return Ok(());
    }

    if migrate_registry {
        println!("🔄 Article Registry Migration");
        println!("=====================================\n");
        
        run_registry_migration()?;
        
        return Ok(());
    }

    if enrich_registry {
        println!("🔍 Registry Enrichment");
        println!("=====================================\n");
        
        run_registry_enrichment().await?;
        
        return Ok(());
    }

    // Função para coleta direta do arXiv sem banco
    async fn run_arxiv_collection_direct() -> anyhow::Result<()> {
        use crate::collectors::arxiv_collector::ArxivCollector;
        use crate::models::raw_document::ArticleMetadata;
        use std::path::Path;

        // Inicializar registry
        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;
        
        // Debug: verificar quantos artigos foram carregados
        let total_articles = registry.get_all_articles().len();
        println!("📋 Registry loaded: {} articles in total", total_articles);
        if total_articles > 0 {
            let sample_ids: Vec<String> = registry.get_all_articles()
                .iter()
                .take(5)
                .map(|a| a.id.clone())
                .collect();
            println!("   Sample IDs: {:?}", sample_ids);
        }

        // Criar temp dir e download dir
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let temp_dir = base_dir.join("temp");
        let temp_dir_clone = temp_dir.clone();
        let _arxiv_collector = ArxivCollector::new(temp_dir_clone);

        // Download dos PDFs
        let download_dir = base_dir.join("arxiv");
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = download_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        // Definir target_count antes de usar nos prints
        let target_count = 10;
        let mut start_offset = 0;
        let mut downloaded_count = 0;

        println!("⬇️  Downloading PDFs from arXiv...");
        println!("   📂 Target directory: {}", date_dir.display());
        println!("   🎯 Target count: {} new papers", target_count);
        println!("   📊 Registry: {} articles already registered", total_articles);
        println!("");

        // Cliente com cookies para manter sessão e evitar reCAPTCHA
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .cookie_store(true)  // IMPORTANTE: Salvar cookies entre requisições
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        // Fazer uma requisição inicial ao arXiv para estabelecer sessão e obter cookies
        println!("🔐 Establishing session with arXiv...");
        match client.get("https://arxiv.org/list/cs.AI/recent").send().await {
            Ok(_) => println!("   Session established ✓"),
            Err(e) => println!("   Warning: Could not establish session: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Safe guards para evitar ban da API
        let max_api_requests = 50; // Máximo de requisições por ciclo
        let max_consecutive_empty = 5; // Máximo de batches vazios consecutivos antes de parar (aumentado para permitir gaps)
        let mut api_request_count = 0;
        let mut consecutive_empty_batches = 0;
        let mut _last_successful_offset = None;
        let mut batches_with_articles_seen = 0; // Contador de batches com artigos encontrados

        // Loop até baixar 10 novos artigos (busca regressiva até encontrar)
        while downloaded_count < target_count {
            // Safe guard: limite de requisições por ciclo
            if api_request_count >= max_api_requests {
                println!("⚠️  Reached maximum API requests limit ({}), stopping search", max_api_requests);
                println!("   Found {} new papers (target was {})", downloaded_count, target_count);
                break;
            }
            
            println!("📡 [BATCH] Fetching articles from arXiv API...");
            println!("   📊 Offset: {}", start_offset);
            println!("   🔢 Batch size: {}", target_count * 2);
            println!("   📈 Progress: {} new papers downloaded (target: {})", downloaded_count, target_count);
            println!("   🔄 API requests: {}/{}", api_request_count + 1, max_api_requests);

            // URL correta do arXiv API (não use DeepSeek aqui!)
            // Usar submittedDate antiga para evitar reCAPTCHA
            let url = format!(
                "https://export.arxiv.org/api/query?search_query=cat:cs.AI+AND+submittedDate:[20240101*+TO+{}]&start={}&max_results={}&sortBy=submittedDate&sortOrder=descending",
                chrono::Utc::now().format("%Y%m%d"),
                start_offset,
                target_count * 2 // Buscar mais para garantir que achamos novos
            );
            
            println!("  URL: {}", url);

            // Safe guard: delay entre requisições de batch (evitar rate limiting)
            if api_request_count > 0 {
                let delay_seconds = if consecutive_empty_batches > 0 {
                    // Backoff exponencial se encontrar batches vazios
                    2.0 + (consecutive_empty_batches as f64 * 0.5)
                } else {
                    1.0 // Delay normal de 1 segundo
                };
                println!("  ⏳ Waiting {:.1}s before API request (safe guard)...", delay_seconds);
                tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay_seconds)).await;
            }

            api_request_count += 1;
            
            let response = client.get(&url).send().await?;
            let xml = response.text().await?;
            
            // Debug: verificar se recebemos dados válidos
            println!("  Response length: {} bytes", xml.len());
            if xml.len() < 100 {
                println!("  ⚠️  Warning: Very short response, might be an error page");
            }

            // Parse básico do XML com extração de título (suporta títulos multilinha)
            let mut current_id = None;
            let mut current_title = None;
            let mut collecting_title = false;
            let mut title_parts = Vec::new();
            let mut articles = Vec::new();

            for line in xml.lines() {
                // Extrair ID do artigo
                if line.contains("<id>") {
                    if let Some(start) = line.find("<id>") {
                        if let Some(end) = line.find("</id>") {
                            let id = &line[start + 4..end];
                            if id.contains("arxiv.org/abs/") {
                                let mut paper_id = id
                                    .replace("http://arxiv.org/abs/", "")
                                    .replace("https://arxiv.org/abs/", "");
                                // Remove version suffix (v1, v2, etc.) to get published version
                                // Verificar se termina com "v" seguido de dígitos antes de remover
                                if let Some(pos) = paper_id.rfind('v') {
                                    // Verificar se após 'v' há apenas dígitos até o fim da string
                                    if pos + 1 < paper_id.len() {
                                        let after_v = &paper_id[pos + 1..];
                                        if after_v.chars().all(|c| c.is_ascii_digit()) {
                                            paper_id = paper_id[..pos].to_string();
                                        }
                                    }
                                }
                                current_id = Some(paper_id.clone());
                                // Debug: mostrar ID extraído
                                if api_request_count <= 2 {
                                    println!("  [DEBUG] Extracted ID from XML: {} -> {}", id.trim(), paper_id);
                                }
                            }
                        }
                    }
                }
                
                // Extrair título (suporta títulos multilinha)
                if line.contains("<title>") {
                    collecting_title = true;
                    title_parts.clear();
                    
                    // Caso 1: título completo na mesma linha
                    if let Some(start) = line.find("<title>") {
                        if let Some(end) = line.find("</title>") {
                            let title = line[start + 7..end].trim().to_string();
                            if !title.is_empty() {
                                current_title = Some(title);
                                collecting_title = false;
                            }
                        } else {
                            // Caso 2: título começa na linha mas continua em outras
                            if let Some(start) = line.find("<title>") {
                                let title_part = line[start + 7..].trim().to_string();
                                if !title_part.is_empty() {
                                    title_parts.push(title_part);
                                }
                            }
                        }
                    }
                } else if collecting_title {
                    // Continuar coletando título até encontrar </title>
                    if let Some(end_pos) = line.find("</title>") {
                        // Fim do título encontrado
                        let title_part = line[..end_pos].trim().to_string();
                        if !title_part.is_empty() {
                            title_parts.push(title_part);
                        }
                        // Combinar todas as partes do título
                        let full_title = title_parts.join(" ").trim().to_string();
                        if !full_title.is_empty() {
                            current_title = Some(full_title);
                        }
                        collecting_title = false;
                        title_parts.clear();
                    } else {
                        // Continuar coletando título
                        let title_part = line.trim().to_string();
                        if !title_part.is_empty() {
                            title_parts.push(title_part);
                        }
                    }
                }
                
                // Quando encontrar </entry>, finalizar artigo
                if line.contains("</entry>") {
                    if let Some(id) = current_id.take() {
                        // Se ainda estava coletando título mas não encontrou </title>, usar o que coletou
                        if collecting_title && !title_parts.is_empty() {
                            let full_title = title_parts.join(" ").trim().to_string();
                            if !full_title.is_empty() {
                                current_title = Some(full_title);
                            }
                            collecting_title = false;
                            title_parts.clear();
                        }
                        
                        let title = current_title.take().unwrap_or_else(|| {
                            eprintln!("  ⚠️  WARNING: Article {} has no title - using 'Untitled'", id);
                            "Untitled".to_string()
                        });
                        
                        articles.push(ArticleMetadata {
                            id: id.clone(),
                            title: title.clone(),
                            url: format!("https://arxiv.org/abs/{}", id),
                            author: Some("Unknown".to_string()),
                            summary: Some("No summary available".to_string()),
                            published_date: Some(chrono::Utc::now()),
                            image_url: None,
                            source_type: Some("arxiv".to_string()),
                            content_html: None,
                            content_text: None,
                            category: None,
                        });
                    }
                    // Reset estado para próximo artigo (se ainda estava coletando)
                    if collecting_title {
                        collecting_title = false;
                        title_parts.clear();
                    }
                }
            }

            println!("  ✅ Batch fetched successfully");
            println!("  📄 Found {} papers in this batch", articles.len());
            if !articles.is_empty() {
                println!("  📋 First article: {} - {}", articles[0].id, articles[0].title.chars().take(60).collect::<String>());
            }
            
            // Safe guard: verificar se batch está vazio
            if articles.is_empty() {
                consecutive_empty_batches += 1;
                println!("  ⚠️  Empty batch encountered (consecutive: {}/{})", consecutive_empty_batches, max_consecutive_empty);
                
                // Se muitos batches vazios consecutivos, pode ser que chegamos ao fim dos resultados recentes
                // Mas só parar se já encontramos alguns artigos novos OU se já vimos muitos batches com artigos
                if consecutive_empty_batches >= max_consecutive_empty {
                    // Se ainda não encontramos nenhum artigo novo, continuar buscando mesmo com gaps
                    // (pode ser gaps temporários na API, não o fim dos resultados)
                    if downloaded_count == 0 && batches_with_articles_seen > 0 {
                        // Reset contador se ainda não encontramos nenhum novo - pode ser gaps na API
                        println!("  ℹ️  Reset empty batch count (continuing search - may be API gaps, seen {} batches with articles)", batches_with_articles_seen);
                        consecutive_empty_batches = 0;
                        // Continuar para próximo batch
                        let batch_size = target_count * 2;
                        start_offset += batch_size;
                        continue;
                    } else if downloaded_count > 0 {
                        // Já encontramos alguns novos, então batches vazios provavelmente indicam fim
                        println!("  ⚠️  Too many consecutive empty batches ({}), stopping (found {} new articles)", consecutive_empty_batches, downloaded_count);
                        break;
                    } else if api_request_count >= max_api_requests {
                        println!("  ⚠️  Reached max API requests limit, stopping");
                        break;
                    } else {
                        // Continuar buscando
                        println!("  ℹ️  Continuing search (found 0 new so far, {} requests made)", api_request_count);
                        consecutive_empty_batches = 0;
                        let batch_size = target_count * 2;
                        start_offset += batch_size;
                        continue;
                    }
                }
                
                // Continuar para próximo batch (mas com backoff)
                let batch_size = target_count * 2;
                start_offset += batch_size;
                continue;
            } else {
                // Reset contador se encontramos resultados
                consecutive_empty_batches = 0;
                _last_successful_offset = Some(start_offset);
                batches_with_articles_seen += 1;
            }

            // Tentar baixar artigos não duplicados
            let mut found_new_in_batch = false;
            for article in articles.iter() {
                if downloaded_count >= target_count {
                    break;
                }

                let file_path = date_dir.join(format!("{}.pdf", article.id));

                // Verificar se já foi processado usando registry
                let is_registered = registry.is_article_registered(&article.id);
                if is_registered {
                    // Mostrar sempre detalhes quando encontra duplicado
                    println!(
                        "  [{}/{}]: ⏭️  SKIPPED (duplicate): {}",
                        downloaded_count + 1,
                        target_count,
                        article.id
                    );
                    println!("      📄 Title: {}", article.title);
                    let metadata = registry.get_metadata(&article.id);
                    if let Some(meta) = metadata {
                        println!("      📊 Status: {:?}", meta.status);
                        if let Some(output_dir) = &meta.output_dir {
                            println!("      📁 Output dir: {}", output_dir.display());
                        }
                    }
                    println!("      ℹ️  Article already in registry, skipping download");
                    continue;
                }
                
                found_new_in_batch = true;

                // Baixar (use published ID without version suffix)
                // Usar a API REST oficial do arXiv para baixar PDFs
                let pdf_url = format!("https://export.arxiv.org/pdf/{}.pdf", article.id);
                let arxiv_url = format!("https://arxiv.org/abs/{}", article.id);
                println!(
                    "  [{}/{}]: 📥 DOWNLOADING: {}",
                    downloaded_count + 1,
                    target_count,
                    article.id
                );
                println!("      📄 Title: {}", article.title);
                println!("      🔗 URL: {}", arxiv_url);
                println!("      ⬇️  PDF URL: {}", pdf_url);
                print!("      ⏳ Downloading... ");

                // Criar requisição com headers para evitar reCAPTCHA
                let request = client
                    .get(&pdf_url)
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .header("Accept", "application/pdf,text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                    .header("Accept-Language", "en-US,en;q=0.9")
                    .header("Accept-Encoding", "gzip, deflate, br")
                    .header("DNT", "1")
                    .header("Connection", "keep-alive")
                    .header("Upgrade-Insecure-Requests", "1")
                    .header("Sec-Fetch-Dest", "document")
                    .header("Sec-Fetch-Mode", "navigate")
                    .header("Sec-Fetch-Site", "none")
                    .header("Cache-Control", "max-age=0");

                let download_start = std::time::Instant::now();
                match request.send().await {
                    Ok(response) => {
                        // Verificar se é uma resposta de sucesso
                        if response.status().is_success() {
                            // Verificar Content-Length se disponível
                            if let Some(content_length) = response.headers().get("content-length") {
                                if let Ok(len_str) = content_length.to_str() {
                                    if let Ok(len_bytes) = len_str.parse::<u64>() {
                                        let len_mb = len_bytes as f64 / 1_048_576.0;
                                        println!("({:.2} MB)", len_mb);
                                    }
                                }
                            }
                            let bytes = response.bytes().await;
                            match bytes {
                                Ok(b) => {
                                    let file_size_mb = b.len() as f64 / 1_048_576.0;
                                    // Verify it's actually a PDF (starts with %PDF)
                                    if b.len() > 4 && &b[0..4] == b"%PDF" {
                                        match tokio::fs::write(&file_path, &b).await {
                                            Ok(_) => {
                                                let download_duration = download_start.elapsed();
                                                println!("      ✅ Downloaded successfully!");
                                                println!("      📦 File size: {:.2} MB", file_size_mb);
                                                println!("      📁 Saved to: {}", file_path.display());
                                                println!("      ⏱️  Download time: {:.2}s", download_duration.as_secs_f64());
                                                print!("      📝 Registering in registry... ");
                                                // Registrar no registry após download bem-sucedido
                                                if let Err(e) = registry.register_collected(
                                                    article.id.clone(),
                                                    article.title.clone(),
                                                    arxiv_url.clone(),
                                                    pdf_url.clone(),
                                                ) {
                                                    eprintln!("⚠️  Failed: {}", e);
                                                } else {
                                                    // Define destinos com base nos sites que têm arXiv habilitado
                                                    let destinations = get_enabled_sites_for_source("arxiv");
                                                    if let Err(e) = registry.set_destinations(&article.id, destinations) {
                                                        eprintln!("⚠️  Failed to set destinations: {}", e);
                                                    }
                                                    downloaded_count += 1;
                                                    println!("✅ Registered");
                                                    println!("      ✅ Article {} registered successfully!", article.id);
                                                }
                                            }
                                            Err(e) => {
                                                println!("❌ Failed to write file: {}", e);
                                                println!("      💥 Error details: {:?}", e);
                                            },
                                        }
                                    } else {
                                        println!("❌ Invalid PDF format (got HTML or redirect)");
                                        println!("      💥 Response size: {} bytes", b.len());
                                        println!("      💥 First bytes: {:?}", &b[..std::cmp::min(100, b.len())]);
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to read response bytes: {}", e);
                                    println!("      💥 Error details: {:?}", e);
                                },
                            }
                        } else {
                            println!("❌ HTTP Error: {} {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"));
                            if let Ok(status_text) = response.text().await {
                                if !status_text.is_empty() {
                                    let preview = status_text.chars().take(200).collect::<String>();
                                    println!("      💥 Response preview: {}", preview);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Request failed: {}", e);
                        println!("      💥 Error details: {:?}", e);
                    },
                }
                
                // Delay entre downloads para evitar rate limiting
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }

            // Se não baixou nenhum novo neste batch, incrementar offset para próximo batch
            if downloaded_count < target_count && !found_new_in_batch {
                // Incrementar pelo número de artigos buscados no batch anterior
                let batch_size = target_count * 2; // max_results = 20
                start_offset += batch_size;
                println!("  No new papers in this batch, trying older articles (offset {})...\n", start_offset);
            } else if found_new_in_batch {
                // Se encontrou novos neste batch, resetar contador de vazios
                consecutive_empty_batches = 0;
            }

            // Safety: não ficar em loop infinito (permitir buscar até 1000 artigos no histórico)
            // Se já tentou muito longe e ainda não achou 10 novos, provavelmente já processou tudo recente
            if start_offset > 1000 {
                println!("⚠️  Reached max offset (1000), stopping search");
                println!("   Found {} new papers so far (target was {})", downloaded_count, target_count);
                break;
            }
        }

        println!("\n✅ Collection completed!");
        println!("   New papers downloaded: {}/10", downloaded_count);
        println!("   Location: {}", date_dir.display());

        // Limpar arquivos temporários
        println!("\n🧹 Cleaning temporary files...");
        cleanup_temp_files(&temp_dir).await?;

        // Filtro científico (processa todos os PDFs não filtrados)
        println!("\n🔬 Starting Scientific Filter...");
        println!("   (Blogs and non-scientific sources will be skipped)");

        let filter_result =
            filter::pipeline::run_filter_pipeline(Path::new("G:/Hive-Hub/News-main/downloads"))
                .await?;

        println!("\n✅ Filter completed!");
        println!("   Approved: {}", filter_result.approved);
        println!("   Rejected: {}", filter_result.rejected);
        println!("   Skipped (non-scientific): {}", filter_result.skipped);
        println!("   Total processed: {}", filter_result.total);

        // Final step: ensure destinations are set for this cycle (arxiv)
        println!("\n📌 Finalizing destinations for arXiv cycle...");
        ensure_destinations_for_cycle("arxiv", &date_dir, &registry);

        Ok(())
    }

    async fn ensure_dir(path: &std::path::Path) -> anyhow::Result<()> {
        if !path.exists() { tokio::fs::create_dir_all(path).await?; }
        Ok(())
    }

    async fn run_pmc_collection_direct() -> anyhow::Result<()> {
        use crate::collectors::pmc_collector::PmcCollector;
        use crate::models::raw_document::ArticleMetadata;
        use std::path::Path;

        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let pmc_dir = base_dir.join("pmc");
        ensure_dir(&pmc_dir).await?;
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = pmc_dir.join(&date);
        ensure_dir(&date_dir).await?;

        let temp_dir = base_dir.join("temp_pmc");
        ensure_dir(&temp_dir).await?;
        let collector = PmcCollector::new(temp_dir);

        // determine destinations (sites) that have PMC enabled
        let destinations = get_enabled_sites_for_source("pmc");

        // loop por páginas até achar novos (batch=20)
        let mut retstart: usize = 0;
        let mut new_found = 0usize;
        for page_idx in 0..5 {
            println!("PMC page {} (retstart={}): fetching up to 20...", page_idx + 1, retstart);
            let articles: Vec<ArticleMetadata> = collector.fetch_recent_papers(20, retstart).await?;
            println!("PMC page {}: fetched {} candidates", page_idx + 1, articles.len());
            if articles.is_empty() { println!("PMC: no candidates on this page"); break; }
            let mut existing = 0usize;
            let mut downloaded = 0usize;
            for a in articles.iter() {
            let id = a.id.trim().to_string();
                if registry.is_article_registered(&id) { existing += 1; continue; }
            let pdf_path = date_dir.join(format!("{}.pdf", id));
            if let Err(e) = collector.download_pdf(&id, &pdf_path).await {
                    eprintln!("PMC download failed for {}: {}", id, e);
                continue;
            }
            let arxiv_like_url = a.url.clone();
            let pdf_url = format!("{}", pdf_path.display());
            if let Err(e) = registry.register_collected(id.clone(), a.title.clone(), arxiv_like_url, pdf_url) {
                eprintln!("Registry error for {}: {}", id, e);
            }
            if let Err(e) = registry.set_destinations(&id, destinations.clone()) { eprintln!("Destinations write error for {}: {}", id, e); }
            // small delay to avoid bursts
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                new_found += 1;
                downloaded += 1;
            }
            println!("PMC page {} summary: existing={}, downloaded_new={}", page_idx + 1, existing, downloaded);
            if new_found > 0 { break; }
            retstart += 20;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        println!("✅ PMC collection done → {}", date_dir.display());
        // Final step: ensure destinations are set for this cycle (pmc)
        println!("\n📌 Finalizing destinations for PMC cycle...");
        ensure_destinations_for_cycle("pmc", &date_dir, &registry);
        Ok(())
    }

    async fn run_semantic_scholar_collection_direct() -> anyhow::Result<()> {
        use crate::collectors::semantic_scholar_collector::SemanticScholarCollector;
        use crate::models::raw_document::ArticleMetadata;
        use std::path::Path;

        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let ss_dir = base_dir.join("semantic_scholar");
        ensure_dir(&ss_dir).await?;
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = ss_dir.join(&date);
        ensure_dir(&date_dir).await?;

        let temp_dir = base_dir.join("temp_ss");
        ensure_dir(&temp_dir).await?;
        let api_key = std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok();
        let collector = SemanticScholarCollector::new(temp_dir, api_key);

        // determine destinations (sites) that have Semantic Scholar enabled
        let destinations = get_enabled_sites_for_source("semantic");

        // loop com offset para achar novos (batch=20)
        let mut offset: usize = 0;
        let mut new_found = 0usize;
        for page_idx in 0..5 {
            println!("Semantic Scholar page {} (offset={}): fetching up to 20...", page_idx + 1, offset);
            let articles: Vec<ArticleMetadata> = collector.fetch_recent_papers(20, offset, None).await?;
            println!("Semantic Scholar page {}: fetched {} candidates", page_idx + 1, articles.len());
            if articles.is_empty() { println!("Semantic Scholar: no candidates on this page"); break; }
            let mut existing = 0usize;
            let mut downloaded = 0usize;
            for a in articles.iter() {
            let id = a.id.trim().to_string();
                if registry.is_article_registered(&id) { existing += 1; continue; }
            let pdf_path = date_dir.join(format!("{}.pdf", id));
            if let Err(e) = collector.download_pdf(&id, &pdf_path).await {
                eprintln!("Semantic Scholar download failed for {}: {}", id, e);
                continue;
            }
            let url = a.url.clone();
            let pdf_url = format!("{}", pdf_path.display());
            if let Err(e) = registry.register_collected(id.clone(), a.title.clone(), url, pdf_url) {
                eprintln!("Registry error for {}: {}", id, e);
            }
            if let Err(e) = registry.set_destinations(&id, destinations.clone()) { eprintln!("Destinations write error for {}: {}", id, e); }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                new_found += 1;
                downloaded += 1;
            }
            println!("Semantic Scholar page {} summary: existing={}, downloaded_new={}", page_idx + 1, existing, downloaded);
            if new_found > 0 { break; }
            offset += 20;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        println!("✅ Semantic Scholar collection done → {}", date_dir.display());
        // Final step: ensure destinations are set for this cycle (semantic)
        println!("\n📌 Finalizing destinations for Semantic Scholar cycle...");
        ensure_destinations_for_cycle("semantic", &date_dir, &registry);
        Ok(())
    }

    async fn run_collect_enabled_from_config() -> anyhow::Result<()> {
        use std::path::Path;
        use crate::utils::config_manager::ConfigManager;
        use crate::utils::site_config_manager::SiteConfigManager;

        // Primeiro tentar ler do collectors_config.json (prioridade)
        // Se não existir, ler do system_config.json (compatibilidade)
        let possible_config_paths = vec![
            Path::new("collectors_config.json"),
            Path::new("G:/Hive-Hub/News-main/news-backend/collectors_config.json"),
            Path::new("G:/Hive-Hub/News-main/collectors_config.json"),
        ];
        
        let possible_system_paths = vec![
            Path::new("system_config.json"),
            Path::new("G:/Hive-Hub/News-main/news-backend/system_config.json"),
            Path::new("G:/Hive-Hub/News-main/system_config.json"),
        ];
        
        let mut config = None;
        let mut config_path_used = None;
        let mut use_system_config = false;
        
        // Tentar collectors_config.json primeiro
        for path in &possible_config_paths {
            if path.exists() {
                let manager = ConfigManager::new(path);
                match manager.load() {
                    Ok(c) => {
                        config = Some(c.collectors);
                        config_path_used = Some(path.to_path_buf());
                        break;
                    }
                    Err(_) => continue,
                }
            }
        }
        
        // Se não encontrou collectors_config.json, tentar system_config.json
        if config.is_none() {
            for path in &possible_system_paths {
                if path.exists() {
                    let manager = SiteConfigManager::new(path);
                    match manager.load() {
                        Ok(system_config) => {
                            // Pegar coletores de todos os sites habilitados e converter para CollectorConfig do config_manager
                            use crate::utils::config_manager::CollectorConfig as ConfigCollectorConfig;
                            let mut all_collectors: Vec<ConfigCollectorConfig> = Vec::new();
                            for (_site_id, site) in &system_config.sites {
                                if site.enabled {
                                    for site_collector in &site.collectors {
                                        // Converter de site_config_manager::CollectorConfig para config_manager::CollectorConfig
                                        all_collectors.push(ConfigCollectorConfig {
                                            id: site_collector.id.clone(),
                                            name: site_collector.name.clone(),
                                            enabled: site_collector.enabled,
                                            api_key: site_collector.api_key.clone(),
                                            collector_type: site_collector.collector_type.clone(),
                                            feed_url: site_collector.feed_url.clone(),
                                            base_url: site_collector.base_url.clone(),
                                            selectors: site_collector.selectors.clone(),
                                            config: site_collector.config.clone(),
                                        });
                                    }
                                }
                            }
                            config = Some(all_collectors);
                            config_path_used = Some(path.to_path_buf());
                            use_system_config = true;
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
        
        let collectors = match config {
            Some(c) => c,
            None => {
                anyhow::bail!("No config file found (collectors_config.json or system_config.json). Please configure collectors first.");
            }
        };
        
        if let Some(path) = config_path_used {
            println!("  Using config from: {}", path.display());
            if use_system_config {
                println!("  (Reading from system_config.json - all enabled sites)");
            }
        }

        // Determinar fontes habilitadas a partir dos collectors habilitados
        let mut use_arxiv = false;
        let mut use_pmc = false;
        let mut use_ss = false;
        let mut rss_collectors = Vec::new();
        let mut html_collectors = Vec::new();

        for c in &collectors {
            if !c.enabled { continue; }
            
            let collector_type = c.collector_type.as_deref().unwrap_or("api");
            
            match collector_type {
                "rss" => {
                    if let Some(feed_url) = &c.feed_url {
                        let base_url = c.base_url.clone();
                        let max_results = c.config.get("max_results")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);
                        rss_collectors.push((c.id.clone(), feed_url.clone(), base_url, max_results));
                    }
                }
                "html" => {
                    if let Some(base_url) = &c.base_url {
                        let selectors = c.selectors.as_ref().and_then(|s| {
                            use std::collections::HashMap;
                            if let Ok(map) = serde_json::from_value::<HashMap<String, String>>(s.clone()) {
                                Some(map)
                            } else {
                                None
                            }
                        });
                        let max_results = c.config.get("max_results")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);
                        html_collectors.push((c.id.clone(), base_url.clone(), selectors, max_results));
                    }
                }
                _ => {
                    // API collectors (arxiv, pmc, semantic_scholar)
                    let id = c.id.to_lowercase();
                    if id.contains("arxiv") { use_arxiv = true; }
                    if id.contains("pmc") || id.contains("pubmed") { use_pmc = true; }
                    if id.contains("semantic") { use_ss = true; }
                }
            }
        }
        
        println!("Enabled sources summary:");
        println!("  arXiv:   {}", if use_arxiv { "ON" } else { "OFF" });
        println!("  PMC:     {}", if use_pmc { "ON" } else { "OFF" });
        println!("  Semantic:{}", if use_ss { "ON" } else { "OFF" });
        println!("  RSS:     {} collector(s)", rss_collectors.len());
        println!("  HTML:    {} collector(s)", html_collectors.len());

        // Check if there are enabled sites for each source before collecting
        // This prevents unnecessary collection attempts
        
        // For article sources (arxiv, pmc, semantic): check if any site has enabled article collectors
        let has_article_sites = use_arxiv || use_pmc || use_ss;
        if has_article_sites {
            let arxiv_sites = if use_arxiv { get_enabled_sites_for_source("arxiv") } else { Vec::new() };
            let pmc_sites = if use_pmc { get_enabled_sites_for_source("pmc") } else { Vec::new() };
            let semantic_sites = if use_ss { get_enabled_sites_for_source("semantic") } else { Vec::new() };
            
            // Only collect if at least one site has enabled collectors for that source
            if use_arxiv && !arxiv_sites.is_empty() {
                println!("✅ Collecting arXiv articles for {} enabled site(s)", arxiv_sites.len());
                run_arxiv_collection_direct().await?;
            } else if use_arxiv {
                println!("⏭️  Skipping arXiv collection: no enabled sites");
            }
            
            if use_pmc && !pmc_sites.is_empty() {
                println!("✅ Collecting PMC articles for {} enabled site(s)", pmc_sites.len());
                run_pmc_collection_direct().await?;
            } else if use_pmc {
                println!("⏭️  Skipping PMC collection: no enabled sites");
            }
            
            if use_ss && !semantic_sites.is_empty() {
                println!("✅ Collecting Semantic Scholar articles for {} enabled site(s)", semantic_sites.len());
                run_semantic_scholar_collection_direct().await?;
            } else if use_ss {
                println!("⏭️  Skipping Semantic Scholar collection: no enabled sites");
            }
        } else {
            println!("⏭️  Skipping article collection: no article sources enabled");
        }

        // For news sources (rss, html): check if any site has enabled news collectors
        if !rss_collectors.is_empty() {
            let rss_sites = get_enabled_sites_for_source("rss");
            if !rss_sites.is_empty() {
                println!("✅ Collecting RSS news for {} enabled site(s)", rss_sites.len());
                run_rss_collectors(&rss_collectors).await?;
            } else {
                println!("⏭️  Skipping RSS collection: no enabled sites");
            }
        }
        
        if !html_collectors.is_empty() {
            let html_sites = get_enabled_sites_for_source("html");
            if !html_sites.is_empty() {
                println!("✅ Collecting HTML news for {} enabled site(s)", html_sites.len());
                run_html_collectors(&html_collectors).await?;
            } else {
                println!("⏭️  Skipping HTML collection: no enabled sites");
            }
        }

        Ok(())
    }

    fn get_enabled_sites_for_source(source_key: &str) -> Vec<String> {
        use std::path::Path;
        use crate::utils::site_config_manager::SiteConfigManager;
        let config_path = Path::new("G:/Hive-Hub/News-main/news-backend/system_config.json");
        let manager = SiteConfigManager::new(config_path);
        let mut result = Vec::new();
        
        // Determine if source is for articles (arxiv, pmc, semantic) or news (rss, html)
        let is_article_source = matches!(source_key, "arxiv" | "pmc" | "semantic");
        let is_news_source = matches!(source_key, "rss" | "html");
        
        if let Ok(sites) = manager.get_all_sites() {
            for s in sites {
                let mut enabled_for_source = false;
                
                // Skip logging for optimization - only log if site has relevant collectors
                let mut has_relevant_collectors = false;
                
                for c in &s.collectors {
                    let _id = c.id.to_lowercase();
                    let collector_type = c.collector_type.as_deref().unwrap_or("api");
                    
                    // Quick check: if article source, skip sites that only have news collectors
                    if is_article_source && matches!(collector_type, "rss" | "html") {
                        continue; // Skip news collectors when checking article sources
                    }
                    // Quick check: if news source, skip sites that only have article collectors
                    if is_news_source && matches!(collector_type, "api") && !matches!(source_key, "arxiv" | "pmc" | "semantic") {
                        // Only skip if it's an API collector that's NOT for articles (this shouldn't happen, but safety check)
                        continue;
                    }
                    
                    has_relevant_collectors = true;
                    break; // Found at least one relevant collector, can proceed
                }
                
                // Skip site entirely if it has no relevant collectors for this source type
                if !has_relevant_collectors {
                    continue; // Skip site - no relevant collectors for this source type
                }
                
                println!("  🔍 Checking site: {} (id: {})", s.name, s.id);
                
                for c in s.collectors {
                    let id_lower = c.id.to_lowercase();
                    println!("    📦 Collector: {} (enabled: {}, type: {:?})", c.id, c.enabled, c.collector_type);
                    
                    // CRITICAL: Skip disabled collectors first (before any matching)
                    if !c.enabled { 
                        println!("      ⏭️  Skipping disabled collector");
                        continue; 
                    }
                    
                    // Check collector type if available
                    let collector_type = c.collector_type.as_deref().unwrap_or("api");
                    
                    match (source_key, collector_type) {
                        ("arxiv", "api") if id_lower.contains("arxiv") => {
                            enabled_for_source = true;
                            println!("      ✅ Matched: arxiv collector for arxiv source");
                        },
                        ("pmc", "api") if id_lower.contains("pmc") || id_lower.contains("pubmed") => {
                            enabled_for_source = true;
                            println!("      ✅ Matched: pmc/pubmed collector for pmc source");
                        },
                        ("semantic", "api") if id_lower.contains("semantic") => {
                            enabled_for_source = true;
                            println!("      ✅ Matched: semantic collector for semantic source");
                        },
                        ("rss", "rss") | ("rss", _) if id_lower.contains("rss") => {
                            enabled_for_source = true;
                            println!("      ✅ Matched: rss collector for rss source");
                        },
                        ("html", "html") | ("html", _) if id_lower.contains("html") => {
                            enabled_for_source = true;
                            println!("      ✅ Matched: html collector for html source");
                        },
                        _ => {
                            // Fallback: check by ID pattern
                            if source_key == "arxiv" && id_lower.contains("arxiv") { 
                                enabled_for_source = true;
                                println!("      ✅ Matched (fallback): arxiv collector for arxiv source");
                            }
                            if source_key == "pmc" && (id_lower.contains("pmc") || id_lower.contains("pubmed")) { 
                                enabled_for_source = true;
                                println!("      ✅ Matched (fallback): pmc/pubmed collector for pmc source");
                            }
                            if source_key == "semantic" && id_lower.contains("semantic") { 
                                enabled_for_source = true;
                                println!("      ✅ Matched (fallback): semantic collector for semantic source");
                            }
                            if source_key == "rss" && (id_lower.contains("rss") || collector_type == "rss") { 
                                enabled_for_source = true;
                                println!("      ✅ Matched (fallback): rss collector for rss source");
                            }
                            if source_key == "html" && (id_lower.contains("html") || collector_type == "html") { 
                                enabled_for_source = true;
                                println!("      ✅ Matched (fallback): html collector for html source");
                            }
                        }
                    }
                }
                if enabled_for_source { 
                    result.push(s.id.clone());
                    println!("  ✅ Site '{}' (id: {}) added to destinations for source '{}'", s.name, s.id, source_key);
                } else {
                    println!("  ❌ Site '{}' (id: {}) NOT enabled for source '{}'", s.name, s.id, source_key);
                }
            }
        }
        println!("  🎯 Final destinations for '{}': {:?}", source_key, result);
        result
    }

    async fn run_rss_collectors(
        collectors: &[(String, String, Option<String>, Option<u32>)],
    ) -> anyhow::Result<()> {
        use crate::collectors::rss_collector::RssCollector;
        use crate::utils::article_registry::RegistryManager;
        use std::path::{Path, PathBuf};
        use serde_json;

        println!("\n📡 Starting RSS collectors...\n");

        // Inicializar registry
        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        let temp_dir = PathBuf::from("G:/Hive-Hub/News-main/downloads/temp");
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Criar diretório para salvar artigos raw
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let raw_dir = base_dir.join("raw");
        tokio::fs::create_dir_all(&raw_dir).await?;
        
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = raw_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        // Inicializar filtro de notícias (verifica no registry)
        let registry_path = PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json");
        let rejected_dir = base_dir.join("rejected");
        let news_filter = crate::filter::NewsFilter::new(registry_path, rejected_dir)?;
        news_filter.check_registry()?;

        let rss_collector = RssCollector::new(temp_dir);
        let mut total_saved = 0;
        let mut total_rejected = 0;

        println!("📋 Total RSS collectors to process: {}\n", collectors.len());
        
        for (idx, (collector_id, feed_url, base_url, max_results)) in collectors.iter().enumerate() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📡 [RSS COLLECTOR {}/{}] {}", idx + 1, collectors.len(), collector_id);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  🌐 Feed URL: {}", feed_url);
            println!("  📊 Max results: {:?}", max_results);
            println!("  🔗 Base URL: {:?}", base_url);
            println!("  ⏳ Fetching feed...\n");
            
            let fetch_start = std::time::Instant::now();
            
            match rss_collector.fetch_feed(
                feed_url,
                *max_results,
                base_url.as_deref(),
            ).await {
                Ok(articles) => {
                    let fetch_duration = fetch_start.elapsed();
                    println!("  ✅ Fetch completed in {:?}", fetch_duration);
                    println!("  📄 Found {} articles from {}", articles.len(), collector_id);
                    
                    // Salvar artigos coletados
                    println!("  🔄 Processing {} articles...\n", articles.len());
                    
                    for (art_idx, article) in articles.iter().enumerate() {
                        println!("    ┌─ [ARTICLE {}/{}] {}", art_idx + 1, articles.len(), article.id);
                        println!("    │  📝 Title: {}", article.title);
                        println!("    │  🔗 URL: {}", article.url);
                        
                        // Verificar se já está registrado
                        println!("    │  🔍 Checking if already registered...");
                        if registry.is_article_registered(&article.id) {
                            println!("    │  ⏭️  Already registered - skipping");
                            println!("    └─\n");
                            continue;
                        }

                        // Verificar se é duplicata no registry (por ID ou URL)
                        println!("    │  🔍 Checking for duplicates...");
                        if news_filter.is_duplicate(&article.id, &article.url) {
                            println!("    │  ⚠️  Duplicate detected (ID or URL already exists)");
                            println!("    │  ⏭️  Rejecting: {} - {}", article.id, article.title);
                            
                            // Salvar JSON temporariamente para poder mover
                            println!("    │  💾 Saving JSON temporarily...");
                            let json_path = date_dir.join(format!("{}.json", article.id));
                            if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                                if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                                    eprintln!("    │  ❌ Failed to save JSON: {}", e);
                                    println!("    └─\n");
                                    continue;
                                }
                                println!("    │  ✅ JSON saved: {}", json_path.display());
                            }

                            // Mover para pasta rejected
                            println!("    │  🗑️  Moving to rejected folder...");
                            if let Err(e) = news_filter.reject_news(&json_path).await {
                                eprintln!("    │  ❌ Failed to reject: {}", e);
                                // Deletar arquivo se não conseguir mover
                                let _ = tokio::fs::remove_file(&json_path).await;
                                println!("    └─\n");
                                continue;
                            }
                            
                            println!("    │  ✅ Moved to rejected folder");
                            println!("    └─ ❌ REJECTED\n");
                            total_rejected += 1;
                            continue;
                        }
                        
                        println!("    │  ✅ Not a duplicate - proceeding");

                        // Salvar JSON em downloads/raw/{date}/{id}.json
                        println!("    │  💾 Saving JSON to raw folder...");
                        let json_path = date_dir.join(format!("{}.json", article.id));
                        if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                            if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                                eprintln!("    │  ❌ Failed to save JSON: {}", e);
                                println!("    └─\n");
                                continue;
                            }
                            println!("    │  ✅ JSON saved: {}", json_path.display());
                        }

                        // Registrar no registry (usando url como pdf_url para compatibilidade)
                        println!("    │  📝 Registering in registry...");
                        if let Err(e) = registry.register_collected(
                            article.id.clone(),
                            article.title.clone(),
                            article.url.clone(),
                            article.url.clone(), // Web articles não têm PDF, usar URL como pdf_url
                        ) {
                            eprintln!("    │  ❌ Failed to register: {}", e);
                            println!("    └─\n");
                            continue;
                        }
                        println!("    │  ✅ Registered successfully");

                        // Definir destinos baseado nos sites que têm RSS/HTML collectors habilitados
                        println!("    │  🎯 Setting destinations...");
                        let destinations = get_enabled_sites_for_source("rss");
                        if let Err(e) = registry.set_destinations(&article.id, destinations.clone()) {
                            eprintln!("    │  ⚠️  Failed to set destinations: {}", e);
                        } else {
                            println!("    │  ✅ Destinations set: {:?}", destinations);
                        }

                        total_saved += 1;
                        println!("    └─ ✅ SAVED: {} - {}\n", article.id, article.title);
                    }
                }
                Err(e) => {
                    println!("    ❌ Error: {}", e);
                }
            }
        }

        println!("\n✅ RSS collection completed!");
        println!("   Saved {} new articles to {}", total_saved, date_dir.display());
        if total_rejected > 0 {
            println!("   Rejected {} duplicate articles (already published)", total_rejected);
        }
        Ok(())
    }

    async fn run_html_collectors(
        collectors: &[(String, String, Option<std::collections::HashMap<String, String>>, Option<u32>)],
    ) -> anyhow::Result<()> {
        use crate::collectors::html_collector::HtmlCollector;
        use crate::utils::article_registry::RegistryManager;
        use std::path::{Path, PathBuf};
        use serde_json;

        println!("\n🌐 Starting HTML collectors...\n");

        // Inicializar registry
        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        let temp_dir = PathBuf::from("G:/Hive-Hub/News-main/downloads/temp");
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Criar diretório para salvar artigos raw
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let raw_dir = base_dir.join("raw");
        tokio::fs::create_dir_all(&raw_dir).await?;
        
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = raw_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        // Inicializar filtro de notícias (verifica no registry)
        let registry_path = PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json");
        let rejected_dir = base_dir.join("rejected");
        let news_filter = crate::filter::NewsFilter::new(registry_path, rejected_dir)?;
        news_filter.check_registry()?;

        let html_collector = HtmlCollector::new(temp_dir);
        let mut total_saved = 0;
        let mut total_rejected = 0;

        println!("📋 Total collectors to process: {}\n", collectors.len());
        
        for (idx, (collector_id, base_url, selectors, max_results)) in collectors.iter().enumerate() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📰 [HTML COLLECTOR {}/{}] {}", idx + 1, collectors.len(), collector_id);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  🌐 URL: {}", base_url);
            println!("  📊 Max results: {:?}", max_results);
            println!("  🔍 Selectors: {:?}", selectors.as_ref().map(|s| s.keys().collect::<Vec<_>>()));
            println!("  ⏳ Fetching page...\n");
            
            let fetch_start = std::time::Instant::now();
            
            match html_collector.fetch_page(
                base_url,
                selectors.as_ref(),
                *max_results,
                Some(collector_id), // Passar collector_id para detectar se precisa JS rendering
            ).await {
                Ok(articles) => {
                    let fetch_duration = fetch_start.elapsed();
                    println!("  ✅ Fetch completed in {:?}", fetch_duration);
                    println!("  📄 Found {} articles from {}", articles.len(), collector_id);
                    
                    // Salvar artigos coletados
                    println!("  🔄 Processing {} articles...\n", articles.len());
                    
                    for (art_idx, article) in articles.iter().enumerate() {
                        println!("    ┌─ [ARTICLE {}/{}] {}", art_idx + 1, articles.len(), article.id);
                        println!("    │  📝 Title: {}", article.title);
                        println!("    │  🔗 URL: {}", article.url);
                        
                        // Verificar se já está registrado
                        println!("    │  🔍 Checking if already registered...");
                        if registry.is_article_registered(&article.id) {
                            println!("    │  ⏭️  Already registered - skipping");
                            println!("    └─\n");
                            continue;
                        }

                        // Verificar se é duplicata no registry (por ID ou URL)
                        println!("    │  🔍 Checking for duplicates...");
                        if news_filter.is_duplicate(&article.id, &article.url) {
                            println!("    │  ⚠️  Duplicate detected (ID or URL already exists)");
                            println!("    │  ⏭️  Rejecting: {} - {}", article.id, article.title);
                            
                            // Salvar JSON temporariamente para poder mover
                            println!("    │  💾 Saving JSON temporarily...");
                            let json_path = date_dir.join(format!("{}.json", article.id));
                            if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                                if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                                    eprintln!("    │  ❌ Failed to save JSON: {}", e);
                                    println!("    └─\n");
                                    continue;
                                }
                                println!("    │  ✅ JSON saved: {}", json_path.display());
                            }

                            // Mover para pasta rejected
                            println!("    │  🗑️  Moving to rejected folder...");
                            if let Err(e) = news_filter.reject_news(&json_path).await {
                                eprintln!("    │  ❌ Failed to reject: {}", e);
                                // Deletar arquivo se não conseguir mover
                                let _ = tokio::fs::remove_file(&json_path).await;
                                println!("    └─\n");
                                continue;
                            }
                            
                            println!("    │  ✅ Moved to rejected folder");
                            println!("    └─ ❌ REJECTED\n");
                            total_rejected += 1;
                            continue;
                        }
                        
                        println!("    │  ✅ Not a duplicate - proceeding");

                        // Salvar JSON em downloads/raw/{date}/{id}.json
                        println!("    │  💾 Saving JSON to raw folder...");
                        let json_path = date_dir.join(format!("{}.json", article.id));
                        if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                            if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                                eprintln!("    │  ❌ Failed to save JSON: {}", e);
                                println!("    └─\n");
                                continue;
                            }
                            println!("    │  ✅ JSON saved: {}", json_path.display());
                        }

                        // Registrar no registry (usando url como pdf_url para compatibilidade)
                        println!("    │  📝 Registering in registry...");
                        if let Err(e) = registry.register_collected(
                            article.id.clone(),
                            article.title.clone(),
                            article.url.clone(),
                            article.url.clone(), // Web articles não têm PDF, usar URL como pdf_url
                        ) {
                            eprintln!("    │  ❌ Failed to register: {}", e);
                            println!("    └─\n");
                            continue;
                        }
                        println!("    │  ✅ Registered successfully");

                        // Definir destinos baseado nos sites que têm RSS/HTML collectors habilitados
                        println!("    │  🎯 Setting destinations...");
                        let destinations = get_enabled_sites_for_source("html");
                        if let Err(e) = registry.set_destinations(&article.id, destinations.clone()) {
                            eprintln!("    │  ⚠️  Failed to set destinations: {}", e);
                        } else {
                            println!("    │  ✅ Destinations set: {:?}", destinations);
                        }

                        total_saved += 1;
                        println!("    └─ ✅ SAVED: {} - {}\n", article.id, article.title);
                    }
                }
                Err(e) => {
                    println!("    ❌ Error: {}", e);
                }
            }
        }

        println!("\n✅ HTML collection completed!");
        println!("   Saved {} new articles to {}", total_saved, date_dir.display());
        if total_rejected > 0 {
            println!("   Rejected {} duplicate articles (already published)", total_rejected);
        }
        Ok(())
    }

    async fn run_news_writer() -> anyhow::Result<()> {
        use crate::writer::news_writer::NewsWriterService;
        use std::path::{Path, PathBuf};

        println!("📰 Processing collected news articles...\n");

        // Inicializar news writer
        let output_base = PathBuf::from("G:/Hive-Hub/News-main/output");
        let registry_path = PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json");
        let news_writer = NewsWriterService::new(output_base, registry_path)?;

        // Encontrar todos os artigos coletados em downloads/raw/
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let raw_dir = base_dir.join("raw");
        
        if !raw_dir.exists() {
            println!("⚠️  No raw directory found. Run collectors first.");
            return Ok(());
        }

        // Encontrar todas as pastas de data
        let mut all_articles = Vec::new();
        
        let mut date_entries = tokio::fs::read_dir(&raw_dir).await?;
        while let Some(date_entry) = date_entries.next_entry().await? {
            let date_dir = date_entry.path();
            
            if !date_dir.is_dir() {
                continue;
            }

            // Encontrar todos os arquivos JSON neste diretório
            let mut json_entries = tokio::fs::read_dir(&date_dir).await?;
            while let Some(json_entry) = json_entries.next_entry().await? {
                let json_path = json_entry.path();
                
                if json_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    all_articles.push(json_path);
                }
            }
        }

        if all_articles.is_empty() {
            println!("⚠️  No news articles found in downloads/raw/");
            return Ok(());
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📄 Found {} news articles to process", all_articles.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let mut processed = 0;
        let mut failed = 0;
        let mut successfully_processed_articles = Vec::new();

        // Processar cada artigo
        for (i, article_path) in all_articles.iter().enumerate() {
            let article_id = article_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("✍️  [WRITER {}/{}] {}", i + 1, all_articles.len(), article_id);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  📄 JSON path: {}", article_path.display());
            println!("  ⏳ Processing article...\n");

            let process_start = std::time::Instant::now();

            match news_writer.process_news_article(article_path).await {
                Ok(results) => {
                    let process_duration = process_start.elapsed();
                    println!("  ✅ Processing completed in {:?}", process_duration);
                    println!("  📊 Generated content for {} destination(s):", results.len());
                    
                    for (idx, result) in results.iter().enumerate() {
                        println!("    [{}/{}] ✅ {} → {}", idx + 1, results.len(), result.site_name, result.output_dir.display());
                    }
                    
                    processed += 1;
                    successfully_processed_articles.push(article_path.clone());
                    println!("");
                }
                Err(e) => {
                    let process_duration = process_start.elapsed();
                    eprintln!("  ❌ Processing failed after {:?}: {}", process_duration, e);
                    failed += 1;
                    println!("");
                }
            }
        }

        println!("\n✅ News writing completed!");
        println!("   Processed: {}", processed);
        if failed > 0 {
            println!("   Failed: {}", failed);
        }

        // Cleanup: verificar arquivos criados, atualizar registry e remover JSONs processados
        if !successfully_processed_articles.is_empty() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🧹 [CLEANUP] Starting cleanup for {} processed articles", successfully_processed_articles.len());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            
            let cleanup_start = std::time::Instant::now();
            
            match news_writer.cleanup_processed_articles(&successfully_processed_articles).await {
                Ok(stats) => {
                    let cleanup_duration = cleanup_start.elapsed();
                    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("✅ [CLEANUP] Completed in {:?}", cleanup_duration);
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("  ✅ Verified: {} articles", stats.verified);
                    println!("  📝 Updated in registry: {} articles", stats.updated);
                    println!("  🗑️  Removed from raw: {} JSON files", stats.removed);
                    println!("");
                }
                Err(e) => {
                    let cleanup_duration = cleanup_start.elapsed();
                    eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    eprintln!("❌ [CLEANUP] Failed after {:?}: {}", cleanup_duration, e);
                    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                }
            }
        }

        Ok(())
    }

    async fn run_cleanup_news() -> anyhow::Result<()> {
        use crate::writer::news_writer::NewsWriterService;
        use crate::utils::article_registry::RegistryManager;
        use std::path::{Path, PathBuf};

        println!("🧹 Running cleanup on already processed articles...\n");

        // Inicializar news writer
        let output_base = PathBuf::from("G:/Hive-Hub/News-main/output");
        let registry_path = PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json");
        
        let registry = RegistryManager::new(&registry_path)?;
        let news_writer = NewsWriterService::new(output_base, registry_path)?;

        // Encontrar todos os artigos no registry que já foram publicados (têm output_dir)
        let registry_data = registry.get_all_articles();
        let mut processed_articles = Vec::new();

        for metadata in registry_data {
            if let Some(_output_dir) = &metadata.output_dir {
                // Verificar se o artigo tem um JSON correspondente na pasta raw
                let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
                let raw_dir = base_dir.join("raw");
                
                // Procurar o JSON em todas as pastas de data
                let mut json_found: Option<PathBuf> = None;
                
                if raw_dir.exists() {
                    let mut date_entries = tokio::fs::read_dir(&raw_dir).await?;
                    while let Some(date_entry) = date_entries.next_entry().await? {
                        let date_dir = date_entry.path();
                        if !date_dir.is_dir() {
                            continue;
                        }
                        
                        let json_path = date_dir.join(format!("{}.json", metadata.id));
                        if json_path.exists() {
                            json_found = Some(json_path);
                            break;
                        }
                    }
                }

                if let Some(json_path) = json_found {
                    let path_display = json_path.display().to_string();
                    processed_articles.push(json_path);
                    println!("  📄 Found JSON for {}: {}", metadata.id, path_display);
                } else {
                    println!("  ℹ️  Article {} has no JSON in raw/ (already cleaned?)", metadata.id);
                }
            } else {
                // Even if no output_dir, check if JSON exists in raw/ (orphaned files)
                let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
                let raw_dir = base_dir.join("raw");
                
                if raw_dir.exists() {
                    let mut date_entries = tokio::fs::read_dir(&raw_dir).await?;
                    while let Some(date_entry) = date_entries.next_entry().await? {
                        let date_dir = date_entry.path();
                        if !date_dir.is_dir() {
                            continue;
                        }
                        
                        let json_path = date_dir.join(format!("{}.json", metadata.id));
                        if json_path.exists() {
                            let path_display = json_path.display().to_string();
                            processed_articles.push(json_path);
                            println!("  📄 Found orphaned JSON for {}: {}", metadata.id, path_display);
                            break;
                        }
                    }
                }
            }
        }

        if processed_articles.is_empty() {
            println!("⚠️  No articles found that need cleanup.");
            return Ok(());
        }

        println!("\n📄 Found {} articles to cleanup\n", processed_articles.len());

        // Executar cleanup
        match news_writer.cleanup_processed_articles(&processed_articles).await {
            Ok(stats) => {
                println!("\n✅ Cleanup completed:");
                println!("   Verified: {}", stats.verified);
                println!("   Updated in registry: {}", stats.updated);
                println!("   Removed from raw: {}", stats.removed);
            }
            Err(e) => {
                eprintln!("  ❌ Cleanup failed: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn run_news_pipeline() -> anyhow::Result<()> {
        let pipeline_start = std::time::Instant::now();
        
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔄 NEWS PIPELINE - COMPLETE PROCESSING FLOW                  ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("");

        // Step 1: Collect news
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  📥 STEP 1: COLLECT NEWS                                       ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("");
        let collect_start = std::time::Instant::now();
        run_collect_enabled_from_config().await?;
        let collect_duration = collect_start.elapsed();
        println!("");
        println!("✅ STEP 1 completed in {:?}\n", collect_duration);

        // Step 2: Filter news (already integrated in collect, but show status)
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔍 STEP 2: FILTER NEWS (duplicate check)                     ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("✅ Filtering is integrated in collection step");
        println!("✅ STEP 2 completed\n");

        // Step 3: Write news
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ✍️  STEP 3: WRITE NEWS ARTICLES                                ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("");
        let write_start = std::time::Instant::now();
        run_news_writer().await?;
        let write_duration = write_start.elapsed();
        println!("");
        println!("✅ STEP 3 completed in {:?}\n", write_duration);

        // Step 4: Cleanup processed articles
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🧹 STEP 4: CLEANUP PROCESSED ARTICLES                         ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("");
        let cleanup_start = std::time::Instant::now();
        run_cleanup_news().await?;
        let cleanup_duration = cleanup_start.elapsed();
        println!("");
        println!("✅ STEP 4 completed in {:?}\n", cleanup_duration);

        let pipeline_duration = pipeline_start.elapsed();
        
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ✅ PIPELINE COMPLETED SUCCESSFULLY!                            ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("");
        println!("📊 Total execution time: {:?}", pipeline_duration);
        println!("   📥 Collection: {:?}", collect_duration);
        println!("   ✍️  Writing: {:?}", write_duration);
        println!("   🧹 Cleanup: {:?}", cleanup_duration);
        println!("");

        Ok(())
    }

    /// Helper function to log with timestamp
    fn debug_log(message: &str) {
        let now = chrono::Utc::now();
        println!("[{}] {}", now.format("%H:%M:%S%.3f"), message);
    }

    /// Run news pipeline with ultra-detailed debug logging
    async fn run_news_pipeline_debug() -> anyhow::Result<()> {
        let pipeline_start = std::time::Instant::now();
        let start_time = chrono::Utc::now();
        
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("🔄 NEWS PIPELINE - DEBUG MODE ACTIVATED");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log(&format!("Pipeline started at: {}", start_time.format("%Y-%m-%d %H:%M:%S%.3f")));
        debug_log("");

        // Step 1: Collect news
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("📥 STEP 1: COLLECT NEWS (DEBUG MODE)");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("");
        
        let collect_start = std::time::Instant::now();
        debug_log("Calling run_collect_enabled_from_config()...");
        
        match run_collect_enabled_from_config().await {
            Ok(_) => {
                let collect_duration = collect_start.elapsed();
                debug_log(&format!("✅ STEP 1 completed successfully in {:?}", collect_duration));
                debug_log(&format!("   Duration breakdown: {}s", collect_duration.as_secs_f64()));
            }
            Err(e) => {
                let collect_duration = collect_start.elapsed();
                debug_log(&format!("❌ STEP 1 failed after {:?}: {}", collect_duration, e));
                return Err(e);
            }
        }
        debug_log("");

        // Step 2: Filter news (already integrated in collect, but show status)
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("🔍 STEP 2: FILTER NEWS (duplicate check)");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("ℹ️  Filtering is integrated in collection step");
        debug_log("   - Duplicate checking happens during collection");
        debug_log("   - Articles are checked against registry");
        debug_log("   - Duplicates are moved to rejected/ folder");
        debug_log("✅ STEP 2 completed (integrated with Step 1)");
        debug_log("");

        // Step 3: Write news
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("✍️  STEP 3: WRITE NEWS ARTICLES (DEBUG MODE)");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("");
        
        let write_start = std::time::Instant::now();
        debug_log("Calling run_news_writer()...");
        
        match run_news_writer().await {
            Ok(_) => {
                let write_duration = write_start.elapsed();
                debug_log(&format!("✅ STEP 3 completed successfully in {:?}", write_duration));
                debug_log(&format!("   Duration breakdown: {}s", write_duration.as_secs_f64()));
            }
            Err(e) => {
                let write_duration = write_start.elapsed();
                debug_log(&format!("❌ STEP 3 failed after {:?}: {}", write_duration, e));
                return Err(e);
            }
        }
        debug_log("");

        // Step 4: Cleanup processed articles
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("🧹 STEP 4: CLEANUP PROCESSED ARTICLES (DEBUG MODE)");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("");
        
        let cleanup_start = std::time::Instant::now();
        debug_log("Calling run_cleanup_news()...");
        
        match run_cleanup_news().await {
            Ok(_) => {
                let cleanup_duration = cleanup_start.elapsed();
                debug_log(&format!("✅ STEP 4 completed successfully in {:?}", cleanup_duration));
                debug_log(&format!("   Duration breakdown: {}s", cleanup_duration.as_secs_f64()));
            }
            Err(e) => {
                let cleanup_duration = cleanup_start.elapsed();
                debug_log(&format!("❌ STEP 4 failed after {:?}: {}", cleanup_duration, e));
                return Err(e);
            }
        }
        debug_log("");

        let pipeline_duration = pipeline_start.elapsed();
        let end_time = chrono::Utc::now();
        
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("✅ PIPELINE COMPLETED SUCCESSFULLY!");
        debug_log("═══════════════════════════════════════════════════════════════════");
        debug_log("");
        debug_log(&format!("Pipeline started:  {}", start_time.format("%Y-%m-%d %H:%M:%S%.3f")));
        debug_log(&format!("Pipeline finished: {}", end_time.format("%Y-%m-%d %H:%M:%S%.3f")));
        debug_log(&format!("Total execution time: {:?}", pipeline_duration));
        debug_log("");
        debug_log("📊 Detailed Timing Breakdown:");
        debug_log(&format!("   📥 Collection: {:?} ({:.2}s)", collect_start.elapsed(), collect_start.elapsed().as_secs_f64()));
        debug_log(&format!("   ✍️  Writing:   {:?} ({:.2}s)", write_start.elapsed(), write_start.elapsed().as_secs_f64()));
        debug_log(&format!("   🧹 Cleanup:   {:?} ({:.2}s)", cleanup_start.elapsed(), cleanup_start.elapsed().as_secs_f64()));
        debug_log(&format!("   🕐 Total:      {:?} ({:.2}s)", pipeline_duration, pipeline_duration.as_secs_f64()));
        debug_log("");
        debug_log("═══════════════════════════════════════════════════════════════════");

        Ok(())
    }

    async fn run_rss_collector_test() -> anyhow::Result<()> {
        use crate::collectors::rss_collector::RssCollector;
        use crate::utils::article_registry::RegistryManager;
        use std::path::{Path, PathBuf};
        use serde_json;

        println!("📡 Testing RSS Collector with real feeds...\n");

        // Inicializar registry
        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        // Criar diretório para salvar artigos raw
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let raw_dir = base_dir.join("raw");
        tokio::fs::create_dir_all(&raw_dir).await?;
        
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = raw_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        let temp_dir = PathBuf::from("G:/Hive-Hub/News-main/downloads/temp");
        tokio::fs::create_dir_all(&temp_dir).await?;

        let collector = RssCollector::new(temp_dir);

        // Test 1: OpenAI Blog RSS
        println!("1️⃣  OpenAI Blog RSS");
        println!("   URL: https://openai.com/blog/rss.xml\n");
        
        match collector.fetch_feed(
            "https://openai.com/blog/rss.xml",
            Some(3), // Apenas 3 artigos para teste
            Some("https://openai.com"),
        ).await {
            Ok(articles) => {
                println!("   ✅ Collected {} articles:", articles.len());
                let mut saved_count = 0;
                
                for article in articles {
                    // Verificar se já está registrado
                    if registry.is_article_registered(&article.id) {
                        println!("      ⏭️  Skipped (already registered): {}", article.id);
                        continue;
                    }

                    // Salvar JSON em downloads/raw/{date}/{id}.json
                    let json_path = date_dir.join(format!("{}.json", article.id));
                    if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                        if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                            eprintln!("      ⚠️  Failed to save JSON for {}: {}", article.id, e);
                            continue;
                        }
                    }

                    // Registrar no registry
                    if let Err(e) = registry.register_collected(
                        article.id.clone(),
                        article.title.clone(),
                        article.url.clone(),
                        article.url.clone(),
                    ) {
                        eprintln!("      ⚠️  Failed to register {}: {}", article.id, e);
                        continue;
                    }

                    // Definir destinos
                    let destinations = get_enabled_sites_for_source("rss");
                    if let Err(e) = registry.set_destinations(&article.id, destinations) {
                        eprintln!("      ⚠️  Failed to set destinations for {}: {}", article.id, e);
                    }

                    saved_count += 1;
                    println!("      ✅ Saved: {} - {}", article.id, article.title);
                    println!("         Content: {} chars", article.content_text.as_ref().map(|s| s.len()).unwrap_or(0));
                    println!("         URL: {}", article.url);
                }
                
                println!("\n   📊 Saved {} new articles to {}", saved_count, date_dir.display());
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }

        println!("\n✅ RSS Collector test completed!\n");
        
        Ok(())
    }

    async fn run_html_collector_test() -> anyhow::Result<()> {
        use crate::collectors::html_collector::HtmlCollector;
        use crate::utils::article_registry::RegistryManager;
        use std::collections::HashMap;
        use std::path::{Path, PathBuf};
        use serde_json;

        println!("🌐 Testing HTML Collector with real pages...\n");

        // Inicializar registry
        let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
        let registry = RegistryManager::new(registry_path)?;

        // Criar diretório para salvar artigos raw
        let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
        let raw_dir = base_dir.join("raw");
        tokio::fs::create_dir_all(&raw_dir).await?;
        
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = raw_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        let temp_dir = PathBuf::from("G:/Hive-Hub/News-main/downloads/temp");
        tokio::fs::create_dir_all(&temp_dir).await?;

        let collector = HtmlCollector::new(temp_dir);

        // Test 1: TechCrunch (HTML scraping friendly)
        println!("1️⃣  TechCrunch - HTML Articles");
        println!("   URL: https://techcrunch.com/\n");

        // Seletores para TechCrunch (site mais acessível para scraping)
        let mut techcrunch_selectors = HashMap::new();
        techcrunch_selectors.insert("article".to_string(), "article.post-block".to_string());
        techcrunch_selectors.insert("title".to_string(), "a.post-block__title__link".to_string());
        techcrunch_selectors.insert("content".to_string(), ".article-content".to_string());

        match collector.fetch_page(
            "https://techcrunch.com/",
            Some(&techcrunch_selectors),
            Some(3), // Apenas 3 artigos para teste
            None, // TechCrunch não precisa de JS rendering
        ).await {
            Ok(articles) => {
                println!("   ✅ Collected {} articles:", articles.len());
                let mut saved_count = 0;
                
                for article in articles {
                    // Verificar se já está registrado
                    if registry.is_article_registered(&article.id) {
                        println!("      ⏭️  Skipped (already registered): {}", article.id);
                        continue;
                    }

                    // Salvar JSON em downloads/raw/{date}/{id}.json
                    let json_path = date_dir.join(format!("{}.json", article.id));
                    if let Ok(json_str) = serde_json::to_string_pretty(&article) {
                        if let Err(e) = tokio::fs::write(&json_path, json_str).await {
                            eprintln!("      ⚠️  Failed to save JSON for {}: {}", article.id, e);
                            continue;
                        }
                    }

                    // Registrar no registry
                    if let Err(e) = registry.register_collected(
                        article.id.clone(),
                        article.title.clone(),
                        article.url.clone(),
                        article.url.clone(),
                    ) {
                        eprintln!("      ⚠️  Failed to register {}: {}", article.id, e);
                        continue;
                    }

                    // Definir destinos
                    let destinations = get_enabled_sites_for_source("html");
                    if let Err(e) = registry.set_destinations(&article.id, destinations) {
                        eprintln!("      ⚠️  Failed to set destinations for {}: {}", article.id, e);
                    }

                    saved_count += 1;
                    println!("      ✅ Saved: {} - {}", article.id, article.title);
                    println!("         Content: {} chars", article.content_text.as_ref().map(|s| s.len()).unwrap_or(0));
                    println!("         URL: {}", article.url);
                }
                
                println!("\n   📊 Saved {} new articles to {}", saved_count, date_dir.display());
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                println!("   ℹ️  Note: Some sites may block scrapers or have different HTML structure");
            }
        }

        println!("\n✅ HTML Collector test completed!\n");
        
        Ok(())
    }

    /// Ensure destinations are present for each article in the given date directory
    fn ensure_destinations_for_cycle(source_key: &str, date_dir: &std::path::Path, registry: &RegistryManager) {
        use std::fs;
        let mut updated = 0usize;
        let mut skipped = 0usize;
        let mut errors = 0usize;
        let destinations = get_enabled_sites_for_source(source_key);
        if !date_dir.exists() {
            println!("  (No date directory found: {})", date_dir.display());
            return;
        }

        let entries = match fs::read_dir(date_dir) { Ok(e) => e, Err(e) => { eprintln!("  Failed to read dir: {}", e); return; } };
        for entry in entries {
            if let Ok(ent) = entry {
                let path = ent.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                    let id = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if id.is_empty() { continue; }
                    let meta = registry.get_metadata(&id);
                    let already = meta.as_ref().and_then(|m| m.destinations.clone()).map(|v| !v.is_empty()).unwrap_or(false);
                    if already { skipped += 1; continue; }
                    if let Err(e) = registry.set_destinations(&id, destinations.clone()) {
                        eprintln!("  Failed to set destinations for {}: {}", id, e);
                        errors += 1;
                    } else {
                        updated += 1;
                    }
                }
            }
        }
        println!("  Destinations finalized → updated={}, skipped(existing)={}, errors={}", updated, skipped, errors);
    }

    

    async fn cleanup_temp_files(temp_dir: &std::path::Path) -> anyhow::Result<()> {
        use std::fs;

        if !temp_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(temp_dir)?;
        let mut deleted_count = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Deletar apenas arquivos XML temporários
            if path.is_file() && path.extension().map(|e| e == "xml").unwrap_or(false) {
                match fs::remove_file(&path) {
                    Ok(_) => {
                        deleted_count += 1;
                        println!(
                            "  ✓ Deleted: {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    }
                    Err(e) => {
                        eprintln!("  ✗ Failed to delete {}: {}", path.display(), e);
                    }
                }
            }
        }

        if deleted_count > 0 {
            println!("  Cleaned {} temporary file(s)", deleted_count);
        } else {
            println!("  No temporary files to clean");
        }

        Ok(())
    }

    // Connect to database (optional - auth and config work without DB)
    // Note: For now, database is required to start the server
    // Auth uses file-based storage (users.json), so it could work without DB
    // TODO: Make database truly optional
    let db = Database::new().await?;

    // Build application
    let app = Router::new()
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/auth/me", get(routes::auth::get_me))
        .route("/api/auth/change-password", post(routes::auth::change_password))
        .route("/api/collectors", get(routes::collectors::get_collectors))
        .route("/api/collectors/enabled", get(routes::collectors::get_enabled_collectors))
        .route("/api/collectors/:id/status", put(routes::collectors::update_collector_status))
        .route("/api/collectors/:id/config", put(routes::collectors::update_collector_config))
        .route("/api/collectors/:id/sites", put(routes::collectors::update_collector_sites))
        .route("/api/sites", get(routes::sites::get_all_sites))
        .route("/api/sites/:site_id", get(routes::sites::get_site_config))
        .route("/api/sites/:site_id/writer", put(routes::sites::update_writer_config))
        .route("/api/sites/:site_id/collectors/:collector_id/status", put(routes::sites::update_collector_status))
        .route("/api/sites/:site_id/social/:social_id/status", put(routes::sites::update_social_status))
        .route("/api/sites/:site_id/social/:social_id/config", put(routes::sites::update_social_config))
        .route("/api/sites/:site_id/education/:source_id/status", put(routes::sites::update_education_status))
        .route("/api/sites/:site_id/education/:source_id/config", put(routes::sites::update_education_config))
        .route("/api/sites/:site_id/collect/start", post(routes::sites::start_collect_for_site))
        .route("/api/pages", get(routes::pages::list_pages))
        .route("/api/pages", post(routes::pages::create_page))
        .route("/api/pages/:id", get(routes::pages::get_page))
        .route("/api/pages/:id", put(routes::pages::update_page))
        .route("/api/pages/:id", delete(routes::pages::delete_page))
        .route("/api/sources", get(routes::sources::list_sources))
        .route("/api/sources", post(routes::sources::create_source))
        .route("/api/logs", get(routes::logs::list_logs))
        .route("/api/logs/articles/:id", delete(routes::logs::hide_article))
        .route("/api/logs/articles/:id/hidden", put(routes::logs::set_hidden))
        .route("/api/logs/articles/:id/featured", put(routes::logs::set_featured))
        .route("/api/logs/enrich-titles", post(routes::logs::enrich_titles_from_arxiv))
        .route("/api/health", get(routes::system::health))
        .route("/api/system/status", get(routes::system::system_status))
        .nest("/api/courses", routes::courses::router())
        .route(
            "/api/collector/start",
            post(routes::collector::start_collection),
        )
        .route(
            "/api/collector/status/:portal_id",
            get(routes::collector::get_collection_status),
        )
        .route(
            "/api/collector/logs",
            get(routes::collector::list_collection_logs),
        )
        .layer(Extension(Arc::new(db)))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005").await?;
    tracing::info!("Server listening on http://localhost:3005");

    axum::serve(listener, app).await?;

    Ok(())
}

// removed legacy health_check; using routes::system::health

async fn run_writer_pipeline() -> anyhow::Result<()> {
    use crate::writer::WriterService;
    use crate::utils::site_config_manager::SiteConfigManager;
    use std::path::Path;
    
    // Inicializar registry
    let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
    let registry = RegistryManager::new(registry_path)?;
    
    // Try to determine which site to use from config or env
    let site_id = std::env::var("WRITER_DEFAULT_SITE")
        .ok()
        .map(|s| s.to_lowercase())
        .or_else(|| {
            // Try to find first enabled site in config
            let config_manager = SiteConfigManager::new(Path::new("system_config.json"));
            if let Ok(sites) = config_manager.get_all_sites() {
                sites.iter()
                    .find(|s| s.enabled && s.writer.enabled)
                    .map(|s| s.id.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "airesearch".to_string());
    
    println!("📝 Using site: {}", site_id);
    let writer = WriterService::new_with_site(Some(&site_id))?;
    
    // Scan filtered directory for approved PDFs
    let filtered_dir = Path::new("G:/Hive-Hub/News-main/downloads/filtered");
    let all_approved_pdfs = scan_filtered_directory(filtered_dir)?;
    
    println!("📄 Found {} approved documents in filtered/\n", all_approved_pdfs.len());
    
    if all_approved_pdfs.is_empty() {
        println!("⚠️  No filtered PDFs found in downloads/filtered/");
        println!("   Run collector first to generate content");
        return Ok(());
    }
    
    // Filtrar apenas PDFs ainda não processados (filtered mas não published)
    let mut pending_pdfs = Vec::new();
    let site = writer.get_site();
    let _output_base = writer.get_output_base();
    
    for pdf_path in all_approved_pdfs.iter() {
        let article_id = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        // Verificar no registry se já foi publicado
        if !registry.is_article_published(article_id) {
            pending_pdfs.push(pdf_path.clone());
        } else {
            println!("⏭️  Skipping {} (already published)", article_id);
        }
    }
    
    println!("📝 {} new documents to process for {}\n", pending_pdfs.len(), site);
    
    if pending_pdfs.is_empty() {
        println!("✅ All documents already processed for {}", site);
        return Ok(());
    }
    
    for (i, pdf_path) in pending_pdfs.iter().enumerate() {
        let filename = pdf_path.file_name().unwrap().to_string_lossy();
        let article_id = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        println!("[{}/{}] Processing: {}", i + 1, pending_pdfs.len(), filename);
        println!("  Phase 1: Generating article (Nature/Science style)...");
        
        match writer.process_pdf(pdf_path).await {
            Ok(result) => {
                println!("  ✅ Content saved → {}", result.output_dir.display());
                println!("     Tokens: {} → {} ({:.1}% savings)", 
                         result.original_tokens,
                         result.compressed_tokens,
                         result.compression_ratio * 100.0);
                
                // Registrar como publicado no registry
                if let Err(e) = registry.register_published(article_id, result.output_dir.clone()) {
                    eprintln!("  ⚠️  Failed to register published article: {}", e);
                } else {
                    println!("  ✅ Registered in article registry");
                }
                
                // Deletar PDF imediatamente após processar
                let pdf_path_to_remove = pdf_path.clone();
                if let Err(e) = std::fs::remove_file(&pdf_path_to_remove) {
                    eprintln!("  ⚠️  Failed to delete PDF {}: {}", pdf_path_to_remove.display(), e);
                } else {
                    println!("  🗑️  PDF deleted: {} (content saved in registry)", pdf_path_to_remove.display());
                }
            }
            Err(e) => {
                println!("  ❌ Error: {}\n", e);
            }
        }
    }
    
    println!("✅ Writer pipeline completed!");
    println!("   Output: G:\\Hive-Hub\\News-main\\output\\{}\\", site);
    
    Ok(())
}


fn scan_filtered_directory(base_dir: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut pdfs = Vec::new();
    
    if !base_dir.exists() {
        return Ok(pdfs);
    }
    
    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            for sub_entry in std::fs::read_dir(path)? {
                let sub_path = sub_entry?.path();
                if sub_path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                    pdfs.push(sub_path);
                }
            }
        }
    }
    
    Ok(pdfs)
}

fn run_registry_migration() -> anyhow::Result<()> {
    use crate::utils::article_registry::{ArticleRegistry, ArticleMetadata, ArticleStatus};
    use std::path::Path;
    use std::fs;
    use chrono::Utc;

    let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
    
    // Carregar registry existente (ou criar novo)
    let mut registry = match ArticleRegistry::load(registry_path) {
        Ok(r) => r,
        Err(_) => {
            println!("   Creating new registry...");
            ArticleRegistry {
                articles: std::collections::HashMap::new(),
            }
        }
    };

    println!("📂 Scanning existing articles...\n");

    let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
    let output_dir = Path::new("G:/Hive-Hub/News-main/output/AIResearch");
    
    let mut migrated_count = 0;
    let mut skipped_count = 0;

    // 1. Migrar artigos de output/ (já publicados)
    if output_dir.exists() {
        println!("📄 Scanning published articles in output/AIResearch/...");
        
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let article_dir = entry.path();
            
            if article_dir.is_dir() {
                let article_id = article_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Verificar se já tem article.md (publicado)
                let article_file = article_dir.join("article.md");
                if article_file.exists() {
                    // Verificar se título existe
                    let title_file = article_dir.join("title.txt");
                    let title = if title_file.exists() {
                        fs::read_to_string(&title_file).unwrap_or_else(|_| "Untitled".to_string())
                    } else {
                        "Untitled".to_string()
                    };

                    let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);

                    // Verificar se já está no registry
                    if !registry.is_article_registered(article_id) {
                        println!("  ✅ Migrating published: {} - {}", article_id, title);
                        
                        // Criar metadata completo
                        let mut metadata = ArticleMetadata {
                            id: article_id.to_string(),
                            title: title.trim().to_string(),
                            arxiv_url: arxiv_url.clone(),
                            pdf_url: pdf_url.clone(),
                            status: ArticleStatus::Published,
                            filter_score: None,
                            category: None,
                            rejection_reason: None,
                            collected_at: None,
                            filtered_at: None,
                            rejected_at: None,
                            published_at: Some(Utc::now()),
                            output_dir: Some(article_dir.clone()),
                            hidden: Some(false),
                            destinations: None,
                            featured: None,
                        };

                        // Tentar ler categoria se existir
                        let category_file = article_dir.join("image_categories.txt");
                        if category_file.exists() {
                            if let Ok(cats) = fs::read_to_string(&category_file) {
                                if let Some(first_cat) = cats.lines().next() {
                                    metadata.category = Some(first_cat.trim().to_string());
                                }
                            }
                        }

                        registry.articles.insert(article_id.to_string(), metadata);
                        migrated_count += 1;
                    } else {
                        skipped_count += 1;
                    }
                }
            }
        }
    }

    // 2. Migrar artigos de downloads/filtered/ (filtrados mas não publicados)
    let filtered_dir = base_dir.join("filtered");
    if filtered_dir.exists() {
        println!("\n🔍 Scanning filtered articles in downloads/filtered/...");
        
        for entry in fs::read_dir(&filtered_dir)? {
            let entry = entry?;
            let category_dir = entry.path();
            
            if category_dir.is_dir() {
                let category = category_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                for pdf_entry in fs::read_dir(&category_dir)? {
                    let pdf_entry = pdf_entry?;
                    let pdf_path = pdf_entry.path();
                    
                    if pdf_path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                        let article_id = pdf_path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        
                        if !registry.is_article_registered(article_id) {
                            println!("  ✅ Migrating filtered: {} → {}", article_id, category);
                            
                            let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                            let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);
                            
                            let metadata = ArticleMetadata {
                                id: article_id.to_string(),
                                title: "Untitled (from migration)".to_string(),
                                arxiv_url,
                                pdf_url,
                                status: ArticleStatus::Filtered,
                                filter_score: None,
                                category: Some(category.to_string()),
                                rejection_reason: None,
                                collected_at: None,
                                filtered_at: Some(Utc::now()),
                                rejected_at: None,
                                published_at: None,
                                output_dir: None,
                                hidden: Some(false),
                                destinations: None,
                                featured: None,
                            };
                            
                            registry.articles.insert(article_id.to_string(), metadata);
                            migrated_count += 1;
                        } else {
                            skipped_count += 1;
                        }
                    }
                }
            }
        }
    }

    // 3. Migrar artigos de downloads/rejected/ (rejeitados)
    let rejected_dir = base_dir.join("rejected");
    if rejected_dir.exists() {
        println!("\n❌ Scanning rejected articles in downloads/rejected/...");
        
        for entry in fs::read_dir(&rejected_dir)? {
            let entry = entry?;
            let pdf_path = entry.path();
            
            if pdf_path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                let article_id = pdf_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if !registry.is_article_registered(article_id) {
                    println!("  ✅ Migrating rejected: {}", article_id);
                    
                    let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);
                    
                    let metadata = ArticleMetadata {
                        id: article_id.to_string(),
                        title: "Untitled (from migration)".to_string(),
                        arxiv_url,
                        pdf_url,
                        status: ArticleStatus::Rejected,
                        filter_score: None,
                        category: None,
                        rejection_reason: Some("Migrated from rejected/".to_string()),
                        collected_at: None,
                        filtered_at: None,
                        rejected_at: Some(Utc::now()),
                        published_at: None,
                        output_dir: None,
                        hidden: Some(false),
                        destinations: None,
                        featured: None,
                    };
                    
                    registry.articles.insert(article_id.to_string(), metadata);
                    migrated_count += 1;
                } else {
                    skipped_count += 1;
                }
            }
        }
    }

    // Salvar registry
    println!("\n💾 Saving registry...");
    registry.save(registry_path)?;

    println!("\n✅ Migration completed!");
    println!("   Migrated: {} articles", migrated_count);
    println!("   Skipped (already in registry): {} articles", skipped_count);
    println!("   Total in registry: {} articles", registry.articles.len());

    Ok(())
}

async fn run_registry_enrichment() -> anyhow::Result<()> {
    use crate::utils::article_registry::{RegistryManager, ArticleStatus, ArticleRegistry};
    use crate::filter::{parser::parse_pdf, experiments::has_experimental_sections, 
                        fake_detector::calculate_fake_penalty, validator::validate_dois,
                        authors::validate_authors, scorer::{FilterResult, calculate_score},
                        categorizer::categorize};
    use std::path::Path;
    use std::fs;

    let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
    let registry = RegistryManager::new(registry_path)?;

    println!("📂 Scanning registry for incomplete articles...\n");

    let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
    let mut enriched_count = 0;
    let mut not_found_count = 0;

    // 1. Processar artigos rejeitados
    let rejected_articles = registry.list_by_status(ArticleStatus::Rejected);
    println!("❌ Processing {} rejected articles...", rejected_articles.len());

    for article in &rejected_articles {
        // Verificar se precisa enriquecer
        let needs_enrichment = article.title == "Untitled (from migration)" || article.filter_score.is_none();

        if !needs_enrichment {
            continue;
        }

        // Procurar PDF em rejected/
        let pdf_path = base_dir.join("rejected").join(format!("{}.pdf", article.id));

        if !pdf_path.exists() {
            println!("  ⚠️  PDF not found: {}", article.id);
            not_found_count += 1;
            continue;
        }

        println!("  🔄 Processing: {}", article.id);

        // Parse do PDF
        let parsed = match parse_pdf(&pdf_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("     ❌ Failed to parse: {}", e);
                continue;
            }
        };

        let title = parsed.title.trim().to_string();
        
        // Sempre tentar categorizar primeiro, mesmo se título estiver vazio
        let category = categorize(&parsed);
        
        // Atualizar categoria mesmo se não passar nos filtros
        {
            let mut reg = ArticleRegistry::load(registry_path)?;
            if let Some(metadata) = reg.articles.get_mut(&article.id) {
                if metadata.category.is_none() {
                    metadata.category = Some(category.clone());
                }
                if !title.is_empty() && title != "Untitled" && metadata.title == "Untitled (from migration)" {
                    metadata.title = title.clone();
                }
                if metadata.rejected_at.is_none() {
                    metadata.rejected_at = Some(chrono::Utc::now());
                }
                reg.save(registry_path)?;
            }
        }
        
        if title.is_empty() || title == "Untitled" {
            println!("     ✅ Updated category only: {} - Category: {}", article.id, category);
            enriched_count += 1;
            continue;
        }

        // Calcular score completo
        let has_tests = has_experimental_sections(&parsed);
        let fake_penalty = calculate_fake_penalty(&parsed.text);

        if !has_tests || fake_penalty > 0.5 {
            continue;
        }

        let doi_ratio = validate_dois(&parsed.dois).await;
        let author_ratio = validate_authors(&parsed.authors).await;

        let result = FilterResult {
            doc: parsed,
            doi_ratio,
            author_ratio,
            has_exp: has_tests,
            fake_penalty,
        };

        let score = calculate_score(&result);

        // Atualizar registry
        let mut reg = ArticleRegistry::load(registry_path)?;
        if let Some(metadata) = reg.articles.get_mut(&article.id) {
            metadata.title = title.clone();
            metadata.filter_score = Some(score as f64);
            metadata.category = Some(category.clone()); // Sempre adicionar categoria
            if metadata.rejection_reason.is_none() || metadata.rejection_reason.as_ref().unwrap() == "Migrated from rejected/" {
                metadata.rejection_reason = Some(format!("Score {:.2} below threshold 0.4", score));
            }
            // Preencher datas null com data atual
            if metadata.rejected_at.is_none() {
                metadata.rejected_at = Some(chrono::Utc::now());
            }
        }
        reg.save(registry_path)?;

        println!("     ✅ Updated: {} - Score: {:.2}, Category: {}", title, score, category);
        enriched_count += 1;
    }

    // 2. Processar artigos filtrados
    let filtered_articles = registry.list_by_status(ArticleStatus::Filtered);
    println!("\n🔍 Processing {} filtered articles...", filtered_articles.len());

    for article in &filtered_articles {
        // Verificar se precisa enriquecer
        let needs_enrichment = article.title == "Untitled (from migration)" 
            || article.filter_score.is_none() 
            || article.category.is_none();

        if !needs_enrichment {
            continue;
        }

        // Procurar PDF em filtered/<category>/ ou qualquer categoria
        let filtered_base = base_dir.join("filtered");
        let pdf_path = find_pdf_by_id_in_filtered(&filtered_base, &article.id);

        if pdf_path.is_none() {
            println!("  ⚠️  PDF not found: {}", article.id);
            not_found_count += 1;
            continue;
        }

        let pdf_path = pdf_path.unwrap();

        println!("  🔄 Processing: {}", article.id);

        // Parse do PDF
        let parsed = match parse_pdf(&pdf_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("     ❌ Failed to parse: {}", e);
                continue;
            }
        };

        let title = parsed.title.trim().to_string();
        if title.is_empty() || title == "Untitled" {
            continue;
        }

        // Calcular score e categoria (categoria primeiro, antes de mover parsed)
        let has_tests = has_experimental_sections(&parsed);
        let fake_penalty = calculate_fake_penalty(&parsed.text);
        let category = categorize(&parsed); // Chamar antes de mover parsed

        if !has_tests || fake_penalty > 0.5 {
            continue;
        }

        let doi_ratio = validate_dois(&parsed.dois).await;
        let author_ratio = validate_authors(&parsed.authors).await;

        let result = FilterResult {
            doc: parsed,
            doi_ratio,
            author_ratio,
            has_exp: has_tests,
            fake_penalty,
        };

        let score = calculate_score(&result);

        // Atualizar registry
        let mut reg = ArticleRegistry::load(registry_path)?;
        if let Some(metadata) = reg.articles.get_mut(&article.id) {
            metadata.title = title.clone();
            metadata.filter_score = Some(score as f64);
            metadata.category = Some(category.clone());
            // Preencher datas null com data atual
            if metadata.filtered_at.is_none() {
                metadata.filtered_at = Some(chrono::Utc::now());
            }
        }
        reg.save(registry_path)?;

        println!("     ✅ Updated: {} - Score: {:.2}, Category: {}", title, score, category);
        enriched_count += 1;
    }

    // 3. Processar artigos publicados que têm filter_score null
    let published_articles = registry.list_by_status(ArticleStatus::Published);
    println!("\n📄 Processing {} published articles with missing info...", published_articles.len());

    let output_dir = Path::new("G:/Hive-Hub/News-main/output/AIResearch");

    for article in &published_articles {
        // Verificar se precisa enriquecer
        let needs_enrichment = article.filter_score.is_none() || article.category.is_none() || article.collected_at.is_none();

        if !needs_enrichment {
            continue;
        }

        // Tentar buscar informação do output/ (já processado)
        let article_dir = output_dir.join(&article.id);
        let title_file = article_dir.join("title.txt");
        
        // Se tem output_dir, tentar ler categoria do arquivo image_categories.txt
        if article_dir.exists() {
            let category_file = article_dir.join("image_categories.txt");
            let mut reg = ArticleRegistry::load(registry_path)?;
            if let Some(metadata) = reg.articles.get_mut(&article.id) {
                let mut updated = false;
                
                // Ler primeira categoria se disponível
                if category_file.exists() {
                    if let Ok(cats) = fs::read_to_string(&category_file) {
                        if let Some(first_cat) = cats.lines().next() {
                            if metadata.category.is_none() {
                                metadata.category = Some(first_cat.trim().to_string());
                                updated = true;
                            }
                        }
                    }
                }
                
                // Se ainda não tem título, ler do title.txt
                if metadata.title.is_empty() || metadata.title == "Untitled" {
                    if let Ok(title) = fs::read_to_string(&title_file) {
                        metadata.title = title.trim().to_string();
                        updated = true;
                    }
                }
                
                // Preencher datas null com data atual
                if metadata.collected_at.is_none() {
                    metadata.collected_at = Some(chrono::Utc::now());
                    updated = true;
                }
                if metadata.filtered_at.is_none() {
                    metadata.filtered_at = Some(chrono::Utc::now());
                    updated = true;
                }
                if metadata.published_at.is_none() {
                    metadata.published_at = Some(chrono::Utc::now());
                    updated = true;
                }
                
                if updated {
                    reg.save(registry_path)?;
                    println!("  ✅ Enriched from output/: {}", article.id);
                    enriched_count += 1;
                }
            }
        }
    }

    // 4. Passo final: preencher categorias e datas faltantes para TODOS os artigos
    println!("\n📋 Final pass: filling missing categories and dates for all articles...");
    
    let mut reg = ArticleRegistry::load(registry_path)?;
    let now = chrono::Utc::now();
    
    for (id, metadata) in reg.articles.iter_mut() {
        let mut updated = false;
        
        // Preencher categoria null baseada no status ou usar "ai" como padrão
        if metadata.category.is_none() {
            // Se for Published, tentar ler do output
            if metadata.status == ArticleStatus::Published {
                let article_dir = output_dir.join(id);
                let category_file = article_dir.join("image_categories.txt");
                if category_file.exists() {
                    if let Ok(cats) = fs::read_to_string(&category_file) {
                        if let Some(first_cat) = cats.lines().next() {
                            metadata.category = Some(first_cat.trim().to_string());
                            updated = true;
                        }
                    }
                }
            }
            // Se ainda não tem, usar "ai" como padrão
            if metadata.category.is_none() {
                metadata.category = Some("ai".to_string());
                updated = true;
            }
        }
        
        // Preencher todas as datas null
        if metadata.collected_at.is_none() {
            metadata.collected_at = Some(now);
            updated = true;
        }
        if metadata.filtered_at.is_none() && (metadata.status == ArticleStatus::Filtered || metadata.status == ArticleStatus::Published) {
            metadata.filtered_at = Some(now);
            updated = true;
        }
        if metadata.rejected_at.is_none() && metadata.status == ArticleStatus::Rejected {
            metadata.rejected_at = Some(now);
            updated = true;
        }
        if metadata.published_at.is_none() && metadata.status == ArticleStatus::Published {
            metadata.published_at = Some(now);
            updated = true;
        }
        
        if updated {
            enriched_count += 1;
        }
    }
    
    reg.save(registry_path)?;
    
    println!("\n✅ Enrichment completed!");
    println!("   Enriched: {} articles", enriched_count);
    println!("   Not found: {} articles", not_found_count);

    Ok(())
}

fn find_pdf_by_id_in_filtered(base_dir: &Path, article_id: &str) -> Option<std::path::PathBuf> {
    use std::fs;
    
    if !base_dir.exists() {
        return None;
    }

    // Procurar em cada subdiretório (categoria)
    for entry in fs::read_dir(base_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        
        if path.is_dir() {
            // Verificar se o PDF existe nesta categoria
            let pdf_path = path.join(format!("{}.pdf", article_id));
            if pdf_path.exists() {
                return Some(pdf_path);
            }
        }
    }

    None
}
