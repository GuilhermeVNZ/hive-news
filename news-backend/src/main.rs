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
                eprintln!("✅ Loaded .env from {}", path.display());
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
    let test_filter = args.len() > 1 && args[1] == "filter";
    let test_writer = args.len() > 1 && args[1] == "write";
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

        println!("⬇️  Downloading PDFs...\n");

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

        let mut start_offset = 0;
        let target_count = 20;
        let mut downloaded_count = 0;
        
        // Safe guards para evitar ban da API
        let max_api_requests = 50; // Máximo de requisições por ciclo
        let max_consecutive_empty = 5; // Máximo de batches vazios consecutivos antes de parar (aumentado para permitir gaps)
        let mut api_request_count = 0;
        let mut consecutive_empty_batches = 0;
        let mut last_successful_offset = None;
        let mut batches_with_articles_seen = 0; // Contador de batches com artigos encontrados

        // Loop até baixar 10 novos artigos (busca regressiva até encontrar)
        while downloaded_count < target_count {
            // Safe guard: limite de requisições por ciclo
            if api_request_count >= max_api_requests {
                println!("⚠️  Reached maximum API requests limit ({}), stopping search", max_api_requests);
                println!("   Found {} new papers (target was {})", downloaded_count, target_count);
                break;
            }
            
            println!("📡 Fetching batch starting from offset {}...", start_offset);

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

            // Parse básico do XML com extração de título
            let mut current_id = None;
            let mut current_title = None;
            let mut articles = Vec::new();

            for line in xml.lines() {
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
                if line.contains("<title>") && line.contains("</title>") {
                    if let Some(start) = line.find("<title>") {
                        if let Some(end) = line.find("</title>") {
                            let title = line[start + 7..end].trim().to_string();
                            current_title = Some(title);
                        }
                    }
                }
                if line.contains("</entry>") {
                    if let Some(id) = current_id.take() {
                        let title = current_title.take().unwrap_or_else(|| "Untitled".to_string());
                        articles.push(ArticleMetadata {
                            id: id.clone(),
                            title: title.clone(),
                            url: format!("https://arxiv.org/abs/{}", id),
                            author: Some("Unknown".to_string()),
                            summary: Some("No summary available".to_string()),
                            published_date: Some(chrono::Utc::now()),
                        });
                    }
                }
            }

            println!("  Found {} papers in this batch", articles.len());
            
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
                last_successful_offset = Some(start_offset);
                batches_with_articles_seen += 1;
            }

            // Tentar baixar artigos não duplicados
            let mut found_new_in_batch = false;
            for (i, article) in articles.iter().enumerate() {
                if downloaded_count >= target_count {
                    break;
                }

                let file_path = date_dir.join(format!("{}.pdf", article.id));

                // Verificar se já foi processado usando registry
                let is_registered = registry.is_article_registered(&article.id);
                if is_registered {
                    // Debug: mostrar detalhes da verificação
                    if api_request_count <= 2 {
                        let metadata = registry.get_metadata(&article.id);
                        if let Some(meta) = metadata {
                            println!(
                                "  [{}/{}]: {}... ⏭️  (already in registry - status: {:?})",
                                downloaded_count + 1,
                                target_count,
                                article.id,
                                meta.status
                            );
                        } else {
                            println!(
                                "  [{}/{}]: {}... ⏭️  (already in registry - metadata not found)",
                                downloaded_count + 1,
                                target_count,
                                article.id
                            );
                        }
                    } else {
                        println!(
                            "  [{}/{}]: {}... ⏭️  (already in registry)",
                            downloaded_count + 1,
                            target_count,
                            article.id
                        );
                    }
                    continue;
                }
                
                found_new_in_batch = true;

                // Baixar (use published ID without version suffix)
                // Usar a API REST oficial do arXiv para baixar PDFs
                let pdf_url = format!("https://export.arxiv.org/pdf/{}.pdf", article.id);
                let arxiv_url = format!("https://arxiv.org/abs/{}", article.id);
                print!(
                    "  [{}/{}]: {}... ",
                    downloaded_count + 1,
                    target_count,
                    article.id
                );

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

                match request.send().await {
                    Ok(response) => {
                        // Verificar se é uma resposta de sucesso
                        if response.status().is_success() {
                            let bytes = response.bytes().await;
                            match bytes {
                                Ok(b) => {
                                    // Verify it's actually a PDF (starts with %PDF)
                                    if b.len() > 4 && &b[0..4] == b"%PDF" {
                                        match tokio::fs::write(&file_path, &b).await {
                                            Ok(_) => {
                                                // Registrar no registry após download bem-sucedido
                                                if let Err(e) = registry.register_collected(
                                                    article.id.clone(),
                                                    article.title.clone(),
                                                    arxiv_url.clone(),
                                                    pdf_url.clone(),
                                                ) {
                                                    eprintln!("  ⚠️  Failed to register article: {}", e);
                                                } else {
                                                    // Define destinos com base nos sites que têm arXiv habilitado
                                                    let destinations = get_enabled_sites_for_source("arxiv");
                                                    if let Err(e) = registry.set_destinations(&article.id, destinations) {
                                                        eprintln!("  ⚠️  Failed to set destinations: {}", e);
                                                    }
                                                    downloaded_count += 1;
                                                    println!("✅ NEW (registered)");
                                                }
                                            }
                                            Err(e) => println!("❌ Write: {}", e),
                                        }
                                    } else {
                                        println!("❌ Invalid PDF (got HTML or redirect)");
                                    }
                                }
                                Err(e) => println!("❌ Download: {}", e),
                            }
                        } else {
                            println!("❌ HTTP Error: {}", response.status());
                        }
                    }
                    Err(e) => println!("❌ Request: {}", e),
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
        use serde_json::{json, Value};

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
        use serde_json::{json, Value};

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

        // Ler do collectors_config.json que é atualizado pelo dashboard
        // Tentar múltiplos caminhos possíveis
        let possible_paths = vec![
            Path::new("collectors_config.json"),
            Path::new("G:/Hive-Hub/News-main/news-backend/collectors_config.json"),
            Path::new("G:/Hive-Hub/News-main/collectors_config.json"),
        ];
        
        let mut config = None;
        let mut config_path_used = None;
        for path in &possible_paths {
            if path.exists() {
                let manager = ConfigManager::new(path);
                match manager.load() {
                    Ok(c) => {
                        config = Some(c);
                        config_path_used = Some(path.to_path_buf());
                        break;
                    }
                    Err(_) => continue,
                }
            }
        }
        
        let config = match config {
            Some(c) => c,
            None => {
                anyhow::bail!("collectors_config.json not found. Please configure collectors via dashboard first.");
            }
        };
        
        if let Some(path) = config_path_used {
            println!("  Using config from: {}", path.display());
        }

        // Determinar fontes habilitadas a partir dos collectors habilitados
        let mut use_arxiv = false;
        let mut use_pmc = false;
        let mut use_ss = false;

        for c in config.collectors {
            let id = c.id.to_lowercase();
            if !c.enabled { continue; }
            if id.contains("arxiv") { use_arxiv = true; }
            if id.contains("pmc") || id.contains("pubmed") { use_pmc = true; }
            if id.contains("semantic") { use_ss = true; }
        }
        println!("Enabled sources summary:");
        println!("  arXiv:   {}", if use_arxiv { "ON" } else { "OFF" });
        println!("  PMC:     {}", if use_pmc { "ON" } else { "OFF" });
        println!("  Semantic:{}", if use_ss { "ON" } else { "OFF" });

        if use_arxiv { run_arxiv_collection_direct().await?; }
        if use_pmc { run_pmc_collection_direct().await?; }
        if use_ss { run_semantic_scholar_collection_direct().await?; }

        Ok(())
    }

    fn get_enabled_sites_for_source(source_key: &str) -> Vec<String> {
        use std::path::Path;
        use crate::utils::site_config_manager::SiteConfigManager;
        let config_path = Path::new("G:/Hive-Hub/News-main/news-backend/system_config.json");
        let manager = SiteConfigManager::new(config_path);
        let mut result = Vec::new();
        if let Ok(sites) = manager.get_all_sites() {
            for s in sites {
                let mut enabled_for_source = false;
                for c in s.collectors {
                    let id = c.id.to_lowercase();
                    if !c.enabled { continue; }
                    if source_key == "arxiv" && id.contains("arxiv") { enabled_for_source = true; }
                    if source_key == "pmc" && (id.contains("pmc") || id.contains("pubmed")) { enabled_for_source = true; }
                    if source_key == "semantic" && id.contains("semantic") { enabled_for_source = true; }
                }
                if enabled_for_source { result.push(s.id); }
            }
        }
        result
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
    let output_base = writer.get_output_base();
    
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
                if let Err(e) = std::fs::remove_file(pdf_path) {
                    eprintln!("  ⚠️  Failed to delete PDF: {}", e);
                } else {
                    println!("  🗑️  PDF deleted (content saved in registry)\n");
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
    use std::path::{Path, PathBuf};
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
