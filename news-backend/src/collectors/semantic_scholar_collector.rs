// Semantic Scholar Collector
// Academic search engine for AI/Computer Science papers

use reqwest::Client;
use std::path::PathBuf;
use crate::models::raw_document::ArticleMetadata;
use anyhow::{Result, Context};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SemanticScholarCollector {
    client: Client,
    temp_dir: PathBuf,
    api_key: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct SemanticScholarPaper {
    paperId: Option<String>,
    title: Option<String>,
    authors: Option<Vec<SemanticScholarAuthor>>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    #[allow(dead_code)]
    venue: Option<String>,
    year: Option<i32>,
    url: Option<String>,
    openAccessPdf: Option<OpenAccessPdf>,
    fieldsOfStudy: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarAuthor {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAccessPdf {
    url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    total: Option<usize>,
    data: Vec<SemanticScholarPaper>,
    offset: Option<usize>,
}

impl SemanticScholarCollector {
    pub fn new(temp_dir: PathBuf, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            temp_dir,
            api_key,
        }
    }

    /// Fetch recent papers filtered by technology topics only
    /// Filters: Computer Science, Artificial Intelligence, Machine Learning, etc.
    pub async fn fetch_recent_papers(
        &self,
        limit: usize,
        offset: usize,
        _categories: Option<Vec<String>>,
    ) -> Result<Vec<ArticleMetadata>> {
        println!("📥 Fetching recent papers from Semantic Scholar (Technology filter active)...");
        
        // Base URL da API do Semantic Scholar
        let base_url = "https://api.semanticscholar.org/graph/v1/paper/search";
        
        // Query de busca: Apenas artigos de tecnologia/Computer Science
        // Filtra automaticamente por campos de estudo relacionados a tecnologia
        let query = "computer science artificial intelligence machine learning deep learning";
        
        // Campos de estudo permitidos (apenas tecnologia)
        let fields_of_study = vec![
            "Computer Science",
            "Artificial Intelligence",
            "Machine Learning",
            "Computer Vision",
            "Natural Language Processing",
            "Data Science",
            "Software Engineering",
        ];
        
        // Construir query parameters (modo low-rate quando não há API key)
        let has_key = self.api_key.is_some();
        let effective_limit = if has_key { limit.min(20) } else { limit.min(5) };
        let limit_str = effective_limit.to_string();
        let offset_str = offset.to_string();
        let params = vec![
            ("query", query),
            ("limit", &limit_str),
            ("offset", &offset_str),
            ("fields", "paperId,title,authors,abstract,venue,year,url,openAccessPdf,fieldsOfStudy"),
            ("sort", "publicationDate:desc"),
        ];
        
        // Se API key disponível, adicionar header
        let mut request = self.client.get(base_url);
        
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        
        // Executar busca (atraso adicional se não houver chave para evitar 429)
        if !has_key { tokio::time::sleep(std::time::Duration::from_millis(3000)).await; }
        let response = request
            .query(&params)
            .send()
            .await
            .context("Failed to fetch from Semantic Scholar")?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("Semantic Scholar HTTP {}: {}", status, body.chars().take(200).collect::<String>());
            if status.as_u16() == 429 {
                // backoff gentil sem quebrar o ciclo
                let wait_ms = if has_key { 4000 } else { 8000 };
                println!("Semantic Scholar rate-limited (429). Waiting {} ms...", wait_ms);
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
            }
            return Ok(Vec::new());
        }

        // Tente parsear; se schema não corresponder (ex.: erro com campo 'message'), retorne vazio com log
        let text = response.text().await.context("Failed to read Semantic Scholar body")?;
        let search_response: Result<SemanticScholarResponse, _> = serde_json::from_str(&text);
        let search_response = match search_response {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Semantic Scholar parse error: {}. Body head: {}", e, text.chars().take(200).collect::<String>());
                if !has_key { tokio::time::sleep(std::time::Duration::from_millis(4000)).await; }
                return Ok(Vec::new());
            }
        };
        
        println!("Found {} papers in Semantic Scholar", search_response.data.len());
        
        // Filtrar por campos de estudo de tecnologia
        let mut articles = Vec::new();
        let allowed_fields: Vec<String> = fields_of_study.iter().map(|s| s.to_lowercase()).collect();
        
        for paper in search_response.data {
            // Verificar se tem campos de estudo relacionados a tecnologia
            let is_tech_paper = if let Some(ref fields) = paper.fieldsOfStudy {
                fields.iter().any(|field| {
                    let field_lower = field.to_lowercase();
                    allowed_fields.iter().any(|allowed| {
                        field_lower.eq_ignore_ascii_case(allowed) ||
                        field_lower.contains(allowed) ||
                        allowed.contains(&field_lower)
                    })
                })
            } else {
                // Se não tem fieldsOfStudy, verificar no título/abstract
                let title_lower = paper.title.as_ref().map(|t| t.to_lowercase()).unwrap_or_default();
                let abstract_lower = paper.abstract_text.as_ref().map(|a| a.to_lowercase()).unwrap_or_default();
                let combined = format!("{} {}", title_lower, abstract_lower);
                
                // Verificar palavras-chave de tecnologia no título/abstract
                combined.contains("artificial intelligence") ||
                combined.contains("machine learning") ||
                combined.contains("deep learning") ||
                combined.contains("neural network") ||
                combined.contains("computer vision") ||
                combined.contains("natural language processing") ||
                combined.contains("computer science") ||
                combined.contains("data science") ||
                combined.contains("algorithm") ||
                combined.contains("software engineering")
            };
            
            if !is_tech_paper {
                continue; // Pular artigos que não são de tecnologia
            }
            
            // Construir metadados do artigo
            let authors_str = paper.authors
                .as_ref()
                .map(|authors| {
                    authors.iter()
                        .filter_map(|a| a.name.as_ref().map(|s| s.as_str()))
                        .take(3)
                        .collect::<Vec<&str>>()
                        .join(", ")
                })
                .unwrap_or_default();
            
            let article_id = paper.paperId.as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| format!("SS_{}", chrono::Utc::now().timestamp()));
            
            let published_date = paper.year.and_then(|y| {
                chrono::NaiveDate::from_ymd_opt(y as i32, 1, 1)
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| chrono::DateTime::from_timestamp(dt.and_utc().timestamp(), 0))
                    .flatten()
            });
            
            let title = paper.title.unwrap_or_else(|| format!("Semantic Scholar Paper {}", article_id));
            articles.push(ArticleMetadata {
                id: article_id.clone(),
                title: title.clone(), // Mantido para compatibilidade
                original_title: Some(title), // Título original da fonte
                generated_title: None, // Será preenchido quando o artigo for publicado
                url: paper.url.unwrap_or_else(|| format!("https://www.semanticscholar.org/paper/{}", article_id)),
                author: if authors_str.is_empty() { None } else { Some(authors_str) },
                summary: paper.abstract_text,
                published_date,
                image_url: None,
                source_type: Some("semantic_scholar".to_string()),
                content_html: None,
                content_text: None,
                category: None,
                slug: None,
            });
        }
        
        println!("✅ Filtered to {} technology papers from Semantic Scholar", articles.len());
        if !has_key { tokio::time::sleep(std::time::Duration::from_millis(2000)).await; }
        
        Ok(articles)
    }
    
    pub async fn download_pdf(&self, article_id: &str, output_path: &PathBuf) -> Result<()> {
        // Primeiro, buscar o paper para obter o PDF URL
        let paper_url = format!("https://api.semanticscholar.org/graph/v1/paper/{}", article_id);
        
        let mut request = self.client.get(&paper_url)
            .query(&[("fields", "openAccessPdf")]);
        
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
        }
        
        let response = request
            .send()
            .await
            .context("Failed to fetch paper metadata from Semantic Scholar")?;
        
        let paper: SemanticScholarPaper = response
            .json()
            .await
            .context("Failed to parse paper metadata")?;
        
        // Verificar se tem PDF open access
        if let Some(pdf_info) = paper.openAccessPdf {
            if let Some(pdf_url) = pdf_info.url {
                let pdf_response = self.client
                    .get(&pdf_url)
                    .send()
                    .await
                    .context("Failed to download PDF from Semantic Scholar")?;
                
                if pdf_response.status().is_success() {
                    let bytes = pdf_response.bytes().await?;
                    tokio::fs::write(output_path, bytes).await?;
                    return Ok(());
                }
            }
        }
        
        // Se não tiver PDF open access, retornar erro
        Err(anyhow::anyhow!("PDF not available for open access: {}", article_id))
    }
}

