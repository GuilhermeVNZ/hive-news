use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Status do artigo no pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArticleStatus {
    Collected,   // Baixado mas ainda não filtrado
    Filtered,    // Aprovado pelo filtro
    Rejected,    // Rejeitado pelo filtro
    Published,   // Conteúdo gerado e publicado
}

/// Metadados completos de um artigo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    pub id: String,
    pub title: String,
    pub arxiv_url: String,
    pub pdf_url: String,
    pub status: ArticleStatus,
    
    // Dados do filtro (se aplicável)
    pub filter_score: Option<f64>,
    pub category: Option<String>,
    pub rejection_reason: Option<String>,
    
    // Timestamps
    pub collected_at: Option<DateTime<Utc>>,
    pub filtered_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    
    // Localização do conteúdo gerado
    pub output_dir: Option<PathBuf>,
    pub hidden: Option<bool>,
    // Sites de destino para publicação (ex.: ["airesearch","scienceai"]) 
    pub destinations: Option<Vec<String>>, 
}

/// Registry completo de artigos
#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleRegistry {
    pub articles: HashMap<String, ArticleMetadata>,
}

impl ArticleRegistry {
    /// Carrega o registry do arquivo JSON
    pub fn load(registry_path: &Path) -> Result<Self> {
        if !registry_path.exists() {
            return Ok(Self {
                articles: HashMap::new(),
            });
        }

        let content = fs::read_to_string(registry_path)
            .context("Failed to read registry file")?;
        
        let registry: ArticleRegistry = serde_json::from_str(&content)
            .context("Failed to parse registry JSON")?;

        Ok(registry)
    }

    /// Salva o registry no arquivo JSON
    pub fn save(&self, registry_path: &Path) -> Result<()> {
        // Criar diretório se não existir
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create registry directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize registry")?;

        fs::write(registry_path, content)
            .context("Failed to write registry file")?;

        Ok(())
    }

    /// Verifica se um artigo já foi registrado (em qualquer status)
    pub fn is_article_registered(&self, article_id: &str) -> bool {
        self.articles.contains_key(article_id)
    }

    /// Verifica se um artigo já foi publicado
    pub fn is_article_published(&self, article_id: &str) -> bool {
        self.articles
            .get(article_id)
            .map(|meta| meta.status == ArticleStatus::Published)
            .unwrap_or(false)
    }

    /// Registra um artigo como coletado
    pub fn register_collected(
        &mut self,
        article_id: String,
        title: String,
        arxiv_url: String,
        pdf_url: String,
    ) {
        let metadata = ArticleMetadata {
            id: article_id.clone(),
            title,
            arxiv_url,
            pdf_url,
            status: ArticleStatus::Collected,
            filter_score: None,
            category: None,
            rejection_reason: None,
            collected_at: Some(Utc::now()),
            filtered_at: None,
            rejected_at: None,
            published_at: None,
            output_dir: None,
            hidden: Some(false),
            destinations: None,
        };

        self.articles.insert(article_id, metadata);
    }

    /// Define/atualiza destinos de publicação para um artigo
    pub fn set_destinations(
        &mut self,
        article_id: &str,
        destinations: Vec<String>,
    ) -> Result<()> {
        let metadata = self.articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;
        metadata.destinations = Some(destinations);
        Ok(())
    }

    /// Atualiza um artigo como filtrado/aprovado
    pub fn register_filtered(
        &mut self,
        article_id: &str,
        filter_score: f64,
        category: String,
    ) -> Result<()> {
        let metadata = self.articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;

        metadata.status = ArticleStatus::Filtered;
        metadata.filter_score = Some(filter_score);
        metadata.category = Some(category);
        metadata.filtered_at = Some(Utc::now());

        Ok(())
    }

    /// Atualiza um artigo como rejeitado
    pub fn register_rejected(
        &mut self,
        article_id: &str,
        filter_score: f64,
        reason: String,
    ) -> Result<()> {
        let metadata = self.articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;

        metadata.status = ArticleStatus::Rejected;
        metadata.filter_score = Some(filter_score);
        metadata.rejection_reason = Some(reason);
        metadata.rejected_at = Some(Utc::now());

        Ok(())
    }

    /// Atualiza um artigo como publicado
    pub fn register_published(
        &mut self,
        article_id: &str,
        output_dir: PathBuf,
    ) -> Result<()> {
        let metadata = self.articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;

        metadata.status = ArticleStatus::Published;
        metadata.published_at = Some(Utc::now());
        metadata.output_dir = Some(output_dir);

        Ok(())
    }

    /// Obtém metadados de um artigo
    pub fn get_metadata(&self, article_id: &str) -> Option<&ArticleMetadata> {
        self.articles.get(article_id)
    }
}

/// Gerenciador thread-safe do registry
pub struct RegistryManager {
    registry_path: PathBuf,
    registry: Mutex<ArticleRegistry>,
}

impl RegistryManager {
    /// Cria um novo gerenciador do registry
    pub fn new(registry_path: impl AsRef<Path>) -> Result<Self> {
        let registry_path = registry_path.as_ref().to_path_buf();
        let registry = ArticleRegistry::load(&registry_path)?;

        Ok(Self {
            registry_path,
            registry: Mutex::new(registry),
        })
    }

    /// Salva o registry no disco
    pub fn save(&self) -> Result<()> {
        let registry = self.registry.lock().unwrap();
        registry.save(&self.registry_path)
    }

    /// Verifica se um artigo já foi registrado
    pub fn is_article_registered(&self, article_id: &str) -> bool {
        let registry = self.registry.lock().unwrap();
        registry.is_article_registered(article_id)
    }

    /// Verifica se um artigo já foi publicado
    pub fn is_article_published(&self, article_id: &str) -> bool {
        let registry = self.registry.lock().unwrap();
        registry.is_article_published(article_id)
    }

    /// Registra um artigo como coletado
    pub fn register_collected(
        &self,
        article_id: String,
        title: String,
        arxiv_url: String,
        pdf_url: String,
    ) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_collected(article_id, title, arxiv_url, pdf_url);
        drop(registry); // Liberar lock antes de salvar
        
        self.save()?;
        Ok(())
    }

    /// Atualiza destinos de um artigo e salva
    pub fn set_destinations(&self, article_id: &str, destinations: Vec<String>) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.set_destinations(article_id, destinations)?;
        drop(registry);
        self.save()?;
        Ok(())
    }

    /// Registra um artigo como filtrado
    pub fn register_filtered(
        &self,
        article_id: &str,
        filter_score: f64,
        category: String,
    ) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_filtered(article_id, filter_score, category)?;
        drop(registry);
        
        self.save()?;
        Ok(())
    }

    /// Registra um artigo como rejeitado
    pub fn register_rejected(
        &self,
        article_id: &str,
        filter_score: f64,
        reason: String,
    ) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_rejected(article_id, filter_score, reason)?;
        drop(registry);
        
        self.save()?;
        Ok(())
    }

    /// Registra um artigo como publicado
    pub fn register_published(
        &self,
        article_id: &str,
        output_dir: PathBuf,
    ) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_published(article_id, output_dir)?;
        drop(registry);
        
        self.save()?;
        Ok(())
    }

    /// Obtém metadados de um artigo
    pub fn get_metadata(&self, article_id: &str) -> Option<ArticleMetadata> {
        let registry = self.registry.lock().unwrap();
        registry.get_metadata(article_id).cloned()
    }

    /// Lista todos os artigos de um status específico
    pub fn list_by_status(&self, status: ArticleStatus) -> Vec<ArticleMetadata> {
        let registry = self.registry.lock().unwrap();
        registry.articles
            .values()
            .filter(|meta| meta.status == status)
            .cloned()
            .collect()
    }

    /// Retorna todos os artigos no registry
    pub fn get_all_articles(&self) -> Vec<ArticleMetadata> {
        let registry = self.registry.lock().unwrap();
        registry.articles.values().cloned().collect()
    }
}


