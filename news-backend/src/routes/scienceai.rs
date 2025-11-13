use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::Path,
    sync::Mutex,
};

use crate::utils::{article_registry::ArticleRegistry, path_resolver};

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

struct ImagePool {
    all_images: Vec<String>,
    available_images: VecDeque<String>,
}

impl ImagePool {
    fn new(all_images: Vec<String>) -> Self {
        Self {
            available_images: VecDeque::from(all_images.clone()),
            all_images,
        }
    }

    fn refill(&mut self) {
        self.available_images = VecDeque::from(self.all_images.clone());
    }
}

lazy_static! {
    static ref IMAGE_POOLS: Mutex<HashMap<String, ImagePool>> = Mutex::new(HashMap::new());
}

fn map_category_to_dir(category: &str) -> &'static str {
    let lower = category.trim().to_lowercase();
    match lower.as_str() {
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
        "techcrunch" | "venturebeat" | "verge" | "wired" => "science",
        "mit" | "nature" | "menlo" => "science",
        _ => "ai",
    }
}

fn load_image_files(category_dir: &Path) -> Option<Vec<String>> {
    let mut files: Vec<String> = fs::read_dir(category_dir)
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
        .collect();

    fn extract_number(filename: &str) -> u32 {
        let mut digits = String::new();
        for ch in filename.chars() {
            if ch.is_ascii_digit() {
                digits.push(ch);
            } else if !digits.is_empty() {
                break;
            }
        }
        digits.parse().unwrap_or(0)
    }

    files.sort_by(|a, b| {
        let na = extract_number(a);
        let nb = extract_number(b);
        na.cmp(&nb).then_with(|| a.cmp(b))
    });

    if files.is_empty() { None } else { Some(files) }
}

fn ensure_image_pool(category_dir: &str, images_base_dir: &Path) -> Option<Vec<String>> {
    let mut pools = IMAGE_POOLS.lock().ok()?;
    if !pools.contains_key(category_dir) {
        let dir_path = images_base_dir.join(category_dir);
        let files = load_image_files(&dir_path)?;
        pools.insert(category_dir.to_string(), ImagePool::new(files));
    } else if let Some(pool) = pools.get_mut(category_dir) {
        let dir_path = images_base_dir.join(category_dir);
        if let Some(files) = load_image_files(&dir_path).filter(|files| *files != pool.all_images) {
            *pool = ImagePool::new(files);
        }
    }

    pools.get(category_dir).map(|pool| pool.all_images.clone())
}

fn fetch_next_feed_image(
    category_dir: &str,
    images_base_dir: &Path,
    used_paths: &HashSet<String>,
) -> Option<String> {
    let mut pools = IMAGE_POOLS.lock().ok()?;
    if !pools.contains_key(category_dir) {
        let dir_path = images_base_dir.join(category_dir);
        let files = load_image_files(&dir_path)?;
        pools.insert(category_dir.to_string(), ImagePool::new(files));
    }

    let pool = pools.get_mut(category_dir)?;
    if pool.all_images.is_empty() {
        return None;
    }

    if pool.available_images.is_empty() {
        pool.refill();
    }

    let mut rotations = 0usize;
    let total = pool.available_images.len();

    while rotations < total {
        if let Some(candidate) = pool.available_images.pop_front() {
            let full_path = format!("/images/{}/{}", category_dir, candidate);
            if !used_paths.contains(&full_path) {
                return Some(candidate);
            } else {
                pool.available_images.push_back(candidate);
                rotations += 1;
            }
        } else {
            break;
        }
    }

    pool.available_images.pop_front()
}

fn hash_identifier(input: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in input.as_bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(u64::from(*byte));
    }
    hash
}

fn select_deterministic_image(
    categories: &[String],
    images_base_dir: &Path,
    article_id: &str,
) -> Option<String> {
    let mut priority = Vec::new();
    if let Some(first) = categories.first() {
        priority.push(first.as_str());
    }
    if let Some(third) = categories.get(2) {
        priority.push(third.as_str());
    }
    if priority.is_empty() {
        priority.push("ai");
    }

    for category in priority {
        let dir = map_category_to_dir(category);
        if let Some(images) = ensure_image_pool(dir, images_base_dir) {
            if images.is_empty() {
                continue;
            }
            let index = (hash_identifier(article_id) as usize) % images.len();
            let file = &images[index];
            return Some(format!("/images/{}/{}", dir, file));
        }
    }
    None
}

fn select_feed_image(
    categories: &[String],
    images_base_dir: &Path,
    article_id: &str,
    used_paths: &mut HashSet<String>,
) -> Option<String> {
    let mut priority = Vec::new();
    if let Some(second) = categories.get(1) {
        priority.push(second.as_str());
    }
    if let Some(third) = categories.get(2) {
        priority.push(third.as_str());
    }
    if let Some(first) = categories.first() {
        priority.push(first.as_str());
    }
    if priority.is_empty() {
        priority.push("ai");
    }

    for category in priority {
        let dir = map_category_to_dir(category);
        if let Some(filename) = fetch_next_feed_image(dir, images_base_dir, used_paths) {
            let path = format!("/images/{}/{}", dir, filename);
            used_paths.insert(path.clone());
            return Some(path);
        }
    }

    select_deterministic_image(categories, images_base_dir, article_id)
}

fn read_article_content(article_dir: &Path) -> Option<String> {
    const ARTICLE_FILES: &[&str] = &["article.md", "article.txt"];
    for filename in ARTICLE_FILES {
        let path = article_dir.join(filename);
        if let Some(content) = fs::read_to_string(&path)
            .ok()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
        {
            return Some(content);
        }
    }
    None
}

fn compute_excerpt(subtitle: &str, content: &str) -> String {
    if !subtitle.trim().is_empty() {
        return subtitle.trim().to_string();
    }
    let summary = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    summary
        .chars()
        .take(200)
        .collect::<String>()
        .trim()
        .to_string()
}

fn resolve_article_slug(article_dir: &Path, title: &str) -> String {
    let slug_path = article_dir.join("slug.txt");
    if let Ok(slug_content) = fs::read_to_string(slug_path) {
        let trimmed = slug_content.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn read_image_categories(article_dir: &Path) -> Vec<String> {
    let categories_path = article_dir.join("image_categories.txt");
    if let Ok(contents) = fs::read_to_string(categories_path) {
        let categories: Vec<String> = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.to_lowercase())
            .collect();
        if !categories.is_empty() {
            return categories;
        }
    }
    Vec::new()
}

fn load_scienceai_articles() -> Result<Vec<Article>, String> {
    let registry_path = path_resolver::resolve_workspace_path("articles_registry.json");
    let images_base_dir = path_resolver::resolve_workspace_path("images");

    let registry = ArticleRegistry::load(&registry_path).map_err(|err| {
        format!(
            "Failed to load articles registry at {}: {}",
            registry_path.display(),
            err
        )
    })?;

    let mut used_feed_images = HashSet::new();
    let mut articles = Vec::new();

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
            .map(|dests| !dests.iter().any(|d| d.eq_ignore_ascii_case("scienceai")))
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

        let title_path = output_dir.join("title.txt");
        let title = match fs::read_to_string(&title_path) {
            Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => continue,
        };

        let article_content = match read_article_content(&output_dir) {
            Some(content) => content,
            None => continue,
        };

        let subtitle_path = output_dir.join("subtitle.txt");
        let subtitle = fs::read_to_string(subtitle_path).unwrap_or_default();

        let source_path = output_dir.join("source.txt");
        let source = fs::read_to_string(source_path)
            .unwrap_or_else(|_| "ScienceAI Team".to_string())
            .trim()
            .to_string();

        let mut image_categories = read_image_categories(&output_dir);
        let primary_category = image_categories.first().cloned().unwrap_or_else(|| {
            metadata
                .category
                .clone()
                .unwrap_or_else(|| "ai".to_string())
        });
        let primary_category = primary_category.to_lowercase();

        if image_categories.is_empty() {
            image_categories.push(primary_category.clone());
        }

        let slug = resolve_article_slug(&output_dir, &title);
        let excerpt = compute_excerpt(&subtitle, &article_content);

        let date = metadata
            .published_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

        let read_time = {
            let words = article_content
                .split_whitespace()
                .filter(|w| !w.is_empty())
                .count();
            std::cmp::max(1, (words as f64 / 200.0).ceil() as u32)
        };

        let feed_image = select_feed_image(
            &image_categories,
            &images_base_dir,
            &metadata.id,
            &mut used_feed_images,
        );
        let deterministic_image =
            select_deterministic_image(&image_categories, &images_base_dir, &metadata.id);

        let article = Article {
            id: metadata.id.clone(),
            slug,
            title,
            category: primary_category,
            image: feed_image.clone(),
            image_carousel: deterministic_image.clone(),
            image_article: deterministic_image,
            excerpt,
            content: article_content.clone(),
            date,
            author: if source.is_empty() {
                "ScienceAI Team".to_string()
            } else {
                source
            },
            read_time,
            featured: metadata.featured.unwrap_or(false),
            image_categories: image_categories.clone(),
        };

        articles.push(article);
    }

    articles.sort_by(|a, b| match (a.featured, b.featured) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.date.cmp(&a.date),
    });

    Ok(articles)
}

fn icon_for_category(slug: &str) -> &'static str {
    match slug {
        "openai" => "Brain",
        "google" => "Search",
        "anthropic" => "Sparkles",
        "deepseek" => "Target",
        "nvidia" => "Cpu",
        "microsoft" => "Monitor",
        "meta" => "Sparkles",
        "x" => "MessageSquare",
        "mistral" => "Wind",
        "alibaba" => "Package",
        "perplexity" => "Search",
        "hivehub" => "Home",
        "stability" => "Image",
        "elevenlabs" => "Volume2",
        "character" => "MessageSquare",
        "inflection" => "Sparkles",
        "ibm" => "Database",
        "apple" => "Laptop",
        "intel" => "Cpu",
        "amd" => "Cpu",
        "salesforce" => "Briefcase",
        "stanford" | "berkeley" | "deepmind" => "GraduationCap",
        "techcrunch" | "venturebeat" | "verge" | "wired" => "Newspaper",
        "mit" | "nature" | "science" => "BookOpen",
        "menlo" => "TrendingUp",
        "robotics" => "Bot",
        "security" => "Shield",
        "sound" => "Volume2",
        _ => "Circle",
    }
}

fn title_case(input: &str) -> String {
    let lower = input.to_lowercase();
    lower
        .split(['-', '_', ' '])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// GET /api/articles - Returns articles for ScienceAI frontend
pub async fn get_articles(
    Query(query): Query<ArticlesQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    eprintln!(
        "[ScienceAI API] GET /api/articles - category: {:?}",
        query.category
    );

    let mut articles = load_scienceai_articles().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load ScienceAI articles: {err}"),
        )
    })?;

    if let Some(category) = query.category
        && category.to_lowercase() != "all"
    {
        let filter = category.to_lowercase();
        articles.retain(|article| article.category == filter);
    }

    eprintln!("[ScienceAI API] Returning {} articles", articles.len());

    Ok(Json(serde_json::json!({ "articles": articles })))
}

/// GET /api/categories - Returns categories for ScienceAI frontend
pub async fn get_categories() -> Result<impl IntoResponse, (StatusCode, String)> {
    let articles = load_scienceai_articles().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load ScienceAI articles: {err}"),
        )
    })?;

    #[derive(Debug)]
    struct CategoryInfo {
        name: String,
        slug: String,
        latest_date: String,
        icon: &'static str,
        is_featured: bool,
    }

    let mut map: HashMap<String, CategoryInfo> = HashMap::new();

    for article in &articles {
        let slug = article.category.to_lowercase();
        let info = map.entry(slug.clone()).or_insert_with(|| CategoryInfo {
            name: title_case(&slug),
            slug: slug.clone(),
            latest_date: article.date.clone(),
            icon: icon_for_category(&slug),
            is_featured: article.featured,
        });

        if article.date > info.latest_date {
            info.latest_date = article.date.clone();
        }
        if article.featured {
            info.is_featured = true;
        }
    }

    let mut infos: Vec<CategoryInfo> = map.into_values().collect();
    infos.sort_by(|a, b| match (a.is_featured, b.is_featured) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.latest_date.cmp(&a.latest_date),
    });

    let categories: Vec<Category> = infos
        .into_iter()
        .take(5)
        .map(|info| Category {
            name: info.name,
            slug: info.slug,
            latest_date: info.latest_date,
            icon: info.icon.to_string(),
        })
        .collect();

    Ok(Json(serde_json::json!({ "categories": categories })))
}
