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
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check if we should run a collection test or writer
    let args: Vec<String> = std::env::args().collect();
    let test_collector = args.len() > 1 && args[1] == "collect";
    let test_writer = args.len() > 1 && args[1] == "write";

    if test_collector {
        println!("🔬 Test Collector - arXiv");
        println!("=====================================\n");

        println!("📥 Starting collection from arXiv...");
        println!("   Category: cs.AI");
        println!("   Papers: 20\n");

        // Coleta direta SEM banco de dados
        run_arxiv_collection_direct().await?;

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

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        let mut start_offset = 0;
        let target_count = 20;
        let mut downloaded_count = 0;

        // Loop até baixar 20 novos artigos
        while downloaded_count < target_count {
            println!("📡 Fetching batch starting from offset {}...", start_offset);

            // Modificar a URL do arXiv collector para usar offset
            let url = format!(
                "https://export.arxiv.org/api/query?search_query=cat:cs.AI&start={}&max_results={}&sortBy=submittedDate&sortOrder=descending",
                start_offset,
                target_count * 2 // Buscar mais para garantir que achamos novos
            );

            let response = client.get(&url).send().await?;
            let xml = response.text().await?;

            // Parse básico do XML
            let mut current_id = None;
            let mut articles = Vec::new();

            for line in xml.lines() {
                if line.contains("<id>") {
                    if let Some(start) = line.find("<id>") {
                        if let Some(end) = line.find("</id>") {
                            let id = &line[start + 4..end];
                            if id.contains("arxiv.org/abs/") {
                                let paper_id = id
                                    .replace("http://arxiv.org/abs/", "")
                                    .replace("https://arxiv.org/abs/", "");
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

                // Baixar
                let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article.id);
                print!(
                    "  [{}/{}]: {}... ",
                    downloaded_count + 1,
                    target_count,
                    article.id
                );

                match client.get(&pdf_url).send().await {
                    Ok(response) => match response.bytes().await {
                        Ok(bytes) => match tokio::fs::write(&file_path, bytes).await {
                            Ok(_) => {
                                downloaded_count += 1;
                                println!("✅ NEW");
                            }
                            Err(e) => println!("❌ Write: {}", e),
                        },
                        Err(e) => println!("❌ Download: {}", e),
                    },
                    Err(e) => println!("❌ Request: {}", e),
                }
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
        println!("   New papers downloaded: {}/50", downloaded_count);
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
    let approved_pdfs = scan_filtered_directory(filtered_dir)?;
    
    println!("📄 Found {} approved documents to process\n", approved_pdfs.len());
    
    if approved_pdfs.is_empty() {
        println!("⚠️  No filtered PDFs found in downloads/filtered/");
        println!("   Run collector first to generate content");
        return Ok(());
    }
    
    for (i, pdf_path) in approved_pdfs.iter().enumerate() {
        let filename = pdf_path.file_name().unwrap().to_string_lossy();
        println!("[{}/{}] Processing: {}", i + 1, approved_pdfs.len(), filename);
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
