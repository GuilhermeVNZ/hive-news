use crate::utils::article_registry::RegistryManager;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

/// Filtro de notícias que verifica duplicatas no registry
pub struct NewsFilter {
    registry: RegistryManager,
    rejected_dir: PathBuf,
}

impl NewsFilter {
    /// Cria um novo filtro de notícias
    pub fn new(registry_path: PathBuf, rejected_dir: PathBuf) -> Result<Self> {
        let registry =
            RegistryManager::new(&registry_path).context("Failed to create registry manager")?;

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
            .filter(|a| {
                matches!(
                    a.status,
                    crate::utils::article_registry::ArticleStatus::Published
                )
            })
            .count();

        info!(
            total_articles = total,
            published_articles = published_count,
            "Registry loaded successfully"
        );

        Ok(total)
    }

    /// Normaliza uma URL para comparação (remove trailing slash, converte para lowercase, etc.)
    /// Mas mantém a URL completa (não apenas domínio)
    fn normalize_url_for_comparison(url: &str) -> String {
        let mut normalized = url.trim().to_string();

        // Remover trailing slash (exceto para raiz do domínio)
        if normalized.ends_with('/') && normalized.len() > 1 && !normalized.ends_with("://") {
            normalized.pop();
        }

        // Converter para lowercase para comparação case-insensitive
        normalized = normalized.to_lowercase();

        normalized
    }

    /// Verifica se uma URL já está registrada no registry (em qualquer status)
    /// Esta é a verificação primária para evitar duplicatas antes de processar
    ///
    /// IMPORTANTE: Verifica URL completa, não apenas domínio
    /// Exemplos de URLs que são consideradas diferentes:
    /// - "https://openai.com/global-affairs/brazil-ai-moment-is-here" (OK - URL completa)
    /// - "https://openai.com/index/introducing-indqa" (OK - URL completa diferente)
    ///
    /// NÃO verifica apenas domínio genérico como "openai.com"
    /// A verificação é específica para cada notícia individual
    ///
    /// Se `collecting_news` for true, permite coletar notícias mesmo que já existam como artigos do arXiv
    pub fn is_url_duplicate(&self, article_url: &str, collecting_news: bool) -> bool {
        // Normalizar URL do artigo sendo verificado (remove trailing slash, lowercase)
        // MAS mantém URL completa (path completo, não apenas domínio)
        let normalized_article_url = Self::normalize_url_for_comparison(article_url);

        // Se estamos coletando notícias (RSS/HTML), verificar se o artigo existente é do arXiv
        // Se for do arXiv, permitir coletar a notícia mesmo que a URL seja a mesma
        if collecting_news {
            let all_articles = self.registry.get_all_articles();
            let mut found_match = false;
            let mut is_arxiv_match = false;
            
            // Para RSS do AIResearch, a URL será https://www.airesearch.news/article/...
            // Essa URL NÃO vai corresponder a arxiv_url ou pdf_url (que são URLs do arXiv)
            // Então precisamos verificar se há algum artigo no registry que:
            // 1. Tem a mesma URL (caso raro, mas possível)
            // 2. É do arXiv (permitir coletar news mesmo assim)
            
            for article in all_articles.iter() {
                let normalized_arxiv = Self::normalize_url_for_comparison(&article.arxiv_url);
                let normalized_pdf = Self::normalize_url_for_comparison(&article.pdf_url);

                // Se a URL corresponde, verificar se o artigo existente é do arXiv
                if normalized_arxiv == normalized_article_url || normalized_pdf == normalized_article_url {
                    found_match = true;
                    // Se o artigo existente é do arXiv (URL contém arxiv.org), permitir coletar a notícia
                    is_arxiv_match = normalized_arxiv.contains("arxiv.org") 
                        || normalized_pdf.contains("arxiv.org");
                    break; // Encontrou match, pode parar
                }
            }

            if found_match {
                if is_arxiv_match {
                    // É do arXiv, NÃO considerar duplicata (permitir coletar news)
                    debug!(
                        article_url = %article_url,
                        normalized_url = %normalized_article_url,
                        "URL matches arXiv article - allowing news collection"
                    );
                    return false; // Não é duplicata, permitir coletar
                } else {
                    // Não é do arXiv, é duplicata real
                    debug!(
                        article_url = %article_url,
                        normalized_url = %normalized_article_url,
                        "Found duplicate by URL in registry (not from arXiv)"
                    );
                    return true;
                }
            }
            // Não encontrou match de URL, não é duplicata por URL
            // A verificação por ID será feita depois
            return false;
        }

        // Lógica original para coleta de artigos (não news)
        let all_articles = self.registry.get_all_articles();
        let url_duplicate = all_articles.iter().any(|article| {
            // Verificar URL em qualquer status (Collected, Filtered, Published, Rejected)
            // Normalizar ambas URLs antes de comparar
            let normalized_arxiv = Self::normalize_url_for_comparison(&article.arxiv_url);
            let normalized_pdf = Self::normalize_url_for_comparison(&article.pdf_url);

            // Comparar URL completa normalizada (não apenas domínio)
            // Exemplo: compara "https://openai.com/global-affairs/brazil-ai-moment-is-here"
            // com "https://openai.com/global-affairs/brazil-ai-moment-is-here" (duplicata)
            // mas NÃO com "https://openai.com/index/introducing-indqa" (diferente)
            normalized_arxiv == normalized_article_url || normalized_pdf == normalized_article_url
        });

        if url_duplicate {
            debug!(
                article_url = %article_url,
                normalized_url = %normalized_article_url,
                "Found duplicate by URL in registry (any status)"
            );
            return true;
        }

        false
    }

    /// Verifica se uma notícia já está registrada no registry (por ID ou URL)
    /// Considera apenas artigos com status "Published", não "Collected" (que podem não ter sido escritos)
    ///
    /// Se `collecting_news` for true, permite coletar notícias mesmo que já existam como artigos do arXiv
    pub fn is_duplicate(&self, article_id: &str, article_url: &str, collecting_news: bool) -> bool {
        use crate::utils::article_registry::ArticleStatus;

        // PRIMEIRO: Verificar URL em qualquer status (verificação mais completa)
        if self.is_url_duplicate(article_url, collecting_news) {
            return true;
        }

        // Se estamos coletando notícias, verificar se o artigo existente é do arXiv
        if collecting_news {
            if let Some(metadata) = self.registry.get_metadata(article_id) {
                // Se o artigo existente é do arXiv (URL contém arxiv.org), permitir coletar a notícia
                let is_arxiv_article = metadata.arxiv_url.contains("arxiv.org") 
                    || metadata.pdf_url.contains("arxiv.org");
                
                debug!(
                    article_id = %article_id,
                    arxiv_url = %metadata.arxiv_url,
                    pdf_url = %metadata.pdf_url,
                    is_arxiv = is_arxiv_article,
                    "Checking if article is from arXiv"
                );
                
                // Se é do arXiv, NÃO considerar duplicata (permitir coletar news)
                if is_arxiv_article {
                    debug!(
                        article_id = %article_id,
                        "Article is from arXiv - allowing news collection"
                    );
                    return false; // Não é duplicata, permitir coletar
                }
                
                // Se não é do arXiv e está Published, é duplicata real
                if matches!(metadata.status, ArticleStatus::Published) {
                    debug!(
                        article_id = %article_id,
                        "Found duplicate by ID in registry (Published, not from arXiv)"
                    );
                    return true;
                }
            }
            return false;
        }

        // Lógica original para coleta de artigos (não news)
        // Verificar se o ID já está no registry E tem status Published
        if let Some(metadata) = self.registry.get_metadata(article_id)
            && matches!(metadata.status, ArticleStatus::Published)
        {
            debug!(
                article_id = %article_id,
                "Found duplicate by ID in registry (Published)"
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
            .filter(|a| {
                matches!(
                    a.status,
                    crate::utils::article_registry::ArticleStatus::Published
                )
            })
            .count();
        (published_count, 0) // (total_published, total_rejected)
    }
}
