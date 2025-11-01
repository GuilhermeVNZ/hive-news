use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};
use crate::utils::article_registry::RegistryManager;

/// Filtro de notícias que verifica duplicatas no registry
pub struct NewsFilter {
    registry: RegistryManager,
    rejected_dir: PathBuf,
}

impl NewsFilter {
    /// Cria um novo filtro de notícias
    pub fn new(registry_path: PathBuf, rejected_dir: PathBuf) -> Result<Self> {
        let registry = RegistryManager::new(&registry_path)
            .context("Failed to create registry manager")?;
        
        Ok(Self {
            registry,
            rejected_dir,
        })
    }

    /// Verifica se o registry foi carregado corretamente
    pub fn check_registry(&self) -> Result<usize> {
        let all_articles = self.registry.get_all_articles();
        let total = all_articles.len();
        let published_count = all_articles
            .iter()
            .filter(|a| matches!(a.status, crate::utils::article_registry::ArticleStatus::Published))
            .count();
        
        info!(
            total_articles = total,
            published_articles = published_count,
            "Registry loaded successfully"
        );
        
        Ok(total)
    }

    /// Verifica se uma notícia já está registrada no registry (por ID ou URL)
    pub fn is_duplicate(&self, article_id: &str, article_url: &str) -> bool {
        // Verificar se o ID já está no registry
        if self.registry.is_article_registered(article_id) {
            debug!(
                article_id = %article_id,
                "Found duplicate by ID in registry"
            );
            return true;
        }

        // Verificar se a URL já está no registry (pode ter IDs diferentes mas mesma URL)
        let all_articles = self.registry.get_all_articles();
        let url_duplicate = all_articles.iter().any(|article| {
            // Comparar arxiv_url ou pdf_url com article_url
            article.arxiv_url == article_url || article.pdf_url == article_url
        });

        if url_duplicate {
            debug!(
                article_url = %article_url,
                "Found duplicate by URL in registry"
            );
            return true;
        }

        false
    }

    /// Move uma notícia rejeitada para a pasta rejected/
    pub async fn reject_news(&self, json_path: &Path) -> Result<()> {
        // Garantir que a pasta rejected existe
        fs::create_dir_all(&self.rejected_dir)
            .await
            .context("Failed to create rejected directory")?;

        // Nome do arquivo
        let filename = json_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;

        // Destino na pasta rejected
        let rejected_path = self.rejected_dir.join(filename);

        // Mover arquivo
        fs::rename(json_path, &rejected_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to move rejected news from {} to {}",
                    json_path.display(),
                    rejected_path.display()
                )
            })?;

        info!(
            source = %json_path.display(),
            destination = %rejected_path.display(),
            "Moved rejected news to rejected directory"
        );

        Ok(())
    }

    /// Retorna estatísticas do filtro
    #[allow(dead_code)]
    pub fn stats(&self) -> (usize, usize) {
        let all_articles = self.registry.get_all_articles();
        let published_count = all_articles
            .iter()
            .filter(|a| matches!(a.status, crate::utils::article_registry::ArticleStatus::Published))
            .count();
        (published_count, 0) // (total_published, total_rejected)
    }
}

