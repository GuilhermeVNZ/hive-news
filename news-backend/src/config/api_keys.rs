use serde::{Deserialize, Serialize};
use std::env;

/// Configuração de chaves API para fontes de coleta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
    /// arXiv - Public API, sem chave necessária
    pub arxiv_base_url: String,
    
    /// Nature Publishing Group
    pub nature_api_key: Option<String>,
    pub nature_base_url: String,
    
    /// Science (AAAS)
    pub science_api_key: Option<String>,
    pub science_base_url: String,
    
    /// PubMed / NCBI - Public API
    pub pubmed_base_url: String,
    
    /// IEEE Xplore
    pub ieee_api_key: Option<String>,
    pub ieee_base_url: String,
    
    /// Springer Nature
    pub springer_api_key: Option<String>,
    pub springer_base_url: String,
    
    /// Elsevier ScienceDirect
    pub elsevier_api_key: Option<String>,
    pub elsevier_base_url: String,
}

impl ApiKeys {
    /// Carrega configurações de chaves API das variáveis de ambiente
    pub fn from_env() -> Self {
        Self {
            arxiv_base_url: env::var("ARXIV_BASE_URL")
                .unwrap_or_else(|_| "https://export.arxiv.org/api/query".to_string()),
            
            nature_api_key: env::var("NATURE_API_KEY").ok(),
            nature_base_url: env::var("NATURE_BASE_URL")
                .unwrap_or_else(|_| "https://api.nature.com".to_string()),
            
            science_api_key: env::var("SCIENCE_API_KEY").ok(),
            science_base_url: env::var("SCIENCE_BASE_URL")
                .unwrap_or_else(|_| "https://api.science.org".to_string()),
            
            pubmed_base_url: env::var("PUBMED_BASE_URL")
                .unwrap_or_else(|_| "https://eutils.ncbi.nlm.nih.gov/entrez/eutils".to_string()),
            
            ieee_api_key: env::var("IEEE_API_KEY").ok(),
            ieee_base_url: env::var("IEEE_BASE_URL")
                .unwrap_or_else(|_| "https://ieeexploreapi.ieee.org/api/v1".to_string()),
            
            springer_api_key: env::var("SPRINGER_API_KEY").ok(),
            springer_base_url: env::var("SPRINGER_BASE_URL")
                .unwrap_or_else(|_| "https://api.springernature.com".to_string()),
            
            elsevier_api_key: env::var("ELSEVIER_API_KEY").ok(),
            elsevier_base_url: env::var("ELSEVIER_BASE_URL")
                .unwrap_or_else(|_| "https://api.elsevier.com/content".to_string()),
        }
    }
    
    /// Verifica se uma fonte está configurada
    pub fn is_source_configured(&self, source: &str) -> bool {
        match source.to_lowercase().as_str() {
            "arxiv" => true, // Sempre disponível
            "nature" => self.nature_api_key.is_some(),
            "science" => self.science_api_key.is_some(),
            "pubmed" => true, // Sempre disponível
            "ieee" => self.ieee_api_key.is_some(),
            "springer" => self.springer_api_key.is_some(),
            "elsevier" => self.elsevier_api_key.is_some(),
            _ => false,
        }
    }
    
    /// Retorna a chave API de uma fonte
    pub fn get_api_key(&self, source: &str) -> Option<String> {
        match source.to_lowercase().as_str() {
            "nature" => self.nature_api_key.clone(),
            "science" => self.science_api_key.clone(),
            "ieee" => self.ieee_api_key.clone(),
            "springer" => self.springer_api_key.clone(),
            "elsevier" => self.elsevier_api_key.clone(),
            _ => None,
        }
    }
    
    /// Retorna a URL base de uma fonte
    pub fn get_base_url(&self, source: &str) -> Option<String> {
        match source.to_lowercase().as_str() {
            "arxiv" => Some(self.arxiv_base_url.clone()),
            "nature" => Some(self.nature_base_url.clone()),
            "science" => Some(self.science_base_url.clone()),
            "pubmed" => Some(self.pubmed_base_url.clone()),
            "ieee" => Some(self.ieee_base_url.clone()),
            "springer" => Some(self.springer_base_url.clone()),
            "elsevier" => Some(self.elsevier_base_url.clone()),
            _ => None,
        }
    }
}

/// Valida se todas as chaves necessárias estão configuradas
pub fn validate_api_keys() -> Result<(), Vec<String>> {
    let api_keys = ApiKeys::from_env();
    let mut missing_keys = Vec::new();
    
    // Verificar chaves críticas (podem ser implementadas futuramente)
    // Por enquanto, apenas arXiv está implementado e não precisa de chave
    
    if missing_keys.is_empty() {
        Ok(())
    } else {
        Err(missing_keys)
    }
}

