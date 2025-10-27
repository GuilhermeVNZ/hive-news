use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn, error};
use crate::models::raw_document::ArticleMetadata;

/// Cliente para coleta de artigos do arXiv
pub struct ArxivCollector {
    client: Client,
    temp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ArxivFeed {
    #[serde(rename = "feed")]
    feed: ArxivFeedContent,
}

#[derive(Debug, Deserialize)]
struct ArxivFeedContent {
    #[serde(rename = "entry")]
    entry: Vec<ArxivEntry>,
}

#[derive(Debug, Deserialize)]
struct ArxivEntry {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "summary")]
    summary: String,
    #[serde(rename = "published")]
    published: String,
    #[serde(rename = "author")]
    authors: Vec<ArxivAuthor>,
}

#[derive(Debug, Deserialize)]
struct ArxivAuthor {
    name: String,
}

impl ArxivCollector {
    /// Cria novo cliente arXiv
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .user_agent("News-System-Collector/1.0 (arXiv)")
                .build()
                .expect("Failed to create arXiv client"),
            temp_dir,
        }
    }

    /// Busca os 10 artigos mais recentes de uma categoria
    pub async fn fetch_recent_papers(&self, category: &str, max_results: u32) -> Result<Vec<ArticleMetadata>> {
        let url = format!(
            "https://export.arxiv.org/api/query?search_query=cat:{}&start=10&max_results={}&sortBy=submittedDate&sortOrder=descending",
            category,
            max_results
        );

        info!(url = %url, category = %category, "Fetching papers from arXiv");

        let response = self.client.get(&url).send().await?;
        let xml_content = response.text().await?;

        // Salvar XML temporário
        let temp_file = self.temp_dir.join(format!("arxiv_feed_{}.xml", chrono::Utc::now().timestamp()));
        tokio::fs::write(&temp_file, &xml_content).await
            .context("Failed to save arXiv feed XML")?;
        
        info!(temp_file = %temp_file.display(), "Saved arXiv feed XML");

        // Parse XML (simplificado - usando regex por enquanto)
        let papers = self.parse_arxiv_xml(&xml_content)?;
        
        info!(count = papers.len(), "Parsed arXiv papers");
        
        Ok(papers)
    }

    /// Faz parsing do XML do arXiv
    fn parse_arxiv_xml(&self, xml: &str) -> Result<Vec<ArticleMetadata>> {
        use regex::Regex;
        
        let mut papers = Vec::new();
        
        // Regex para extrair informações do XML
        let id_re = Regex::new(r#"<id>([^<]+)</id>"#)?;
        let title_re = Regex::new(r#"<title>([^<]+)</title>"#)?;
        let summary_re = Regex::new(r#"<summary>([^<]+)</summary>"#)?;
        let published_re = Regex::new(r#"<published>([^<]+)</published>"#)?;
        
        let mut current_id = None;
        let mut current_title = None;
        let mut current_summary = None;
        let mut current_published = None;
        
        for line in xml.lines() {
            if let Some(caps) = id_re.captures(line) {
                current_id = Some(caps[1].to_string());
            }
            if let Some(caps) = title_re.captures(line) {
                current_title = Some(caps[1].to_string());
            }
            if let Some(caps) = summary_re.captures(line) {
                current_summary = Some(caps[1].to_string());
            }
            if let Some(caps) = published_re.captures(line) {
                current_published = Some(caps[1].to_string());
            }
            
            // Quando encontramos </entry>, criamos o paper
            if line.contains("</entry>") {
                if let Some(id) = current_id.take() {
                    let paper_id = id.replace("http://arxiv.org/abs/", "");
                    let _pdf_url = format!("https://arxiv.org/pdf/{}.pdf", paper_id.clone());
                    let page_url = format!("https://arxiv.org/abs/{}", paper_id.clone());
                    
                    let title = current_title.take().unwrap_or_else(|| "Untitled".to_string());
                    let title_clone = title.clone();
                    let summary = current_summary.take().unwrap_or_default();
                    let published = current_published.take()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc));
                    
                    papers.push(ArticleMetadata {
                        id: paper_id.clone(),
                        title,
                        url: page_url,
                        published_date: published,
                        author: Some("arXiv".to_string()),
                        summary: Some(summary),
                    });
                    
                    info!(paper_id = %paper_id, title = %title_clone, "Found paper");
                }
            }
        }
        
        Ok(papers)
    }

    /// Obtém URL de download do PDF a partir do ID do paper
    pub fn get_pdf_url(&self, paper_id: &str) -> String {
        format!("https://arxiv.org/pdf/{}.pdf", paper_id)
    }
}

