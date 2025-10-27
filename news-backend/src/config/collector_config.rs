use serde::{Deserialize, Serialize};
use std::env;

/// Configurações gerais do Collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Timeout para requests HTTP (segundos)
    pub timeout_seconds: u64,
    
    /// Número máximo de tentativas
    pub max_retries: u32,
    
    /// Diretório base para downloads
    pub download_dir: String,
    
    /// Diretório para arquivos temporários
    pub temp_dir: String,
    
    /// Categoria padrão do arXiv
    pub arxiv_category: String,
    
    /// Número máximo de resultados do arXiv
    pub arxiv_max_results: u32,
    
    /// Retenção de arquivos temporários (dias)
    pub temp_file_retention_days: u32,
    
    /// Tamanho máximo de download (MB)
    pub max_download_size_mb: u64,
    
    /// Rate limiting por API (requests por minuto)
    pub rate_limits: RateLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub nature: u32,
    pub science: u32,
    pub arxiv: u32,
    pub pubmed: u32,
    pub ieee: u32,
    pub springer: u32,
    pub elsevier: u32,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            download_dir: "./downloads".to_string(),
            temp_dir: "./downloads/temp".to_string(),
            arxiv_category: "cs.AI".to_string(),
            arxiv_max_results: 10,
            temp_file_retention_days: 7,
            max_download_size_mb: 50,
            rate_limits: RateLimits {
                nature: 60,
                science: 60,
                arxiv: 120,
                pubmed: 100,
                ieee: 60,
                springer: 60,
                elsevier: 60,
            },
        }
    }
}

impl CollectorConfig {
    /// Carrega configurações das variáveis de ambiente
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(timeout) = env::var("COLLECTOR_TIMEOUT_SECONDS") {
            if let Ok(val) = timeout.parse() {
                config.timeout_seconds = val;
            }
        }
        
        if let Ok(retries) = env::var("COLLECTOR_MAX_RETRIES") {
            if let Ok(val) = retries.parse() {
                config.max_retries = val;
            }
        }
        
        if let Ok(dir) = env::var("COLLECTOR_DOWNLOAD_DIR") {
            config.download_dir = dir.clone();
            config.temp_dir = format!("{}/temp", dir);
        }
        
        if let Ok(category) = env::var("ARXIV_CATEGORY") {
            config.arxiv_category = category;
        }
        
        if let Ok(max) = env::var("ARXIV_MAX_RESULTS") {
            if let Ok(val) = max.parse() {
                config.arxiv_max_results = val;
            }
        }
        
        // Rate limits
        if let Ok(nature) = env::var("NATURE_RATE_LIMIT") {
            if let Ok(val) = nature.parse() {
                config.rate_limits.nature = val;
            }
        }
        
        if let Ok(science) = env::var("SCIENCE_RATE_LIMIT") {
            if let Ok(val) = science.parse() {
                config.rate_limits.science = val;
            }
        }
        
        if let Ok(arxiv) = env::var("ARXIV_RATE_LIMIT") {
            if let Ok(val) = arxiv.parse() {
                config.rate_limits.arxiv = val;
            }
        }
        
        if let Ok(pubmed) = env::var("PUBMED_RATE_LIMIT") {
            if let Ok(val) = pubmed.parse() {
                config.rate_limits.pubmed = val;
            }
        }
        
        if let Ok(retention) = env::var("TEMP_FILE_RETENTION_DAYS") {
            if let Ok(val) = retention.parse() {
                config.temp_file_retention_days = val;
            }
        }
        
        if let Ok(max_size) = env::var("MAX_DOWNLOAD_SIZE_MB") {
            if let Ok(val) = max_size.parse() {
                config.max_download_size_mb = val;
            }
        }
        
        config
    }
    
    /// Retorna o tamanho máximo em bytes
    pub fn max_download_size_bytes(&self) -> u64 {
        self.max_download_size_mb * 1024 * 1024
    }
}

