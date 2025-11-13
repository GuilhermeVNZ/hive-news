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
    Collected, // Baixado mas ainda não filtrado
    Filtered,  // Aprovado pelo filtro
    Rejected,  // Rejeitado pelo filtro
    Published, // Conteúdo gerado e publicado
}

/// Metadados completos de um artigo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    pub id: String,
    pub title: String, // Mantido para compatibilidade, mas será descontinuado
    pub arxiv_url: String,
    pub pdf_url: String,
    pub status: ArticleStatus,

    // Títulos: original (da fonte) e gerado (pelo DeepSeek)
    #[serde(default)]
    pub original_title: Option<String>, // Título original da notícia/artigo (do arXiv ou fonte)
    #[serde(default)]
    pub generated_title: Option<String>, // Título gerado pelo DeepSeek (do title.txt)

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

impl ArticleMetadata {
    fn normalize_paths(&mut self) {
        if let Some(output_dir) = &self.output_dir {
            let normalized = output_dir.to_string_lossy().replace('\\', "/");
            let trimmed = normalized.trim_start_matches("./");
            let mut relative = trimmed.to_string();

            for marker in ["output/", "downloads/", "images/"] {
                if let Some(idx) = relative.find(marker) {
                    relative = relative[idx..].to_string();
                    break;
                }
            }

            if relative.is_empty() {
                relative = normalized;
            }

            self.output_dir = Some(PathBuf::from(relative));
        }
    }
}

/// Registry completo de artigos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleRegistry {
    pub articles: HashMap<String, ArticleMetadata>,
}

impl ArticleRegistry {
    fn normalize_paths(&mut self) {
        for metadata in self.articles.values_mut() {
            metadata.normalize_paths();
        }
    }

    /// Carrega o registry do arquivo JSON
    pub fn load(registry_path: &Path) -> Result<Self> {
        if !registry_path.exists() {
            return Ok(Self {
                articles: HashMap::new(),
            });
        }

        let content = fs::read_to_string(registry_path).context("Failed to read registry file")?;

        // Tentar parse normal primeiro
        match serde_json::from_str::<ArticleRegistry>(&content) {
            Ok(mut registry) => {
                registry.normalize_paths();
                Ok(registry)
            }
            Err(e) => {
                // Tentar reparar JSON corrompido
                eprintln!(
                    "⚠️  Failed to parse registry JSON: {}. Attempting repair...",
                    e
                );

                // Estratégia 1: Remover trailing characters simples
                let trimmed = content.trim();
                if let Ok(mut registry) = serde_json::from_str::<ArticleRegistry>(trimmed) {
                    registry.normalize_paths();
                    eprintln!("✅ Successfully repaired registry JSON (simple trim)!");
                    if let Err(save_err) = registry.save(registry_path) {
                        eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                    }
                    return Ok(registry);
                }

                // Estratégia 2: Encontrar último } válido usando contador de chaves
                if let Some(repaired) = Self::repair_json_by_finding_last_valid_brace(trimmed)
                    && let Ok(mut registry) = serde_json::from_str::<ArticleRegistry>(&repaired)
                {
                    registry.normalize_paths();
                    eprintln!("✅ Successfully repaired registry JSON (brace matching)!");
                    if let Err(save_err) = registry.save(registry_path) {
                        eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                    }
                    return Ok(registry);
                }

                // Estratégia 3: Tentar extrair apenas a seção "articles"
                if let Some(articles_json) = Self::extract_articles_section(trimmed) {
                    let repaired = format!("{{\"articles\":{}}}", articles_json);
                    if let Ok(mut registry) = serde_json::from_str::<ArticleRegistry>(&repaired) {
                        registry.normalize_paths();
                        eprintln!(
                            "✅ Successfully repaired registry JSON (extracted articles section)!"
                        );
                        if let Err(save_err) = registry.save(registry_path) {
                            eprintln!("⚠️  Failed to save repaired registry: {}", save_err);
                        }
                        return Ok(registry);
                    }
                }

                // Estratégia 4: Se tudo falhar, criar backup e iniciar novo registry
                eprintln!(
                    "⚠️  All repair strategies failed. Creating backup and initializing new registry..."
                );
                let backup_path = format!(
                    "{}.backup.{}",
                    registry_path.display(),
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                );
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

        last_valid_pos.map(|pos| content[..=pos].to_string())
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

                end_pos.map(|end| json_part[..end].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Salva o registry no arquivo JSON usando escrita atômica (tempfile + rename)
    /// com retry automático em caso de EBUSY/conflitos
    pub fn save(&self, registry_path: &Path) -> Result<()> {
        use std::io::Write;
        use std::thread;
        use std::time::Duration;
        use tempfile::NamedTempFile;

        const MAX_RETRIES: u32 = 5;
        const INITIAL_BACKOFF_MS: u64 = 50;

        // Criar diretório se não existir
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent).context("Failed to create registry directory")?;
        }

        let mut registry_clone = self.clone();
        registry_clone.normalize_paths();

        let content = serde_json::to_string_pretty(&registry_clone)
            .context("Failed to serialize registry")?;

        let parent_dir = registry_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Registry path has no parent directory"))?;

        // Retry loop com backoff exponencial
        let mut last_error = None;
        let mut retry_count = 0;
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let backoff = INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1);
                thread::sleep(Duration::from_millis(backoff));
                retry_count += 1;
            }

            // Criar arquivo temporário no mesmo diretório do arquivo final
            let mut tmp = match NamedTempFile::new_in(parent_dir) {
                Ok(t) => t,
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };

            // Escrever conteúdo no arquivo temporário
            if let Err(e) = tmp.as_file_mut().write_all(content.as_bytes()) {
                last_error = Some(e);
                continue;
            }

            // Forçar sincronização física (flush to disk)
            if let Err(e) = tmp.as_file_mut().sync_all() {
                last_error = Some(e);
                continue;
            }

            // Rename atômico (move temp -> final)
            match tmp.persist(registry_path) {
                Ok(_) => {
                    if retry_count > 0 {
                        eprintln!("[ArticleRegistry] ✅ Saved after {} retries", retry_count);
                    }
                    return Ok(());
                }
                Err(persist_err) => {
                    last_error = Some(persist_err.error);
                    let _ = persist_err.file.close();
                }
            }
        }

        // Fallback direto se todas as tentativas falharem
        std::fs::write(registry_path, content.as_bytes()).context(format!(
            "Failed to save registry after {} retries. Last error: {:?}",
            MAX_RETRIES, last_error
        ))?;
        if retry_count > 0 {
            eprintln!(
                "[ArticleRegistry] ✅ Saved via fallback after {} retries",
                retry_count
            );
        }
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
            title: title.clone(), // Mantido para compatibilidade
            arxiv_url,
            pdf_url,
            status: ArticleStatus::Collected,
            original_title: Some(title.clone()), // Título original da fonte
            generated_title: None,               // Será preenchido quando o artigo for publicado
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
    pub fn set_destinations(&mut self, article_id: &str, destinations: Vec<String>) -> Result<()> {
        let metadata = self
            .articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;
        metadata.destinations = Some(destinations);
        Ok(())
    }

    /// Remove um artigo do registry (permite retentativa)
    pub fn remove_article(&mut self, article_id: &str) {
        self.articles.remove(article_id);
    }

    /// Atualiza um artigo como filtrado/aprovado
    pub fn register_filtered(
        &mut self,
        article_id: &str,
        filter_score: f64,
        category: String,
    ) -> Result<()> {
        let metadata = self
            .articles
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
        let metadata = self
            .articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;

        metadata.status = ArticleStatus::Rejected;
        metadata.filter_score = Some(filter_score);
        metadata.rejection_reason = Some(reason);
        metadata.rejected_at = Some(Utc::now());

        Ok(())
    }

    /// Atualiza um artigo como publicado
    pub fn register_published(&mut self, article_id: &str, output_dir: PathBuf) -> Result<()> {
        let metadata = self
            .articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;

        metadata.status = ArticleStatus::Published;
        metadata.published_at = Some(Utc::now());
        let normalized_output = output_dir.to_string_lossy().replace('\\', "/");
        metadata.output_dir = Some(PathBuf::from(normalized_output));
        metadata.normalize_paths();

        // Tenta ler o título gerado do title.txt no output_dir
        let title_txt = output_dir.join("title.txt");
        if title_txt.exists()
            && let Ok(title_content) = fs::read_to_string(&title_txt)
        {
            let generated_title = title_content.trim().to_string();
            if !generated_title.is_empty() {
                metadata.generated_title = Some(generated_title);
            }
        }

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

    /// Remove um artigo do registry (permite retentativa) e salva
    pub fn remove_article(&self, article_id: &str) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.remove_article(article_id);
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
    pub fn register_published(&self, article_id: &str, output_dir: PathBuf) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_published(article_id, output_dir)?;
        drop(registry);

        self.save()?;
        Ok(())
    }

    /// Atualiza o título gerado (do title.txt) para um artigo
    #[allow(dead_code)]
    pub fn set_generated_title(&self, article_id: &str, generated_title: String) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let metadata = registry
            .articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;
        metadata.generated_title = Some(generated_title);
        drop(registry);

        self.save()?;
        Ok(())
    }

    /// Atualiza o título original (da fonte) para um artigo
    #[allow(dead_code)]
    pub fn set_original_title(&self, article_id: &str, original_title: String) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let metadata = registry
            .articles
            .get_mut(article_id)
            .context(format!("Article {} not found in registry", article_id))?;
        metadata.original_title = Some(original_title);
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
        registry
            .articles
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
        eprintln!(
            "[RegistryManager] set_featured called: article_id={}, featured={}",
            article_id, featured
        );
        eprintln!("[RegistryManager] Registry path: {:?}", self.registry_path);
        let mut registry = self.registry.lock().unwrap();
        if let Some(meta) = registry.articles.get_mut(article_id) {
            eprintln!(
                "[RegistryManager] Found article, old featured value: {:?}",
                meta.featured
            );
            meta.featured = Some(featured);
            eprintln!("[RegistryManager] Updated featured to: {:?}", meta.featured);
            drop(registry); // Liberar lock antes de salvar
            match self.save() {
                Ok(_) => {
                    eprintln!(
                        "[RegistryManager] ✅ Successfully saved registry with featured={} for article {}",
                        featured, article_id
                    );
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[RegistryManager] ❌ Failed to save registry: {}", e);
                    Err(e)
                }
            }
        } else {
            drop(registry);
            eprintln!(
                "[RegistryManager] ❌ Article '{}' not found in registry",
                article_id
            );
            Err(anyhow::anyhow!(
                "Article with ID '{}' not found",
                article_id
            ))
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
            Err(anyhow::anyhow!(
                "Article with ID '{}' not found",
                article_id
            ))
        }
    }
}
