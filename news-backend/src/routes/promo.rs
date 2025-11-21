use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::fs as async_fs;
use uuid::Uuid;

use crate::utils::path_resolver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoArticle {
    pub id: String,
    pub site: String, // "airesearch" or "scienceai"
    pub title: String,
    pub subtitle: String,
    pub content: String,
    pub image_url: Option<String>,
    pub external_link: Option<String>,
    pub category: String,
    pub featured: bool,
    pub hidden: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromoRequest {
    pub site: String,
    pub title: String,
    pub subtitle: String,
    pub content: String,
    pub external_link: Option<String>,
    pub category: Option<String>,
    pub featured: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisibilityRequest {
    pub hidden: bool,
}

pub fn routes() -> Router {
    Router::new()
        .route("/articles", get(get_articles).post(create_article))
        .route("/articles/:id", put(update_article).delete(delete_article))
        .route("/articles/:id/visibility", put(update_visibility))
}

async fn get_articles() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match load_promo_articles().await {
        Ok(articles) => Ok(Json(serde_json::json!({
            "success": true,
            "articles": articles
        }))),
        Err(e) => {
            tracing::error!("Failed to load promo articles: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load promo articles: {}", e),
            ))
        }
    }
}

async fn create_article(mut multipart: Multipart) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut site = String::new();
    let mut title = String::new();
    let mut subtitle = String::new();
    let mut content = String::new();
    let mut external_link: Option<String> = None;
    let mut category = "featured".to_string(); // Default category
    let mut featured = true; // Default to featured for promo articles
    let mut image_data: Option<Vec<u8>> = None;
    let mut image_filename: Option<String> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Failed to parse form data: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "site" => {
                site = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read site: {}", e))
                })?;
            }
            "title" => {
                title = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read title: {}", e))
                })?;
            }
            "subtitle" => {
                subtitle = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read subtitle: {}", e))
                })?;
            }
            "content" => {
                let raw_content = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read content: {}", e))
                })?;
                
                // Normalize paragraph breaks: ensure double line breaks between paragraphs
                // This handles cases where the frontend might send single line breaks
                content = raw_content
                    .lines()
                    .collect::<Vec<&str>>()
                    .split(|line| line.trim().is_empty())
                    .filter(|paragraph| !paragraph.is_empty())
                    .map(|paragraph| paragraph.join(" ").trim().to_string())
                    .filter(|paragraph| !paragraph.is_empty())
                    .collect::<Vec<String>>()
                    .join("\n\n");
                
                println!("[PROMO DEBUG] Raw content: {:?}", raw_content);
                println!("[PROMO DEBUG] Normalized content: {:?}", content);
                println!("[PROMO DEBUG] Contains \\n\\n: {}", content.contains("\n\n"));
            }
            "external_link" => {
                let link = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read external_link: {}", e))
                })?;
                if !link.trim().is_empty() {
                    external_link = Some(link.trim().to_string());
                }
            }
            "category" => {
                category = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read category: {}", e))
                })?;
            }
            "featured" => {
                let featured_str = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read featured: {}", e))
                })?;
                featured = featured_str == "true";
            }
            "image" => {
                if let Some(filename) = field.file_name() {
                    image_filename = Some(filename.to_string());
                    image_data = Some(field.bytes().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Failed to read image data: {}", e))
                    })?.to_vec());
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    if site.trim().is_empty() || title.trim().is_empty() || subtitle.trim().is_empty() || content.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Site, title, subtitle, and content are required".to_string()));
    }

    if site != "airesearch" && site != "scienceai" {
        return Err((StatusCode::BAD_REQUEST, "Site must be 'airesearch' or 'scienceai'".to_string()));
    }

    // Generate unique ID
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Handle image upload if provided
    let mut image_url: Option<String> = None;
    if let (Some(data), Some(filename)) = (image_data, image_filename) {
        // Validate image type
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !["jpg", "jpeg", "png", "webp"].contains(&extension.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "Image must be JPG, PNG, or WebP".to_string()));
        }

        // Create promo images directory
        let images_dir = path_resolver::resolve_workspace_path("images/promo");
        if let Err(e) = fs::create_dir_all(&images_dir) {
            tracing::error!("Failed to create promo images directory: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create images directory".to_string()));
        }

        // Save image with unique filename
        let image_filename = format!("{}_{}.{}", id, chrono::Utc::now().timestamp(), extension);
        let image_path = images_dir.join(&image_filename);

        if let Err(e) = async_fs::write(&image_path, data).await {
            tracing::error!("Failed to save image: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to save image".to_string()));
        }

        image_url = Some(format!("/images/promo/{}", image_filename));
    }

    // Create promo article
    let article = PromoArticle {
        id: id.clone(),
        site: site.trim().to_string(),
        title: title.trim().to_string(),
        subtitle: subtitle.trim().to_string(),
        content: content.trim().to_string(),
        image_url,
        external_link,
        category: category.trim().to_string(),
        featured,
        hidden: false,
        created_at: now.clone(),
        updated_at: now,
    };

    // Save to storage
    match save_promo_article(&article).await {
        Ok(_) => {
            tracing::info!("Created promo article: {} for site: {}", article.title, article.site);
            Ok(Json(serde_json::json!({
                "success": true,
                "article": article
            })))
        }
        Err(e) => {
            tracing::error!("Failed to save promo article: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save article: {}", e)))
        }
    }
}

async fn update_article(
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Load existing article
    let mut articles = load_promo_articles().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load articles: {}", e))
    })?;

    let article_index = articles.iter().position(|a| a.id == id).ok_or((
        StatusCode::NOT_FOUND,
        "Article not found".to_string(),
    ))?;

    let mut article = articles[article_index].clone();

    // Parse multipart form data for updates
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Failed to parse form data: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "site" => {
                let site = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read site: {}", e))
                })?;
                if site != "airesearch" && site != "scienceai" {
                    return Err((StatusCode::BAD_REQUEST, "Site must be 'airesearch' or 'scienceai'".to_string()));
                }
                article.site = site.trim().to_string();
            }
            "title" => {
                article.title = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read title: {}", e))
                })?.trim().to_string();
            }
            "subtitle" => {
                article.subtitle = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read subtitle: {}", e))
                })?.trim().to_string();
            }
            "content" => {
                article.content = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read content: {}", e))
                })?.trim().to_string();
            }
            "external_link" => {
                let link = field.text().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Failed to read external_link: {}", e))
                })?;
                article.external_link = if link.trim().is_empty() {
                    None
                } else {
                    Some(link.trim().to_string())
                };
            }
            "image" => {
                if let Some(filename) = field.file_name() {
                    let filename = filename.to_string(); // Clone filename before moving field
                    let image_data = field.bytes().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Failed to read image data: {}", e))
                    })?.to_vec();

                    // Validate image type
                    let extension = std::path::Path::new(&filename)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if !["jpg", "jpeg", "png", "webp"].contains(&extension.as_str()) {
                        return Err((StatusCode::BAD_REQUEST, "Image must be JPG, PNG, or WebP".to_string()));
                    }

                    // Create promo images directory
                    let images_dir = path_resolver::resolve_workspace_path("images/promo");
                    if let Err(e) = fs::create_dir_all(&images_dir) {
                        tracing::error!("Failed to create promo images directory: {}", e);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create images directory".to_string()));
                    }

                    // Delete old image if exists
                    if let Some(old_image_url) = &article.image_url {
                        if let Some(old_filename) = old_image_url.strip_prefix("/images/promo/") {
                            let old_image_path = images_dir.join(old_filename);
                            let _ = fs::remove_file(old_image_path); // Ignore errors
                        }
                    }

                    // Save new image
                    let image_filename = format!("{}_{}.{}", article.id, chrono::Utc::now().timestamp(), extension);
                    let image_path = images_dir.join(&image_filename);

                    if let Err(e) = async_fs::write(&image_path, image_data).await {
                        tracing::error!("Failed to save image: {}", e);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to save image".to_string()));
                    }

                    article.image_url = Some(format!("/images/promo/{}", image_filename));
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    if article.title.is_empty() || article.subtitle.is_empty() || article.content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Title, subtitle, and content are required".to_string()));
    }

    // Update timestamp
    article.updated_at = chrono::Utc::now().to_rfc3339();

    // Update in storage
    articles[article_index] = article.clone();
    match save_all_promo_articles(&articles).await {
        Ok(_) => {
            tracing::info!("Updated promo article: {}", article.title);
            Ok(Json(serde_json::json!({
                "success": true,
                "article": article
            })))
        }
        Err(e) => {
            tracing::error!("Failed to update promo article: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update article: {}", e)))
        }
    }
}

async fn update_visibility(
    Path(id): Path<String>,
    Json(request): Json<UpdateVisibilityRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut articles = load_promo_articles().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load articles: {}", e))
    })?;

    let article_index = articles.iter().position(|a| a.id == id).ok_or((
        StatusCode::NOT_FOUND,
        "Article not found".to_string(),
    ))?;

    articles[article_index].hidden = request.hidden;
    articles[article_index].updated_at = chrono::Utc::now().to_rfc3339();

    match save_all_promo_articles(&articles).await {
        Ok(_) => {
            let action = if request.hidden { "hidden" } else { "shown" };
            tracing::info!("Promo article {} {}", articles[article_index].title, action);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Article {}", action)
            })))
        }
        Err(e) => {
            tracing::error!("Failed to update article visibility: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update visibility: {}", e)))
        }
    }
}

async fn delete_article(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut articles = load_promo_articles().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load articles: {}", e))
    })?;

    let article_index = articles.iter().position(|a| a.id == id).ok_or((
        StatusCode::NOT_FOUND,
        "Article not found".to_string(),
    ))?;

    let article = articles.remove(article_index);

    // Delete associated image if exists
    if let Some(image_url) = &article.image_url {
        if let Some(filename) = image_url.strip_prefix("/images/promo/") {
            let images_dir = path_resolver::resolve_workspace_path("images/promo");
            let image_path = images_dir.join(filename);
            let _ = fs::remove_file(image_path); // Ignore errors
        }
    }

    match save_all_promo_articles(&articles).await {
        Ok(_) => {
            tracing::info!("Deleted promo article: {}", article.title);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Article deleted successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to delete promo article: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete article: {}", e)))
        }
    }
}

// Storage functions
async fn load_promo_articles() -> Result<Vec<PromoArticle>, Box<dyn std::error::Error + Send + Sync>> {
    let promo_file = path_resolver::resolve_workspace_path("promo_articles.json");
    
    if !promo_file.exists() {
        return Ok(Vec::new());
    }

    let content = async_fs::read_to_string(promo_file).await?;
    let articles: Vec<PromoArticle> = serde_json::from_str(&content)?;
    Ok(articles)
}

async fn save_promo_article(article: &PromoArticle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut articles = load_promo_articles().await?;
    articles.push(article.clone());
    save_all_promo_articles(&articles).await
}

async fn save_all_promo_articles(articles: &[PromoArticle]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let promo_file = path_resolver::resolve_workspace_path("promo_articles.json");
    let content = serde_json::to_string_pretty(articles)?;
    async_fs::write(promo_file, content).await?;
    Ok(())
}

// Public function to get visible promo articles for a specific site
pub async fn get_visible_promo_articles(site: &str) -> Result<Vec<PromoArticle>, Box<dyn std::error::Error + Send + Sync>> {
    let articles = load_promo_articles().await?;
    let filtered: Vec<PromoArticle> = articles
        .into_iter()
        .filter(|article| article.site == site && !article.hidden && article.featured)
        .collect();
    Ok(filtered)
}
