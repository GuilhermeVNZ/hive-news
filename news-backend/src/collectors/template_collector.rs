// Template para criar novos collectors de artigos científicos
// Copie este arquivo e adapte para sua API específica

use crate::models::raw_document::ArticleMetadata;
use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;
use chrono::Utc;

/// Template Collector - Substitua pelo nome do seu collector
/// 
/// Exemplos:
/// - SemanticScholarCollector
/// - CrossrefCollector
/// - PubMedCollector
pub struct TemplateCollector {
    client: Client,
    temp_dir: PathBuf,
}

impl TemplateCollector {
    /// Cria nova instância do collector
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .user_agent("News-System-Collector/1.0 (Template)")
                .cookie_store(true) // Se necessário para manter sessão
                .build()
                .expect("Failed to create client"),
            temp_dir,
        }
    }

    /// Busca artigos recentes da API
    /// 
    /// # Parâmetros
    /// - `category`: Categoria de busca (ex: "artificial-intelligence", "machine-learning")
    /// - `max_results`: Número máximo de artigos a buscar
    /// 
    /// # Retorna
    /// Vetor de metadados dos artigos encontrados
    pub async fn fetch_recent_papers(
        &self,
        category: &str,
        max_results: usize,
    ) -> Result<Vec<ArticleMetadata>> {
        // TODO: Implementar busca na API específica
        
        // Exemplo de estrutura:
        /*
        let url = format!(
            "https://api.exemplo.com/v1/papers?category={}&limit={}",
            category, max_results
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key)) // Se necessário
            .send()
            .await
            .context("Failed to fetch papers from API")?;

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse API response")?;

        // Extrair artigos da resposta JSON
        let mut articles = Vec::new();
        
        // Adaptar conforme estrutura da API
        for paper in json["papers"].as_array().unwrap_or(&vec![]) {
            articles.push(ArticleMetadata {
                id: paper["id"].as_str().unwrap().to_string(),
                title: paper["title"].as_str().unwrap().to_string(),
                authors: paper["authors"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|a| a["name"].as_str().unwrap_or("").to_string())
                    .collect(),
                abstract_text: paper["abstract"].as_str().unwrap_or("").to_string(),
                published_date: paper["published_date"].as_str().unwrap_or("").to_string(),
                doi: paper["doi"].as_str().map(|s| s.to_string()),
                pdf_url: paper["pdf_url"].as_str().unwrap_or("").to_string(),
                url: paper["url"].as_str().unwrap_or("").to_string(),
                categories: vec![category.to_string()],
            });
        }

        Ok(articles)
        */

        // Por enquanto, retorna vazio
        Ok(vec![])
    }

    /// Download do PDF de um artigo
    /// 
    /// # Parâmetros
    /// - `paper_id`: ID único do artigo
    /// - `pdf_url`: URL do PDF para download
    /// - `output_path`: Caminho onde salvar o PDF
    pub async fn download_pdf(
        &self,
        paper_id: &str,
        pdf_url: &str,
        output_path: &PathBuf,
    ) -> Result<()> {
        // TODO: Implementar download do PDF
        
        // Exemplo:
        /*
        let response = self.client
            .get(pdf_url)
            .send()
            .await
            .context(format!("Failed to download PDF for {}", paper_id))?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download PDF: HTTP {}", response.status());
        }

        let pdf_data = response
            .bytes()
            .await
            .context("Failed to read PDF data")?;

        tokio::fs::write(output_path, pdf_data)
            .await
            .context(format!("Failed to save PDF to {}", output_path.display()))?;

        Ok(())
        */

        // Por enquanto, não faz nada
        Ok(())
    }

    /// Verifica se o artigo já existe no registry
    /// 
    /// Isso evita downloads duplicados
    pub fn is_duplicate(
        &self,
        paper_id: &str,
        registry: &crate::utils::article_registry::RegistryManager,
    ) -> bool {
        registry.is_article_registered(paper_id)
    }

    /// Extrai metadados adicionais da API (se disponível)
    /// 
    /// Algumas APIs fornecem informações extras como citações, métricas, etc.
    pub async fn fetch_additional_metadata(
        &self,
        paper_id: &str,
    ) -> Result<serde_json::Value> {
        // TODO: Implementar busca de metadados adicionais
        // Ex: citações, Altmetric score, etc.
        
        Ok(serde_json::json!({}))
    }
}

// Exemplo de uso do collector no main.rs:
//
// async fn run_template_collection_direct() -> anyhow::Result<()> {
//     use crate::collectors::template_collector::TemplateCollector;
//     use crate::utils::article_registry::RegistryManager;
//     use std::path::Path;
//
//     // Inicializar registry
//     let registry_path = Path::new("/opt/news-system/articles_registry.json");
//     let registry = RegistryManager::new(registry_path)?;
//
//     // Inicializar collector
//     let base_dir = Path::new("/opt/news-system/downloads");
//     let temp_dir = base_dir.join("temp");
//     let collector = TemplateCollector::new(temp_dir);
//
//     // Buscar artigos
//     println!("📡 Fetching papers from Template API...");
//     let articles = collector.fetch_recent_papers("ai", 10).await?;
//     println!("✅ Found {} papers", articles.len());
//
//     // Processar cada artigo
//     for article in articles {
//         if collector.is_duplicate(&article.id, &registry) {
//             println!("⏭️  Skipping {} (already in registry)", article.id);
//             continue;
//         }
//
//         // Download PDF
//         let pdf_path = base_dir.join("template").join(&format!("{}.pdf", article.id));
//         collector.download_pdf(&article.id, &article.pdf_url, &pdf_path).await?;
//
//         // Registrar no registry
//         registry.register_collected(&article.id, &article.title)?;
//
//         println!("✅ Downloaded: {} - {}", article.id, article.title);
//     }
//
//     Ok(())
// }























































