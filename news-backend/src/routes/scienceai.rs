use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::utils::article_registry::ArticleRegistry;

#[derive(Debug, Serialize)]
struct Article {
    id: String,
    slug: String,
    title: String,
    category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "imageCarousel")]
    image_carousel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "imageArticle")]
    image_article: Option<String>,
    excerpt: String,
    content: String,
    date: String,
    author: String,
    #[serde(rename = "readTime")]
    read_time: u32,
    featured: bool,
    #[serde(rename = "imageCategories")]
    image_categories: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Category {
    name: String,
    slug: String,
    #[serde(rename = "latestDate")]
    latest_date: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
pub struct ArticlesQuery {
    category: Option<String>,
}

/// GET /api/articles - Returns articles for ScienceAI frontend
pub async fn get_articles(
    Query(query): Query<ArticlesQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    eprintln!("[ScienceAI API] GET /api/articles - category: {:?}", query.category);
    
    // Load articles from registry
    let registry_path = crate::utils::path_resolver::resolve_workspace_path("articles_registry.json");
    let registry = ArticleRegistry::load(&registry_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load registry: {}", e)))?;

    // Filter articles for ScienceAI (status=Published, destinations contains "scienceai", not hidden)
    let mut articles: Vec<Article> = registry
        .articles
        .values()
        .filter(|m| {
            // Check if published
            if !matches!(m.status, crate::utils::article_registry::ArticleStatus::Published) {
                return false;
            }

            // Check if has scienceai destination
            if let Some(destinations) = &m.destinations {
                let has_scienceai = destinations.iter().any(|d| {
                    d.to_lowercase() == "scienceai"
                });
                if !has_scienceai {
                    return false;
                }
            } else {
                return false;
            }

            // Check if not hidden
            if m.hidden.unwrap_or(false) {
                return false;
            }

            true
        })
        .filter_map(|m| {
            // Read title from filesystem
            let output_dir = m.output_dir.as_ref()?;
            let full_path = crate::utils::path_resolver::resolve_workspace_path(output_dir);
            let title_path = full_path.join("title.txt");
            let title = std::fs::read_to_string(title_path).ok()?;

            // Generate slug from title
            let slug = title
                .to_lowercase()
                .chars()
                .filter(|&c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '-')
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("-");

            // Get date
            let date = m.published_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

            // Get author (default to ScienceAI Team as we don't have source field)
            let author = "ScienceAI Team".to_string();

            // Read excerpt from subtitle.txt
            let subtitle_path = full_path.join("subtitle.txt");
            let excerpt = std::fs::read_to_string(subtitle_path)
                .unwrap_or_else(|_| "Discover the latest in AI research and technology.".to_string());

            // Extract category from registry or default to "ai"
            let category = m.category
                .as_ref()
                .map(|c| c.to_lowercase())
                .unwrap_or_else(|| "ai".to_string());

            // Derive image categories from category field
            // ScienceAI uses "ai" images by default
            let image_categories = vec![category.clone()];

            // Image selection logic (matching AIResearch)
            // Carousel uses first category, deterministic
            // Feed uses second category (or first if only one), non-repeating via article ID
            let image_carousel = if !image_categories.is_empty() {
                let cat = &image_categories[0];
                let cat_dir = cat.to_lowercase();
                // Use article ID for deterministic selection
                let id_hash: u64 = m.id.chars()
                    .filter(|c| c.is_numeric())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);
                let max_images = if cat_dir == "ai" { 59 } else { 20 };
                let image_num = (id_hash % max_images) + 1;
                Some(format!("/images/{}/{cat_dir}_{image_num}.jpg", cat_dir))
            } else {
                Some("/images/ai/ai_1.jpg".to_string())
            };

            // Feed image uses second category if available
            // Otherwise uses same category but different hash offset for variety
            let image_feed = if image_categories.len() > 1 {
                let cat = &image_categories[1];
                let cat_dir = cat.to_lowercase();
                let id_hash: u64 = m.id.chars()
                    .filter(|c| c.is_numeric())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);
                let max_images = if cat_dir == "ai" { 59 } else { 20 };
                let image_num = (id_hash % max_images) + 1;
                Some(format!("/images/{}/{cat_dir}_{image_num}.jpg", cat_dir))
            } else if !image_categories.is_empty() {
                // Use same category but with different offset to get different image
                let cat = &image_categories[0];
                let cat_dir = cat.to_lowercase();
                let id_hash: u64 = m.id.chars()
                    .filter(|c| c.is_numeric())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);
                let max_images = if cat_dir == "ai" { 59 } else { 20 };
                // Add prime number offset to ensure different image from carousel
                let image_num = ((id_hash + 17) % max_images) + 1;
                Some(format!("/images/{}/{cat_dir}_{image_num}.jpg", cat_dir))
            } else {
                Some("/images/ai/ai_1.jpg".to_string())
            };

            Some(Article {
                id: m.id.clone(),
                slug,
                title: title.trim().to_string(),
                category,
                image: image_feed,
                image_carousel: image_carousel.clone(),
                image_article: image_carousel, // Same as carousel
                excerpt: excerpt.trim().to_string(),
                content: "".to_string(), // Don't load full content for list
                date,
                author,
                read_time: 5, // Default estimate
                featured: m.featured.unwrap_or(false),
                image_categories,
            })
        })
        .collect();

    // Filter by category if provided
    if let Some(category) = query.category {
        if category != "all" {
            articles.retain(|a| a.category == category);
        }
    }

    // Sort: featured first, then by date (newest first)
    articles.sort_by(|a, b| {
        match (a.featured, b.featured) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.date.cmp(&a.date),
        }
    });

    eprintln!("[ScienceAI API] Returning {} articles", articles.len());

    Ok(Json(serde_json::json!({ "articles": articles })))
}

/// GET /api/categories - Returns categories for ScienceAI frontend
pub async fn get_categories() -> Result<impl IntoResponse, (StatusCode, String)> {
    // For now, return a static list of categories
    // TODO: Generate dynamically from articles
    let categories = vec![
        Category {
            name: "OpenAI".to_string(),
            slug: "openai".to_string(),
            latest_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            icon: "Brain".to_string(),
        },
        Category {
            name: "Google".to_string(),
            slug: "google".to_string(),
            latest_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            icon: "Search".to_string(),
        },
        Category {
            name: "Anthropic".to_string(),
            slug: "anthropic".to_string(),
            latest_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            icon: "Sparkles".to_string(),
        },
        Category {
            name: "DeepSeek".to_string(),
            slug: "deepseek".to_string(),
            latest_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            icon: "Target".to_string(),
        },
        Category {
            name: "NVIDIA".to_string(),
            slug: "nvidia".to_string(),
            latest_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            icon: "Cpu".to_string(),
        },
    ];

    Ok(Json(serde_json::json!({ "categories": categories })))
}

