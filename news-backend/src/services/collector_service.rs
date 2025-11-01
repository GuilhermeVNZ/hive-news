use crate::collectors::arxiv_collector::ArxivCollector;
use crate::models::raw_document::*;
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

/// Serviço de coleta de documentos de fontes externas
#[allow(dead_code)]
pub struct CollectorService {
    db: PgPool,
    client: reqwest::Client,
    /// Diretório base para downloads: G:\Hive-Hub\News-main\downloads
    download_dir: PathBuf,
    /// Cliente para arXiv
    arxiv_collector: ArxivCollector,
}

#[allow(dead_code)]
impl CollectorService {
    /// Cria uma nova instância do CollectorService
    pub fn new(db: PgPool, download_dir: impl AsRef<Path>) -> Self {
        // Cliente HTTP com configurações de segurança para evitar reCAPTCHA
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .cookie_store(true)  // Manter sessão entre requisições (evita reCAPTCHA)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("Failed to create HTTP client");

        let temp_dir = PathBuf::from(download_dir.as_ref()).join("temp");
        let arxiv_collector = ArxivCollector::new(temp_dir);

        Self {
            db,
            client,
            download_dir: download_dir.as_ref().to_path_buf(),
            arxiv_collector,
        }
    }

    /// Coleta artigos para um portal específico
    ///
    /// Fluxo:
    /// 1. Busca configuração do portal no banco
    /// 2. Para cada source configurada, busca novos artigos
    /// 3. Faz download dos arquivos
    /// 4. Salva metadados no banco de dados
    pub async fn collect_for_portal(&self, portal_id: i32) -> Result<CollectionResult> {
        let start_time = std::time::Instant::now();
        let mut documents_collected = 0;
        let mut errors = Vec::new();

        info!(portal_id = %portal_id, "Starting collection for portal");

        // 1. Buscar configuração do portal
        let portal_name = self.get_portal_name(portal_id).await?;
        info!(portal_id = %portal_id, portal_name = %portal_name, "Portal configuration loaded");

        // 2. Buscar sources configuradas para este portal
        let sources = self.get_portal_sources(portal_id).await?;

        for source in sources {
            info!(source = %source.name, "Collecting from source");

            // TODO: Implementar lógica de coleta por API
            // Por enquanto, mock para demonstração
            match self.collect_from_source(&source.name, &portal_name).await {
                Ok(downloaded) => {
                    documents_collected += downloaded;
                }
                Err(e) => {
                    let error_msg = format!("Failed to collect from {}: {}", source.name, e);
                    error!(error = %error_msg, "Collection error");
                    errors.push(error_msg);
                }
            }
        }

        let duration = start_time.elapsed();

        info!(
            portal_id = %portal_id,
            documents_collected = documents_collected,
            duration_ms = duration.as_millis(),
            "Collection completed"
        );

        Ok(CollectionResult {
            success: errors.is_empty(),
            documents_collected,
            duration_ms: duration.as_millis() as i64,
            errors,
        })
    }

    /// Coleta documentos de uma fonte específica
    async fn collect_from_source(&self, source: &str, portal: &str) -> Result<i32> {
        let mut count = 0;

        // Mock: Simular busca de artigos
        let articles = self.fetch_articles_from_api(source).await?;

        for article in articles {
            match self.download_article(&article, portal, source).await {
                Ok(_) => {
                    count += 1;
                    info!(source = %source, title = %article.title, "Article collected");
                }
                Err(e) => {
                    warn!(source = %source, title = %article.title, error = %e, "Failed to download article");
                }
            }
        }

        Ok(count)
    }

    /// Busca artigos de uma API (implementado para arXiv)
    async fn fetch_articles_from_api(&self, source: &str) -> Result<Vec<ArticleMetadata>> {
        match source.to_lowercase().as_str() {
            "arxiv" => {
                info!(source = %source, "Fetching from arXiv API");
                // Buscar papers mais recentes de cs.AI (Computer Science - Artificial Intelligence)
                self.arxiv_collector.fetch_recent_papers("cs.AI", 10).await
            }
            _ => {
                warn!(source = %source, "Unknown source, using mock");
                Ok(vec![])
            }
        }
    }

    /// Faz download de um artigo
    ///
    /// Estrutura: downloads/<origem>/<YYYY-MM-DD>/<arquivo>.pdf
    /// Exemplo: downloads/arxiv/2025-10-27/article_001.pdf
    pub async fn download_article(
        &self,
        article: &ArticleMetadata,
        _portal: &str,
        source: &str,
    ) -> Result<PathBuf> {
        // Criar data no formato YYYY-MM-DD
        let date = Utc::now().format("%Y-%m-%d").to_string();

        // Criar estrutura de diretórios: downloads/<origem>/<YYYY-MM-DD>/
        let download_dir = self.download_dir.join(source).join(&date);
        tokio::fs::create_dir_all(&download_dir)
            .await
            .context("Failed to create download directory")?;

        // Gerar nome do arquivo baseado na URL
        let filename = sanitize_filename(&article.title).unwrap_or_else(|| {
            url::Url::parse(&article.url)
                .ok()
                .and_then(|u| {
                    u.path_segments()
                        .and_then(|segments| segments.last())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| format!("article_{}.pdf", article.id))
        });

        let file_path = download_dir.join(&filename);

        // Verificar se arquivo já existe
        if file_path.exists() {
            info!(
                file_path = %file_path.display(),
                "File already exists, skipping download"
            );
            return Ok(file_path);
        }

        // Fazer download do PDF
        let pdf_url = if source == "arxiv" {
            // arXiv: Usar export.arxiv.org (API oficial, evita reCAPTCHA)
            format!("https://export.arxiv.org/pdf/{}.pdf", article.id)
        } else {
            article.url.clone()
        };

        info!(url = %pdf_url, "Downloading PDF");
        
        // Criar requisição com headers de segurança para evitar reCAPTCHA
        let request = self.client
            .get(&pdf_url)
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
        
        let response = request.send().await?;
        
        // Verificar se é uma resposta de sucesso
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }
        
        let bytes = response.bytes().await?;
        let size = bytes.len();

        // Validar se é um PDF (verificar magic bytes)
        if bytes.len() > 4 && &bytes[0..4] != b"%PDF" {
            return Err(anyhow::anyhow!("Invalid PDF (got HTML or redirect, likely reCAPTCHA)"));
        }

        // Salvar arquivo
        tokio::fs::write(&file_path, bytes)
            .await
            .context("Failed to save downloaded file")?;

        info!(
            source = %source,
            pdf_url = %pdf_url,
            path = %file_path.display(),
            size = size,
            "Article downloaded successfully"
        );

        // Salvar metadados no banco
        let doc = CreateRawDocument {
            portal_id: 1, // TODO: Obter do contexto
            title: article.title.clone(),
            source_url: article.url.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            file_type: "pdf".to_string(),
            file_size: Some(size as i64),
            metadata: Some(serde_json::json!({
                "article_id": article.id,
                "author": article.author,
                "published_date": article.published_date,
            })),
        };

        self.save_document(&doc).await?;

        Ok(file_path)
    }

    /// Helper: Busca nome do portal no banco
    async fn get_portal_name(&self, portal_id: i32) -> Result<String> {
        // TODO: Implementar query real
        Ok(format!("Portal_{}", portal_id))
    }

    /// Helper: Busca sources configuradas para o portal
    async fn get_portal_sources(&self, _portal_id: i32) -> Result<Vec<SourceConfig>> {
        // Para teste: retorna arXiv configurado
        if _portal_id == 1 {
            Ok(vec![SourceConfig {
                name: "arxiv".to_string(),
                api_url: "https://export.arxiv.org/api/query".to_string(),
                api_key: None,
            }])
        } else {
            Ok(vec![])
        }
    }

    /// Coleta direta do arXiv (para testes) - SEM DEPENDÊNCIA DE BANCO
    pub async fn collect_arxiv_direct(&self) -> Result<CollectionResult> {
        info!("Starting direct arXiv collection (no database)");

        let articles = self
            .arxiv_collector
            .fetch_recent_papers("cs.AI", 10)
            .await?;
        let mut count = 0;
        let mut errors = Vec::new();

        println!("Found {} articles to download", articles.len());

        for (i, article) in articles.iter().enumerate() {
            print!("  {}/{}: {}... ", i + 1, articles.len(), article.id);
            match self.download_article(article, "AIResearch", "arxiv").await {
                Ok(_) => {
                    count += 1;
                    info!(title = %article.title, "Article downloaded");
                    println!("✅");
                }
                Err(e) => {
                    let error_msg = format!("Failed to download {}: {}", article.title, e);
                    println!("❌");
                    errors.push(error_msg);
                }
            }
            
            // Rate limiting: delay de 3 segundos entre downloads para evitar bloqueios
            if i < articles.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }

        Ok(CollectionResult {
            success: errors.is_empty(),
            documents_collected: count,
            duration_ms: 0,
            errors,
        })
    }

    /// Salva metadados no banco de dados
    pub async fn save_document(&self, doc: &CreateRawDocument) -> Result<i32> {
        // Por enquanto, apenas log sem banco de dados
        info!(
            title = %doc.title,
            file_path = %doc.file_path,
            source_url = %doc.source_url,
            "Document metadata saved"
        );

        // TODO: Implementar insert real no banco quando disponível
        Ok(1)
    }

    /// Busca documentos não processados
    pub async fn get_unprocessed_documents(&self) -> Result<Vec<RawDocument>> {
        // TODO: Implementar query real quando banco estiver disponível
        info!("Fetching unprocessed documents (mock)");
        Ok(vec![])
    }

    /// Marca documento como processado
    pub async fn mark_as_processed(&self, document_id: i32) -> Result<()> {
        // TODO: Implementar query real quando banco estiver disponível
        info!(document_id = %document_id, "Document marked as processed (mock)");
        Ok(())
    }
}

/// Configuração de source (placeholder)
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SourceConfig {
    name: String,
    api_url: String,
    api_key: Option<String>,
}

/// Sanitiza nome de arquivo para evitar caracteres inválidos
#[allow(dead_code)]
fn sanitize_filename(name: &str) -> Option<String> {
    let sanitized = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            ' ' | '\t' | '\n' | '\r' => '_',
            _ => '-',
        })
        .collect::<String>();

    Some(sanitized).filter(|s| !s.is_empty())
}
