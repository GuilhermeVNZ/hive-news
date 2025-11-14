use axum::{
    Json,
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{HashMap, hash_map::Entry},
    fs,
    path::Path as FsPath,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tokio::sync::RwLock;

use crate::utils::{article_registry::ArticleRegistry, path_resolver};

#[derive(Debug, Serialize, Clone)]
struct Article {
    id: String,
    slug: String,
    title: String,
    excerpt: String,
    article: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    author: String,
    category: String,
    #[serde(rename = "readTime")]
    read_time: u32,
    #[serde(rename = "imageCategories")]
    image_categories: Vec<String>,
    #[serde(rename = "imagePath", skip_serializing_if = "Option::is_none")]
    image_path: Option<String>,
    #[serde(rename = "isPromotional")]
    is_promotional: bool,
    featured: bool,
    hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct ArticlesQuery {
    category: Option<String>,
}

struct CategoryImages {
    files: Vec<String>,
}

struct CachedState {
    articles: Arc<Vec<Article>>,
    registry_signature: u128,
}

struct CacheSnapshot {
    articles: Arc<Vec<Article>>,
}

static AIRESEARCH_CACHE: Lazy<RwLock<Option<CachedState>>> = Lazy::new(|| RwLock::new(None));

const ARTICLE_FILES: [&str; 2] = ["article.md", "article.txt"];

const CATEGORY_KEYWORDS: &[(&str, &[&str])] = &[
    ("openai", &["openai", "gpt", "chatgpt"]),
    ("nvidia", &["nvidia"]),
    ("google", &["google", "deepmind"]),
    ("anthropic", &["anthropic", "claude"]),
    ("deepseek", &["deepseek"]),
    ("meta", &["meta", "facebook"]),
    ("x", &["x.ai", "x.com", "grok", "twitter"]),
    ("mistral", &["mistral"]),
    ("alibaba", &["alibaba", "damo", "alizila"]),
    ("microsoft", &["microsoft", "azure"]),
    ("hivehub", &["hivehub", "hive-hub"]),
    ("perplexity", &["perplexity"]),
    ("huggingface", &["huggingface", "hugging face"]),
    ("stability", &["stability", "stable diffusion"]),
    ("elevenlabs", &["elevenlabs", "eleven labs"]),
    ("character", &["character.ai", "character ai"]),
    ("inflection", &["inflection", "pi ai"]),
    ("ibm", &["ibm", "ibm research"]),
    ("apple", &["apple", "machine learning journal"]),
    ("intel", &["intel"]),
    ("amd", &["amd"]),
    ("salesforce", &["salesforce"]),
    ("stanford", &["stanford", "hai"]),
    ("berkeley", &["berkeley", "bair"]),
    ("deepmind", &["deepmind"]),
    ("venturebeat", &["venturebeat"]),
    ("verge", &["the verge", "verge"]),
    ("wired", &["wired"]),
    ("mit", &["mit", "technology review"]),
    ("nature", &["nature"]),
    ("science", &["science"]),
];

fn sanitize_categories(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|line| !line.is_empty())
        .collect()
}

fn take_non_empty(value: Option<&String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn map_source_to_category(
    source: &str,
    image_categories: &[String],
    fallback_category: Option<&str>,
) -> String {
    if let Some(first) = image_categories.first() {
        return first.clone();
    }

    if let Some(fallback) = fallback_category {
        let trimmed = fallback.trim();
        if !trimmed.is_empty() {
            return trimmed.to_lowercase();
        }
    }

    let normalized = source.to_lowercase();
    for (category, keywords) in CATEGORY_KEYWORDS {
        if keywords.iter().any(|keyword| normalized.contains(keyword)) {
            return (*category).to_string();
        }
    }

    "ai".to_string()
}

fn map_category_to_dir(category: &str) -> &'static str {
    match category {
        "ai" => "ai",
        "robotics" => "robotics",
        "science" => "science",
        "coding" => "coding",
        "crypto" => "crypto",
        "database" => "database",
        "ethics" => "ethics",
        "games" => "games",
        "hardware" => "hardware",
        "legal" => "legal",
        "network" => "network",
        "security" => "security",
        "sound" => "sound",
        "nvidia" => "hardware",
        "openai" => "ai",
        "google" => "ai",
        "anthropic" => "ai",
        "deepseek" => "ai",
        "alibaba" => "ai",
        "microsoft" => "ai",
        "meta" => "ai",
        "x" | "x.ai" | "grok" => "ai",
        "mistral" => "ai",
        "perplexity" => "ai",
        "hivehub" | "hive-hub" => "ai",
        "stability" | "stability ai" => "ai",
        "elevenlabs" | "eleven labs" => "sound",
        "character" | "character.ai" => "ai",
        "inflection" | "pi ai" => "ai",
        "ibm" | "ibm research" => "ai",
        "apple" | "machine learning journal" => "ai",
        "intel" | "amd" | "salesforce" => "ai",
        "stanford" | "berkeley" | "deepmind" => "ai",
        "venturebeat" | "verge" | "wired" | "techcrunch" => "science",
        "mit" | "nature" => "science",
        _ => "ai",
    }
}

fn generate_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn read_first_existing_file(dir: &FsPath, filenames: &[&str]) -> Option<String> {
    for filename in filenames {
        let full_path = dir.join(filename);
        match fs::read_to_string(&full_path) {
            Ok(content) if !content.trim().is_empty() => return Some(content),
            _ => continue,
        }
    }
    None
}

fn read_optional_file(dir: &FsPath, filename: &str) -> String {
    fs::read_to_string(dir.join(filename)).unwrap_or_default()
}

fn compute_excerpt(subtitle: &str, content: &str) -> String {
    let subtitle = subtitle.trim();
    if !subtitle.is_empty() {
        return subtitle.to_string();
    }

    let normalized = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    if normalized.is_empty() {
        return String::new();
    }

    if normalized.len() > 200 {
        format!("{}…", &normalized[..200])
    } else {
        normalized
    }
}

fn compute_read_time(content: &str) -> u32 {
    let words = content.split_whitespace().filter(|w| !w.is_empty()).count();
    std::cmp::max(1, (words as f64 / 200.0).ceil() as u32)
}

fn extract_category_images<'a>(
    pools: &'a mut HashMap<String, CategoryImages>,
    category_dir: &str,
    images_base_dir: &FsPath,
) -> Option<&'a mut CategoryImages> {
    match pools.entry(category_dir.to_string()) {
        Entry::Occupied(entry) => Some(entry.into_mut()),
        Entry::Vacant(entry) => {
            let dir_path = images_base_dir.join(category_dir);
            let files = fs::read_dir(&dir_path)
                .ok()?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| {
                    let lower = name.to_lowercase();
                    lower.ends_with(".jpg")
                        || lower.ends_with(".jpeg")
                        || lower.ends_with(".png")
                        || lower.ends_with(".webp")
                })
                .collect::<Vec<String>>();

            let mut files = files;
            files.sort_by(|a, b| {
                let num_a = extract_first_number(a);
                let num_b = extract_first_number(b);
                num_a.cmp(&num_b).then_with(|| a.cmp(b))
            });

            Some(entry.insert(CategoryImages { files }))
        }
    }
}

fn extract_first_number(filename: &str) -> u32 {
    let digits = filename
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>();
    digits.parse().unwrap_or(0)
}

fn parse_numeric_id(article_id: &str) -> usize {
    let numeric = article_id
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>();

    if numeric.is_empty() {
        0
    } else {
        let cleaned = numeric.replace('.', "");
        cleaned.parse::<usize>().unwrap_or(0)
    }
}

fn select_article_image(
    categories: &[String],
    images_base_dir: &FsPath,
    article_id: &str,
    pools: &mut HashMap<String, CategoryImages>,
) -> Option<String> {
    let numeric_id = parse_numeric_id(article_id);

    for category in categories {
        let dir = map_category_to_dir(category);
        let images = extract_category_images(pools, dir, images_base_dir)?;
        if images.files.is_empty() {
            continue;
        }

        let index = numeric_id % images.files.len();
        let filename = &images.files[index];
        return Some(format!("/images/{}/{}", dir, filename));
    }

    None
}

fn load_airesearch_articles() -> Result<Vec<Article>, String> {
    let registry_path = path_resolver::resolve_workspace_path("articles_registry.json");
    let images_base_dir = path_resolver::resolve_workspace_path("images");

    let registry = ArticleRegistry::load(&registry_path).map_err(|err| {
        format!(
            "Failed to load articles registry at {}: {}",
            registry_path.display(),
            err
        )
    })?;

    let mut articles_with_keys: Vec<(DateTime<Utc>, bool, String, Article)> = Vec::new();
    let mut category_pools: HashMap<String, CategoryImages> = HashMap::new();

    for metadata in registry.articles.values() {
        if !matches!(
            metadata.status,
            crate::utils::article_registry::ArticleStatus::Published
        ) {
            continue;
        }

        if metadata
            .destinations
            .as_ref()
            .map(|dests| !dests.iter().any(|d| d.eq_ignore_ascii_case("airesearch")))
            .unwrap_or(true)
        {
            continue;
        }

        if metadata.hidden.unwrap_or(false) {
            continue;
        }

        let output_dir = match metadata.output_dir.as_ref() {
            Some(path) => path_resolver::resolve_workspace_path(path),
            None => continue,
        };

        if !output_dir.is_dir() {
            continue;
        }

        let article_content = match read_first_existing_file(output_dir.as_path(), &ARTICLE_FILES) {
            Some(content) => content.trim().to_string(),
            None => continue,
        };

        if article_content.is_empty() {
            continue;
        }

        let subtitle_raw = read_optional_file(output_dir.as_path(), "subtitle.txt");
        let categories_raw = read_optional_file(output_dir.as_path(), "image_categories.txt");
        let slug_raw = read_optional_file(output_dir.as_path(), "slug.txt");
        let source_raw = read_optional_file(output_dir.as_path(), "source.txt");

        let title = take_non_empty(metadata.generated_title.as_ref())
            .or_else(|| take_non_empty(metadata.original_title.as_ref()))
            .unwrap_or_else(|| metadata.title.clone());

        let fallback_slug = generate_slug(&title);
        let slug = slug_raw.trim();
        let slug = if slug.is_empty() {
            fallback_slug.clone()
        } else {
            slug.to_string()
        };

        let categories_list = sanitize_categories(&categories_raw);
        let source_trimmed = source_raw.trim();

        let primary_category = map_source_to_category(
            source_trimmed,
            &categories_list,
            metadata.category.as_deref(),
        );

        let mut normalized_categories = Vec::new();
        for cat in categories_list {
            if !normalized_categories.contains(&cat) {
                normalized_categories.push(cat);
            }
        }

        if !normalized_categories.contains(&primary_category) {
            normalized_categories.insert(0, primary_category.clone());
        }

        if normalized_categories.is_empty() {
            normalized_categories.push(primary_category.clone());
        }

        let image_path = select_article_image(
            &normalized_categories,
            images_base_dir.as_path(),
            &metadata.id,
            &mut category_pools,
        );

        let author = if source_trimmed.is_empty() {
            "AI Research".to_string()
        } else {
            source_trimmed.to_string()
        };

        let subtitle_trimmed = subtitle_raw.trim().to_string();

        let published_at_dt = metadata
            .published_at
            .or(metadata.filtered_at)
            .or(metadata.collected_at)
            .unwrap_or_else(Utc::now);

        let published_at = published_at_dt.to_rfc3339();
        let featured = metadata.featured.unwrap_or(false);
        let hidden = metadata.hidden.unwrap_or(false);

        let article = Article {
            id: metadata.id.clone(),
            slug,
            title,
            excerpt: compute_excerpt(&subtitle_trimmed, &article_content),
            article: article_content.clone(),
            published_at,
            author,
            category: primary_category,
            read_time: compute_read_time(&article_content),
            image_categories: normalized_categories.clone(),
            image_path,
            is_promotional: false,
            featured,
            hidden,
        };

        articles_with_keys.push((published_at_dt, featured, metadata.id.clone(), article));
    }

    articles_with_keys.sort_by(|a, b| {
        let date_cmp = b.0.cmp(&a.0);
        if date_cmp != std::cmp::Ordering::Equal {
            return date_cmp;
        }
        match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.2.cmp(&a.2),
        }
    });

    let articles = articles_with_keys
        .into_iter()
        .map(|(_, _, _, article)| article)
        .collect();

    Ok(articles)
}

fn registry_signature(registry_path: &FsPath) -> Result<u128, String> {
    let metadata = fs::metadata(registry_path).map_err(|err| {
        format!(
            "Failed to read metadata for {}: {}",
            registry_path.display(),
            err
        )
    })?;

    let modified = metadata.modified().map_err(|err| {
        format!(
            "Failed to read modification time for {}: {}",
            registry_path.display(),
            err
        )
    })?;

    let duration = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));

    Ok(duration.as_nanos())
}

async fn get_cache_snapshot() -> Result<CacheSnapshot, String> {
    let registry_path = path_resolver::resolve_workspace_path("articles_registry.json");
    let signature = registry_signature(registry_path.as_path())?;

    {
        let cache_guard = AIRESEARCH_CACHE.read().await;
        if let Some(state) = &*cache_guard
            && state.registry_signature == signature
        {
            return Ok(CacheSnapshot {
                articles: state.articles.clone(),
            });
        }
    }

    let mut cache_guard = AIRESEARCH_CACHE.write().await;

    if let Some(state) = &*cache_guard
        && state.registry_signature == signature
    {
        return Ok(CacheSnapshot {
            articles: state.articles.clone(),
        });
    }

    let articles_vec = load_airesearch_articles()?;

    let state = CachedState {
        articles: Arc::new(articles_vec),
        registry_signature: signature,
    };

    let snapshot = CacheSnapshot {
        articles: state.articles.clone(),
    };

    *cache_guard = Some(state);

    Ok(snapshot)
}

pub async fn warm_cache() -> Result<(), String> {
    let _ = get_cache_snapshot().await?;
    Ok(())
}

pub async fn get_articles(
    Query(query): Query<ArticlesQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let snapshot = get_cache_snapshot()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    let articles_arc = snapshot.articles;

    let filtered: Vec<Article> = if let Some(category) = query.category {
        let filter = category.to_lowercase();
        if filter == "all" {
            articles_arc.as_ref().clone()
        } else {
            articles_arc
                .iter()
                .filter(|article| {
                    article.category == filter
                        || article
                            .image_categories
                            .iter()
                            .any(|cat| cat.eq_ignore_ascii_case(&filter))
                })
                .cloned()
                .collect()
        }
    } else {
        articles_arc.as_ref().clone()
    };

    let featured_count = filtered.iter().filter(|article| article.featured).count();

    eprintln!(
        "[AIResearch API] Returning {} articles, {} featured",
        filtered.len(),
        featured_count
    );

    Ok(Json(json!({ "articles": filtered })))
}

pub async fn get_article_by_slug(
    AxumPath(slug): AxumPath<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let normalized = slug.to_lowercase();
    let snapshot = get_cache_snapshot()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    if let Some(article) = snapshot.articles.iter().find(|article| {
        article.slug.eq_ignore_ascii_case(&normalized)
            || article.id.eq_ignore_ascii_case(&normalized)
    }) {
        return Ok(Json(json!({ "article": article })));
    }

    Err((StatusCode::NOT_FOUND, "Article not found".to_string()))
}
