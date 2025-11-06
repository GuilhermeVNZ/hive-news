use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Método de coleta utilizado
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CollectionMethod {
    Rss,
    Html,
    Playwright,
    Api,
}

impl CollectionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            CollectionMethod::Rss => "rss",
            CollectionMethod::Html => "html",
            CollectionMethod::Playwright => "playwright",
            CollectionMethod::Api => "api",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rss" => Some(CollectionMethod::Rss),
            "html" => Some(CollectionMethod::Html),
            "playwright" => Some(CollectionMethod::Playwright),
            "api" => Some(CollectionMethod::Api),
            _ => None,
        }
    }
}

/// Estatísticas de um método de coleta para uma fonte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodStats {
    pub success: u32,
    pub failure: u32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
}

impl Default for MethodStats {
    fn default() -> Self {
        Self {
            success: 0,
            failure: 0,
            last_attempt: None,
            last_success: None,
        }
    }
}

/// Informações sobre uma fonte e seus métodos de coleta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub domain: String,
    pub methods: HashMap<String, MethodStats>,
    pub effective_method: Option<String>,
    pub last_effective_at: Option<DateTime<Utc>>,
}

impl SourceInfo {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            methods: HashMap::new(),
            effective_method: None,
            last_effective_at: None,
        }
    }

    /// Retorna o método mais eficaz (maior taxa de sucesso)
    pub fn get_effective_method(&self) -> Option<String> {
        if let Some(ref effective) = self.effective_method {
            return Some(effective.clone());
        }

        // Calcular método com melhor taxa de sucesso
        let mut best_method: Option<(String, f64)> = None;

        for (method, stats) in &self.methods {
            let total = stats.success + stats.failure;
            if total == 0 {
                continue;
            }

            let success_rate = stats.success as f64 / total as f64;
            
            if let Some((_, best_rate)) = best_method {
                if success_rate > best_rate {
                    best_method = Some((method.clone(), success_rate));
                }
            } else {
                best_method = Some((method.clone(), success_rate));
            }
        }

        best_method.map(|(method, _)| method)
    }
}

/// Registry completo de fontes e métodos
#[derive(Debug, Serialize, Deserialize)]
pub struct SourcesRegistry {
    pub sources: HashMap<String, SourceInfo>,
}

impl SourcesRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn load(registry_path: &Path) -> Result<Self> {
        if !registry_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(registry_path)
            .context(format!("Failed to read sources registry: {}", registry_path.display()))?;

        let registry: SourcesRegistry = serde_json::from_str(&content)
            .context("Failed to parse sources registry JSON")?;

        Ok(registry)
    }

    pub fn save(&self, registry_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize sources registry")?;

        fs::write(registry_path, json)
            .context(format!("Failed to write sources registry: {}", registry_path.display()))?;

        Ok(())
    }

    /// Extrai o domínio de uma URL
    pub fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.to_string();
            }
        }
        // Fallback: tentar extrair manualmente
        url.replace("https://", "")
            .replace("http://", "")
            .split('/')
            .next()
            .unwrap_or(url)
            .to_string()
    }

    /// Registra sucesso de um método para uma fonte
    pub fn record_success(&mut self, url: &str, method: CollectionMethod) {
        let domain = Self::extract_domain(url);
        let method_str = method.as_str();

        let source_info = self.sources
            .entry(domain.clone())
            .or_insert_with(|| SourceInfo::new(domain.clone()));

        let stats = source_info.methods
            .entry(method_str.to_string())
            .or_insert_with(MethodStats::default);

        stats.success += 1;
        stats.last_attempt = Some(Utc::now());
        stats.last_success = Some(Utc::now());

        // Atualizar método eficaz se este foi bem-sucedido
        source_info.effective_method = Some(method_str.to_string());
        source_info.last_effective_at = Some(Utc::now());
    }

    /// Registra falha de um método para uma fonte
    pub fn record_failure(&mut self, url: &str, method: CollectionMethod) {
        let domain = Self::extract_domain(url);
        let method_str = method.as_str();

        let source_info = self.sources
            .entry(domain.clone())
            .or_insert_with(|| SourceInfo::new(domain.clone()));

        let stats = source_info.methods
            .entry(method_str.to_string())
            .or_insert_with(MethodStats::default);

        stats.failure += 1;
        stats.last_attempt = Some(Utc::now());
    }

    /// Obtém o método mais eficaz para uma fonte
    pub fn get_effective_method(&self, url: &str) -> Option<CollectionMethod> {
        let domain = Self::extract_domain(url);

        self.sources
            .get(&domain)
            .and_then(|source_info| source_info.get_effective_method())
            .and_then(|method_str| CollectionMethod::from_str(&method_str))
    }

    /// Obtém estatísticas de um método para uma fonte
    #[allow(dead_code)] // Pode ser útil para debugging ou estatísticas futuras
    pub fn get_method_stats(&self, url: &str, method: &CollectionMethod) -> Option<&MethodStats> {
        let domain = Self::extract_domain(url);
        let method_str = method.as_str();

        self.sources
            .get(&domain)
            .and_then(|source_info| source_info.methods.get(method_str))
    }
}

/// Gerenciador thread-safe do registry de fontes
pub struct SourcesRegistryManager {
    registry_path: PathBuf,
    registry: Mutex<SourcesRegistry>,
}

impl SourcesRegistryManager {
    pub fn new(registry_path: impl AsRef<Path>) -> Result<Self> {
        let registry_path = registry_path.as_ref().to_path_buf();
        let registry = SourcesRegistry::load(&registry_path)?;

        Ok(Self {
            registry_path,
            registry: Mutex::new(registry),
        })
    }

    pub fn save(&self) -> Result<()> {
        let registry = self.registry.lock().unwrap();
        registry.save(&self.registry_path)
    }

    pub fn record_success(&self, url: &str, method: CollectionMethod) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.record_success(url, method);
        drop(registry);
        self.save()
    }

    pub fn record_failure(&self, url: &str, method: CollectionMethod) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        registry.record_failure(url, method);
        drop(registry);
        self.save()
    }

    pub fn get_effective_method(&self, url: &str) -> Option<CollectionMethod> {
        let registry = self.registry.lock().unwrap();
        registry.get_effective_method(url)
    }

    #[allow(dead_code)] // Pode ser útil para debugging ou estatísticas futuras
    pub fn get_method_stats(&self, url: &str, method: &CollectionMethod) -> Option<MethodStats> {
        let registry = self.registry.lock().unwrap();
        registry.get_method_stats(url, method).cloned()
    }
}


