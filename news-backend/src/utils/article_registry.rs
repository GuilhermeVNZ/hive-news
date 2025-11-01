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
    // Artigo destacado para primeira página
    pub featured: Option<bool>,
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
        
        // Tentar parse normal primeiro
        match serde_json::from_str::<ArticleRegistry>(&content) {
            Ok(registry) => Ok(registry),
            Err(e) => {
                // Tentar reparar JSON corrompido
                eprintln!("⚠️  Failed to parse registry JSON: {}. Attempting repair...", e);
                
                // Estratégia 1: Remover trailing characters simples
                let trimmed = content.trim();
                if let Ok(registry) = serde_json::from_str::<ArticleRegistry>(trimmed) {
                    eprintln!("✅ Successfully repaired registry JSON (simple trim)!");
                    if let Err(save_err) = registry.save(registry_path) {
                        eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                    }
                    return Ok(registry);
                }
                
                // Estratégia 2: Encontrar último } válido usando contador de chaves
                if let Some(repaired) = Self::repair_json_by_finding_last_valid_brace(&trimmed) {
                    if let Ok(registry) = serde_json::from_str::<ArticleRegistry>(&repaired) {
                        eprintln!("✅ Successfully repaired registry JSON (brace matching)!");
                        if let Err(save_err) = registry.save(registry_path) {
                            eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                        }
                        return Ok(registry);
                    }
                }
                
                // Estratégia 3: Tentar extrair apenas a seção "articles"
                if let Some(articles_json) = Self::extract_articles_section(&trimmed) {
                    let repaired = format!("{{\"articles\":{}}}", articles_json);
                    if let Ok(registry) = serde_json::from_str::<ArticleRegistry>(&repaired) {
                        eprintln!("✅ Successfully repaired registry JSON (extracted articles section)!");
                        if let Err(save_err) = registry.save(registry_path) {
                            eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                        }
                        return Ok(registry);
                    }
                }
                
                // Estratégia 4: Se tudo falhar, criar backup e iniciar novo registry
                eprintln!("⚠️  All repair strategies failed. Creating backup and initializing new registry...");
                let backup_path = format!("{}.backup.{}", registry_path.display(), chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                if let Err(backup_err) = fs::copy(registry_path, &backup_path) {
                    eprintln!("⚠️  Failed to create backup: {}", backup_err);
                } else {
                    eprintln!("✅ Backup created: {}", backup_path);
                }
                
                // Criar novo registry vazio
                let new_registry = Self {
                    articles: HashMap::new(),
                };
                if let Err(save_err) = new_registry.save(registry_path) {
                    eprintln!("⚠️  Failed to save new registry: {}", save_err);
                }
                eprintln!("✅ Initialized new empty registry");
                Ok(new_registry)
            }
        }
    }
    
    /// Tenta reparar JSON encontrando o último } válido
    fn repair_json_by_finding_last_valid_brace(content: &str) -> Option<String> {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut last_valid_pos = None;
        
        for (i, ch) in content.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        last_valid_pos = Some(i);
                    }
                }
                _ => {}
            }
        }
        
        if let Some(pos) = last_valid_pos {
            Some(content[..=pos].to_string())
        } else {
            None
        }
    }
    
    /// Extrai a seção "articles" do JSON
    fn extract_articles_section(content: &str) -> Option<String> {
        if let Some(articles_start) = content.find("\"articles\":") {
            let after_colon = &content[articles_start + "\"articles\":".len()..];
            let trimmed_after = after_colon.trim_start();
            
            // Encontrar o início do objeto
            if let Some(obj_start) = trimmed_after.find('{') {
                let json_part = &trimmed_after[obj_start..];
                
                // Encontrar o fim do objeto usando contador de chaves
                let mut brace_count = 0;
                let mut in_string = false;
                let mut escape_next = false;
                let mut end_pos = None;
                
                for (i, ch) in json_part.char_indices() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }
                    
                    match ch {
                        '\\' if in_string => {
                            escape_next = true;
                        }
                        '"' => {
                            in_string = !in_string;
                        }
                        '{' if !in_string => {
                            brace_count += 1;
                        }
                        '}' if !in_string => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_pos = Some(i + 1);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                
                if let Some(end) = end_pos {
                    Some(json_part[..end].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
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
            featured: Some(false),
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

    /// Atualiza o status featured de um artigo
    pub fn set_featured(&self, article_id: &str, featured: bool) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        if let Some(meta) = registry.articles.get_mut(article_id) {
            meta.featured = Some(featured);
            drop(registry); // Liberar lock antes de salvar
            self.save()?;
            Ok(())
        } else {
            drop(registry);
            Err(anyhow::anyhow!("Article with ID '{}' not found", article_id))
        }
    }

    /// Atualiza o status hidden de um artigo
    pub fn set_hidden(&self, article_id: &str, hidden: bool) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        if let Some(meta) = registry.articles.get_mut(article_id) {
            meta.hidden = Some(hidden);
            drop(registry); // Liberar lock antes de salvar
            self.save()?;
            Ok(())
        } else {
            drop(registry);
            Err(anyhow::anyhow!("Article with ID '{}' not found", article_id))
        }
    }
}


