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

    // Check if we should run a collection test, filter, or writer
    let args: Vec<String> = std::env::args().collect();
    let test_collector = args.len() > 1 && args[1] == "collect";
    let test_filter = args.len() > 1 && args[1] == "filter";
    let test_writer = args.len() > 1 && args[1] == "write";

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

    // Função para coleta direta do arXiv sem banco
    async fn run_arxiv_collection_direct() -> anyhow::Result<()> {
        use crate::collectors::arxiv_collector::ArxivCollector;
        use crate::models::raw_document::ArticleMetadata;
        use std::path::Path;

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
        let target_count = 10;
        let mut downloaded_count = 0;

        // Loop até baixar 20 novos artigos
        while downloaded_count < target_count {
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

            let response = client.get(&url).send().await?;
            let xml = response.text().await?;
            
            // Debug: verificar se recebemos dados válidos
            println!("  Response length: {} bytes", xml.len());
            if xml.len() < 100 {
                println!("  ⚠️  Warning: Very short response, might be an error page");
            }

            // Parse básico do XML
            let mut current_id = None;
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
                                if let Some(pos) = paper_id.rfind('v') {
                                    paper_id = paper_id[..pos].to_string();
                                }
                                current_id = Some(paper_id);
                            }
                        }
                    }
                }
                if let Some(id) = &current_id {
                    if line.contains("<title>") {
                        articles.push(ArticleMetadata {
                            id: id.clone(),
                            title: "Untitled".to_string(),
                            url: format!("https://arxiv.org/abs/{}", id),
                            author: Some("Unknown".to_string()),
                            summary: Some("No summary available".to_string()),
                            published_date: Some(chrono::Utc::now()),
                        });
                        current_id = None;
                    }
                }
            }

            println!("  Found {} papers in this batch", articles.len());

            // Tentar baixar artigos não duplicados
            for (i, article) in articles.iter().enumerate() {
                if downloaded_count >= target_count {
                    break;
                }

                let file_path = date_dir.join(format!("{}.pdf", article.id));

                // Verificar se já existe em qualquer lugar (arxiv, filtered, rejected)
                if file_already_downloaded(&article.id, base_dir) {
                    println!(
                        "  [{}/{}]: {}... ⏭️  (already exists)",
                        downloaded_count + 1,
                        target_count,
                        article.id
                    );
                    continue;
                }

                // Baixar (use published ID without version suffix)
                // Usar a API REST oficial do arXiv para baixar PDFs
                let pdf_url = format!("https://export.arxiv.org/pdf/{}.pdf", article.id);
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
                                                downloaded_count += 1;
                                                println!("✅ NEW");
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

            // Se não baixou nenhum novo, incrementar offset
            if downloaded_count < target_count {
                start_offset += 10;
                println!("  No new papers, trying next batch...\n");
            }

            // Safety: não ficar em loop infinito
            if start_offset > 100 {
                println!("⚠️  Reached max offset (100), stopping search");
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

        Ok(())
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

    // Connect to database
    let db = Database::new().await?;

    // Build application
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/auth/me", get(routes::auth::get_me))
        .route("/api/pages", get(routes::pages::list_pages))
        .route("/api/pages", post(routes::pages::create_page))
        .route("/api/pages/:id", get(routes::pages::get_page))
        .route("/api/pages/:id", put(routes::pages::update_page))
        .route("/api/pages/:id", delete(routes::pages::delete_page))
        .route("/api/sources", get(routes::sources::list_sources))
        .route("/api/sources", post(routes::sources::create_source))
        .route("/api/logs", get(routes::logs::list_logs))
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Server listening on http://localhost:3001");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn run_writer_pipeline() -> anyhow::Result<()> {
    use crate::writer::WriterService;
    use std::path::Path;
    
    let writer = WriterService::new()?;
    
    // Scan filtered directory for approved PDFs
    let filtered_dir = Path::new("G:/Hive-Hub/News-main/downloads/filtered");
    let all_approved_pdfs = scan_filtered_directory(filtered_dir)?;
    
    println!("📄 Found {} approved documents in filtered/\n", all_approved_pdfs.len());
    
    if all_approved_pdfs.is_empty() {
        println!("⚠️  No filtered PDFs found in downloads/filtered/");
        println!("   Run collector first to generate content");
        return Ok(());
    }
    
    // Filtrar apenas PDFs ainda não processados para este site
    let mut pending_pdfs = Vec::new();
    let site = writer.get_site();
    let output_base = writer.get_output_base();
    
    for pdf_path in all_approved_pdfs.iter() {
        let article_id = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let output_dir = output_base.join(&site).join(article_id);
        
        // Verificar se já existe conteúdo gerado
        if !article_already_processed(&output_dir) {
            pending_pdfs.push(pdf_path.clone());
        } else {
            println!("⏭️  Skipping {} (already processed for {})", article_id, site);
        }
    }
    
    println!("📝 {} new documents to process for {}\n", pending_pdfs.len(), site);
    
    if pending_pdfs.is_empty() {
        println!("✅ All documents already processed for {}", site);
        return Ok(());
    }
    
    for (i, pdf_path) in pending_pdfs.iter().enumerate() {
        let filename = pdf_path.file_name().unwrap().to_string_lossy();
        println!("[{}/{}] Processing: {}", i + 1, pending_pdfs.len(), filename);
        println!("  Phase 1: Generating article (Nature/Science style)...");
        
        match writer.process_pdf(pdf_path).await {
            Ok(result) => {
                println!("  ✅ Content saved → {}", result.output_dir.display());
                println!("     Tokens: {} → {} ({:.1}% savings)\n", 
                         result.original_tokens,
                         result.compressed_tokens,
                         result.compression_ratio * 100.0);
            }
            Err(e) => {
                println!("  ❌ Error: {}\n", e);
            }
        }
    }
    
    println!("✅ Writer pipeline completed!");
    println!("   Output: G:\\Hive-Hub\\News-main\\output\\news\\");
    
    Ok(())
}

/// Verifica se um artigo já foi processado pelo Writer
fn article_already_processed(output_dir: &Path) -> bool {
    // Verifica se o diretório existe
    if !output_dir.exists() {
        return false;
    }
    
    // Verifica se o arquivo article.md existe (arquivo principal gerado)
    let article_file = output_dir.join("article.md");
    article_file.exists()
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
