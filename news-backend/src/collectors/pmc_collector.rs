// PubMed Central Collector
// Alternative source when arXiv has issues

use reqwest::Client;
use std::path::PathBuf;
use crate::models::raw_document::ArticleMetadata;
use anyhow::{Result, Context};
use serde::{Deserialize};

#[derive(Debug)]
pub struct PmcCollector {
    client: Client,
    temp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PmcESearch {
    #[serde(rename = "esearchresult")]
    esearch_result: ESearchResult,
}

#[derive(Debug, Deserialize)]
struct ESearchResult {
    #[serde(rename = "count")]
    _count: Option<String>,
    #[serde(rename = "retmax")]
    _retmax: Option<String>,
    #[serde(rename = "retstart")]
    _retstart: Option<String>,
    #[serde(rename = "idlist")]
    id_list: Vec<String>,
}

impl PmcCollector {
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .expect("Failed to create PMC client"),
            temp_dir,
        }
    }

    pub async fn fetch_recent_papers(&self, limit: usize, retstart: usize) -> Result<Vec<ArticleMetadata>> {
        // PMC API - Technology/Computer Science papers only
        // Filtro abrangente de tecnologia aplicado
        let url = "https://www.ncbi.nlm.nih.gov/pmc/utils/idconv/v1.0/";
        
        println!("📥 Fetching recent papers from PubMed Central (Technology filter active)...");
        
        // Search query: Apenas artigos de tecnologia
        // Filtra por: AI, ML, Computer Science, Data Science, NLP, Computer Vision, etc.
        let tech_search_term = "(\"artificial intelligence\"[Title/Abstract] OR \
            \"machine learning\"[Title/Abstract] OR \
            \"deep learning\"[Title/Abstract] OR \
            \"neural network\"[Title/Abstract] OR \
            \"computer vision\"[Title/Abstract] OR \
            \"natural language processing\"[Title/Abstract] OR \
            \"NLP\"[Title/Abstract] OR \
            \"data science\"[Title/Abstract] OR \
            \"computer science\"[Title/Abstract] OR \
            \"software engineering\"[Title/Abstract] OR \
            \"programming\"[Title/Abstract] OR \
            \"algorithm\"[Title/Abstract] OR \
            \"deep neural network\"[Title/Abstract] OR \
            \"transformer\"[Title/Abstract] OR \
            \"reinforcement learning\"[Title/Abstract])";
        
        // Search for recent tech papers
        let esearch_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi";
        let limit_str = limit.to_string();
        let start_str = retstart.to_string();
        let params = vec![
            ("db", "pmc"),
            ("term", tech_search_term),
            ("retmode", "json"),
            ("retmax", &limit_str),
            ("retstart", &start_str),
            ("sort", "pub+date"),
            ("datetype", "pdat"),
            ("reldate", "365"), // últimos 365 dias para ampliar janela
        ];
        
        let response = self.client
            .get(esearch_url)
            .query(&params)
            .send()
            .await
            .context("Failed to fetch from PMC")?;
        
        let esearch: PmcESearch = response.json().await
            .context("Failed to parse PMC response")?;

        let ids = esearch.esearch_result.id_list;
        println!("Found {} papers in PMC", ids.len());
        
        let mut articles = Vec::new();
        
        // Convert PMC IDs to article metadata
        for pmc_id in ids.iter().take(limit) {
            articles.push(ArticleMetadata {
                id: format!("PMC{}", pmc_id),
                title: format!("Paper ID: {}", pmc_id),
                url: format!("https://www.ncbi.nlm.nih.gov/pmc/articles/PMC{}", pmc_id),
                author: Some("PMC Paper".to_string()),
                summary: Some("PubMed Central paper".to_string()),
                published_date: Some(chrono::Utc::now()),
            });
        }
        
        Ok(articles)
    }
    
    pub async fn download_pdf(&self, article_id: &str, output_path: &PathBuf) -> Result<()> {
        // Remover prefixo "PMC" se presente
        let pmc_id = article_id.strip_prefix("PMC").unwrap_or(article_id);
        
        // Tentar múltiplas URLs para evitar Proof of Work (POW)
        let urls = vec![
            // Opção 1: URL direta do PMC sem POW (usando oo-servidor)
            format!("https://www.ncbi.nlm.nih.gov/pmc/articles/PMC{}/pdf/", pmc_id),
            // Opção 2: URL alternativa do PMC
            format!("https://www.ncbi.nlm.nih.gov/pmc/articles/PMC{}/pdf/main.pdf", pmc_id),
            // Opção 3: FTP alternativo via HTTP (pode não funcionar, mas tenta)
            format!("https://ftp.ncbi.nlm.nih.gov/pub/pmc/{}/main.pdf", pmc_id),
        ];
        
        // Primeiro: estabelecer sessão inicial (como arXiv)
        println!("🔐 Establishing session with PMC...");
        match self.client
            .get("https://www.ncbi.nlm.nih.gov/pmc/")
            .send()
            .await
        {
            Ok(_) => println!("   Session established ✓"),
            Err(e) => println!("   Warning: Could not establish session: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Tentar cada URL até uma funcionar
        let mut last_error = None;
        for (i, url) in urls.iter().enumerate() {
            println!("   Trying URL {}: {}", i + 1, url);
            
            let response = self.client
                .get(url)
                .header("Accept", "application/pdf,application/octet-stream,*/*")
                .header("Referer", "https://www.ncbi.nlm.nih.gov/pmc/")
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let bytes = resp.bytes().await?;
                        
                        // Verificar se não é HTML (que indicaria POW)
                        if bytes.len() > 4 && &bytes[0..4] != b"<htm" && &bytes[0..4] != b"<!DO" {
                            // Parece ser um PDF válido
                            tokio::fs::write(output_path, &bytes).await?;
                            println!("   ✓ Downloaded successfully from URL {}", i + 1);
                            return Ok(());
                        } else {
                            println!("   ✗ Got HTML instead of PDF (POW challenge), trying next URL...");
                            last_error = Some(format!("URL {} returned HTML (POW challenge)", i + 1));
                            continue;
                        }
                    } else {
                        println!("   ✗ HTTP {}: {}", resp.status(), url);
                        last_error = Some(format!("HTTP {} from URL {}", resp.status(), i + 1));
                        // Continuar para próxima URL
                    }
                }
                Err(e) => {
                    println!("   ✗ Error: {}", e);
                    last_error = Some(format!("Request error: {}", e));
                    // Continuar para próxima URL
                }
            }
            
            // Delay entre tentativas
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
        
        // Se todas as URLs falharam, retornar erro
        Err(anyhow::anyhow!(
            "Failed to download PDF from PMC using all {} URLs. Last error: {}",
            urls.len(),
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }
}







