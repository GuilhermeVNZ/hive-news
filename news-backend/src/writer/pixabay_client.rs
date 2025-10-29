// Pixabay API Client with Semantic Re-Ranking
// Searches multiple images and uses semantic similarity to find the best match

use serde::Deserialize;
use anyhow::{Result, Context};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PixabayResponse {
    total: u32,
    total_hits: u32,
    hits: Vec<Hit>,
}

#[derive(Debug, Deserialize)]
struct Hit {
    id: u32,
    #[serde(rename = "pageURL")]
    page_url: String,
    #[serde(rename = "webformatURL")]
    webformat_url: String,
    #[serde(rename = "webformatWidth")]
    webformat_width: u32,
    #[serde(rename = "webformatHeight")]
    webformat_height: u32,
    #[serde(rename = "largeImageURL")]
    large_image_url: String,
    #[serde(rename = "previewURL")]
    preview_url: String,
    tags: String,
    // Allow unknown fields to prevent parsing errors
    #[serde(flatten)]
    _extra: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct PixabayImage {
    pub id: u32,
    pub url: String,
    pub tags: Vec<String>,
    pub width: u32,
    pub height: u32,
    pub page_url: String,
}

pub struct PixabayClient {
    api_key: String,
}

impl PixabayClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Busca múltiplas imagens no Pixabay com as keywords fornecidas
    /// 
    /// Args:
    /// - keywords: Termos de busca
    /// - page: Número da página (1-20) para variar resultados entre artigos
    /// 
    /// Returns: Vec de PixabayImage com até 50 imagens candidatas
    pub async fn search_multiple_images(&self, keywords: &str, page: u32) -> Result<Vec<PixabayImage>> {
        use reqwest::Client;
        
        let client = Client::new();
        
        // API endpoint: https://pixabay.com/api/
        // Buscar até 50 imagens para ter várias opções para re-ranking
        // Varia o 'page' para evitar imagens repetidas entre artigos
        let url = format!(
            "https://pixabay.com/api/?key={}&q={}&image_type=photo&orientation=horizontal&per_page=50&page={}&safesearch=true",
            self.api_key,
            urlencoding::encode(keywords),
            page
        );
        
        println!("  🔍 Searching Pixabay for: '{}' (page {}, up to 50 images)", keywords, page);
        
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to request Pixabay API")?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("Pixabay API error: HTTP {}", status));
        }
        
        let pixabay_response: PixabayResponse = response
            .json()
            .await
            .context("Failed to parse Pixabay response")?;
        
        println!("  ✅ Found {} images on Pixabay", pixabay_response.total_hits);
        
        let images: Vec<PixabayImage> = pixabay_response.hits
            .into_iter()
            .map(|hit| PixabayImage {
                id: hit.id,
                url: hit.large_image_url,
                tags: hit.tags.split(", ").map(|s| s.trim().to_string()).collect(),
                width: hit.webformat_width,
                height: hit.webformat_height,
                page_url: hit.page_url,
            })
            .collect();
        
        Ok(images)
    }

    /// Re-ranqueia imagens usando relevância semântica baseada em tags
    /// 
    /// Estratégia:
    /// 1. Extrai keywords do artigo (título + principais termos)
    /// 2. Compara tags das imagens com keywords do artigo
    /// 3. Retorna imagens ordenadas por similaridade
    pub fn rerank_images(
        &self,
        images: Vec<PixabayImage>,
        article_keywords: &[String],
    ) -> Vec<PixabayImage> {
        println!("  🎯 Re-ranking {} images based on semantic relevance", images.len());
        
        let mut scored_images: Vec<(PixabayImage, f32)> = images
            .into_iter()
            .map(|img| {
                let score = self.calculate_semantic_score(&img, article_keywords);
                (img, score)
            })
            .collect();
        
        // Ordenar por score (maior = mais relevante)
        scored_images.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Retornar top 3 mais relevantes
        let top_images: Vec<PixabayImage> = scored_images
            .into_iter()
            .take(3)
            .map(|(img, score)| {
                println!("    🖼️  Image {} (score: {:.2}): tags={}", 
                    img.id, score, img.tags.join(", "));
                img
            })
            .collect();
        
        top_images
    }

    /// Calcula score de similaridade semântica baseado em matching de palavras
    fn calculate_semantic_score(&self, image: &PixabayImage, article_keywords: &[String]) -> f32 {
        let mut score = 0.0;
        
        // Normalizar keywords do artigo e tags da imagem para lowercase
        let article_keywords_lower: Vec<String> = article_keywords
            .iter()
            .map(|k| k.to_lowercase())
            .collect();
        
        let image_tags_lower: Vec<String> = image.tags
            .iter()
            .map(|t| t.to_lowercase())
            .collect();
        
        // Contar matches
        for keyword in &article_keywords_lower {
            for tag in &image_tags_lower {
                // Match exato = +2.0 pontos
                if tag == keyword {
                    score += 2.0;
                }
                // Substring match = +1.0 pontos
                else if tag.contains(keyword) || keyword.contains(tag) {
                    score += 1.0;
                }
                // Palavras similares (prefixo comum) = +0.5 pontos
                if tag.len() > 3 && keyword.len() > 3 {
                    if tag.chars().take(3).eq(keyword.chars().take(3)) {
                        score += 0.5;
                    }
                }
            }
        }
        
        score
    }

    /// Escolhe a melhor imagem com base em:
    /// 1. Relevância semântica (já calculada)
    /// 2. Qualidade (resolução e proporção)
    pub fn select_best_image(&self, images: Vec<PixabayImage>) -> Option<PixabayImage> {
        if images.is_empty() {
            return None;
        }
        
        // Para agora, retornar a primeira (já vem ordenada por relevância)
        // Pode adicionar mais lógica de qualidade aqui depois
        Some(images[0].clone())
    }

    /// Baixa uma imagem da URL fornecida e salva no diretório de output
    /// 
    /// Returns: PathBuf do arquivo salvo
    pub async fn download_and_save_image(
        &self,
        image_url: &str,
        output_dir: &Path,
        filename: &str,
    ) -> Result<std::path::PathBuf> {
        use reqwest::Client;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        let client = Client::new();
        
        println!("  ⬇️  Downloading image from Pixabay...");
        
        let response = client
            .get(image_url)
            .send()
            .await
            .context(format!("Failed to download image from {}", image_url))?;
        
        let bytes = response
            .bytes()
            .await
            .context("Failed to get image bytes")?;
        
        let file_path = output_dir.join(filename);
        
        let mut file = File::create(&file_path)
            .await
            .context(format!("Failed to create file: {}", file_path.display()))?;
        
        file.write_all(&bytes)
            .await
            .context(format!("Failed to write image to {}", file_path.display()))?;
        
        println!("  ✅ Image saved: {}", file_path.display());
        
        Ok(file_path)
    }
}
