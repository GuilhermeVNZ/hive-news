use axum::{
    Json,
    extract::{Path as AxumPath, Query},
    http::{StatusCode, HeaderMap, HeaderValue},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
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
    #[serde(rename = "linkedinPost", skip_serializing_if = "Option::is_none")]
    linkedin_post: Option<String>,
    #[serde(rename = "xPost", skip_serializing_if = "Option::is_none")]
    x_post: Option<String>,
    #[serde(rename = "sourceUrl", skip_serializing_if = "Option::is_none")]
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArticlesQuery {
    category: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

struct CategoryImages {
    files: Vec<String>,
    used_indices: HashSet<usize>,
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
    ("quantum_computing", &["quantum computing", "quantum computer", "quantum", "qubit", "qiskit", "ibm quantum", "quantum algorithm", "quantum processor", "quantum hardware", "quantum error correction", "quantum supremacy", "quantum advantage"]),
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

// Valid categories for articles (must match prompts and frontend)
const VALID_CATEGORIES: &[&str] = &[
    "ai", "coding", "crypto", "data", "ethics", "games", "hardware",
    "legal", "network", "quantum_computing", "robotics", "science",
    "security", "sound",
];

fn sanitize_categories(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|line| !line.is_empty())
        .filter(|line| {
            // Split by comma if present, then validate each category
            line.split(',')
                .map(|cat| cat.trim().to_lowercase())
                .filter(|cat| !cat.is_empty())
                .any(|cat| VALID_CATEGORIES.contains(&cat.as_str()))
        })
        .flat_map(|line| {
            // Split comma-separated categories
            line.split(',')
                .map(|cat| cat.trim().to_lowercase())
                .filter(|cat| !cat.is_empty())
                .filter(|cat| VALID_CATEGORIES.contains(&cat.as_str()))
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<String>>()
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
    // 1) PRIORIDADE: Se DeepSeek marcou explicitamente como quantum_computing em QUALQUER posição,
    // usamos isso como categoria primária (mesmo que não seja a primeira da lista).
    // Verificar variações: "quantum_computing", "quantum computing", "quantum", "quantum-computing"
    if image_categories.iter().any(|c| {
        let c_lower = c.to_lowercase().replace("_", " ").replace("-", " ");
        c_lower.contains("quantum") && (c_lower.contains("computing") || c_lower == "quantum")
    }) {
        return "quantum_computing".to_string();
    }

    // 2) Verificar se a primeira categoria é uma variação de quantum computing
    if let Some(first) = image_categories.first() {
        let first_lower = first.to_lowercase().replace("_", " ").replace("-", " ");
        if first_lower.contains("quantum") && (first_lower.contains("computing") || first_lower == "quantum") {
            return "quantum_computing".to_string();
        }
        // Se não for quantum, retornar a primeira categoria normalmente
        return first.clone();
    }

    if let Some(fallback) = fallback_category {
        let trimmed = fallback.trim().to_lowercase();
        if !trimmed.is_empty() {
            // Verificar se fallback é quantum computing
            let fallback_normalized = trimmed.replace("_", " ").replace("-", " ");
            if fallback_normalized.contains("quantum") && (fallback_normalized.contains("computing") || fallback_normalized == "quantum") {
                return "quantum_computing".to_string();
            }
            return trimmed;
        }
    }

    // 3) Verificar keywords de quantum no source
    let normalized = source.to_lowercase();
    if normalized.contains("quantum") && (normalized.contains("computing") || normalized.contains("computer")) {
        return "quantum_computing".to_string();
    }

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
        "data" => "data",
        "ethics" => "ethics",
        "games" => "games",
        "hardware" => "hardware",
        "legal" => "legal",
        "network" => "network",
        "security" => "security",
        "sound" => "sound",
        "quantum_computing" | "quantum" => "quantum_computing",
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
    // Calculate reading time based on 200 words per minute
    // Then scale to fit between 2-4 minutes based on content length
    let base_time = words as f64 / 200.0;
    
    // Scale the time to fit between 2-4 minutes
    // Articles with 400 words or less: 2 minutes
    // Articles with 800 words or more: 4 minutes
    // Linear interpolation for values in between
    if base_time <= 2.0 {
        2
    } else if base_time >= 4.0 {
        4
    } else {
        // Linear interpolation: map base_time (2.0-4.0) to output (2-4)
        let ratio = (base_time - 2.0) / 2.0; // 0.0 to 1.0
        let scaled = 2.0 + (ratio * 2.0); // 2.0 to 4.0
        scaled.ceil() as u32
    }
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

            Some(entry.insert(CategoryImages { 
                files,
                used_indices: HashSet::new(),
            }))
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

        // Try to find an unused image
        let mut index = numeric_id % images.files.len();
        let mut attempts = 0;
        let max_attempts = images.files.len();
        
        // If all images are used, reset the pool for this category
        if images.used_indices.len() >= images.files.len() {
            images.used_indices.clear();
        }
        
        // Find an unused image, starting from the calculated index
        while images.used_indices.contains(&index) && attempts < max_attempts {
            index = (index + 1) % images.files.len();
            attempts += 1;
        }
        
        // Mark this image as used
        images.used_indices.insert(index);
        let filename = &images.files[index];
        
        // Priorizar WebP: tentar encontrar versão WebP primeiro
        let webp_filename = filename
            .replace(".jpg", ".webp")
            .replace(".jpeg", ".webp")
            .replace(".png", ".webp");
        
        let final_filename = if images.files.iter().any(|f| f.eq_ignore_ascii_case(&webp_filename)) {
            &webp_filename
        } else {
            filename
        };
        
        return Some(format!("/images/{}/{}", dir, final_filename));
    }

    None
}

fn load_airesearch_articles() -> Result<Vec<Article>, String> {
    let registry_path = path_resolver::resolve_workspace_path("articles_registry.json");
    let images_base_dir = path_resolver::resolve_workspace_path("images");
    let output_base_dir = path_resolver::resolve_workspace_path("output/AIResearch");

    let registry = ArticleRegistry::load(&registry_path).map_err(|err| {
        format!(
            "Failed to load articles registry at {}: {}",
            registry_path.display(),
            err
        )
    })?;

    let mut articles_with_keys: Vec<(DateTime<Utc>, bool, String, Article)> = Vec::new();
    let mut category_pools: HashMap<String, CategoryImages> = HashMap::new();
    
    // Coletar IDs de artigos já processados do registry para evitar duplicatas
    let mut processed_ids: HashSet<String> = HashSet::new();

    // PRIMEIRO: Processar artigos do registry (prioridade)
    for metadata in registry.articles.values() {
        if !matches!(
            metadata.status,
            crate::utils::article_registry::ArticleStatus::Published
        ) {
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

        // Verificar se o artigo é para airesearch:
        // 1. Se destinations contém "airesearch" (case-insensitive)
        // 2. OU se output_dir está em output/AIResearch (para artigos antigos sem destinations)
        let is_airesearch = metadata
            .destinations
            .as_ref()
            .map(|dests| dests.iter().any(|d| d.eq_ignore_ascii_case("airesearch")))
            .unwrap_or(false)
            || output_dir.to_string_lossy().contains("AIResearch");

        if !is_airesearch {
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
        let linkedin_raw = read_optional_file(output_dir.as_path(), "linkedin.txt");
        let x_raw = read_optional_file(output_dir.as_path(), "x.txt");

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

        // Ensure primary category is valid, fallback to "ai" if not
        let primary_category = if VALID_CATEGORIES.contains(&primary_category.as_str()) {
            primary_category
        } else {
            "ai".to_string()
        };

        let mut normalized_categories = Vec::new();
        // Add valid categories from the list (max 3, as per prompt requirements)
        for cat in categories_list.iter().take(3) {
            if VALID_CATEGORIES.contains(&cat.as_str()) && !normalized_categories.contains(cat) {
                normalized_categories.push(cat.clone());
            }
        }

        // Ensure primary category is first if not already present
        if !normalized_categories.contains(&primary_category) {
            normalized_categories.insert(0, primary_category.clone());
        }

        // Limit to 3 categories total (as per prompt requirements)
        normalized_categories.truncate(3);

        // Fallback to primary category if empty
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

        let linkedin_post = linkedin_raw.trim();
        let x_post = x_raw.trim();
        
        // Get source URL from registry metadata (this is the original article URL used by the writer)
        // For news articles, arxiv_url contains the original article URL, not an arXiv URL
        // For arXiv papers, arxiv_url contains the arXiv abstract URL
        // pdf_url contains the PDF URL (arXiv PDF for papers, or same as arxiv_url for news)
        // Prefer pdf_url if it's different from arxiv_url (actual PDF), otherwise use arxiv_url (original source)
        let source_url = if !metadata.pdf_url.is_empty() && metadata.pdf_url != metadata.arxiv_url {
            Some(metadata.pdf_url.clone())
        } else if !metadata.arxiv_url.is_empty() {
            Some(metadata.arxiv_url.clone())
        } else {
            None
        };
        
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
            linkedin_post: if linkedin_post.is_empty() { None } else { Some(linkedin_post.to_string()) },
            x_post: if x_post.is_empty() { None } else { Some(x_post.to_string()) },
            source_url,
        };

        articles_with_keys.push((published_at_dt, featured, metadata.id.clone(), article));
        processed_ids.insert(metadata.id.clone());
    }

    // SEGUNDO: Processar pastas em output/AIResearch que NÃO estão no registry
    // Isso garante que artigos antigos ou não registrados também apareçam
    if output_base_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&output_base_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let folder_name = entry.file_name();
                        let folder_name_str = folder_name.to_string_lossy();
                        let output_dir = output_base_dir.join(&folder_name);
                        
                        // Extrair ID do nome da pasta (formato: YYYY-MM-DD_source_ID ou apenas ID)
                        let article_id = folder_name_str
                            .split('_')
                            .last()
                            .unwrap_or(&folder_name_str)
                            .to_string();
                        
                        // Pular se já foi processado do registry
                        if processed_ids.contains(&article_id) {
                            continue;
                        }
                        
                        // Verificar se está marcado como hidden (arquivo .hidden na pasta)
                        let hidden_file = output_dir.join(".hidden");
                        let is_hidden = hidden_file.exists();
                        
                        if is_hidden {
                            continue; // Pular artigos hidden mesmo sem registry
                        }
                        
                        // Verificar se tem os arquivos necessários
                        let article_content = match read_first_existing_file(&output_dir, &ARTICLE_FILES) {
                            Some(content) => content.trim().to_string(),
                            None => continue,
                        };
                        
                        if article_content.is_empty() {
                            continue;
                        }
                        
                        // Tentar ler metadados dos arquivos
                        let title_raw = read_optional_file(&output_dir, "title.txt");
                        let subtitle_raw = read_optional_file(&output_dir, "subtitle.txt");
                        let categories_raw = read_optional_file(&output_dir, "image_categories.txt");
                        let slug_raw = read_optional_file(&output_dir, "slug.txt");
                        let source_raw = read_optional_file(&output_dir, "source.txt");
                        let linkedin_raw = read_optional_file(&output_dir, "linkedin.txt");
                        let x_raw = read_optional_file(&output_dir, "x.txt");
                        
                        let title = if !title_raw.trim().is_empty() {
                            title_raw.trim().to_string()
                        } else {
                            // Tentar extrair do nome da pasta como fallback
                            folder_name_str.to_string()
                        };
                        
                        let fallback_slug = generate_slug(&title);
                        let slug = if !slug_raw.trim().is_empty() {
                            slug_raw.trim().to_string()
                        } else {
                            fallback_slug.clone()
                        };
                        
                        let categories_list = sanitize_categories(&categories_raw);
                        let source_trimmed = source_raw.trim();
                        
                        let primary_category = map_source_to_category(
                            source_trimmed,
                            &categories_list,
                            None,
                        );
                        
                        let primary_category = if VALID_CATEGORIES.contains(&primary_category.as_str()) {
                            primary_category
                        } else {
                            "ai".to_string()
                        };
                        
                        let mut normalized_categories = Vec::new();
                        for cat in categories_list.iter().take(3) {
                            if VALID_CATEGORIES.contains(&cat.as_str()) && !normalized_categories.contains(cat) {
                                normalized_categories.push(cat.clone());
                            }
                        }
                        
                        if !normalized_categories.contains(&primary_category) {
                            normalized_categories.insert(0, primary_category.clone());
                        }
                        
                        normalized_categories.truncate(3);
                        
                        if normalized_categories.is_empty() {
                            normalized_categories.push(primary_category.clone());
                        }
                        
                        let image_path = select_article_image(
                            &normalized_categories,
                            images_base_dir.as_path(),
                            &article_id,
                            &mut category_pools,
                        );
                        
                        let author = if source_trimmed.is_empty() {
                            "AI Research".to_string()
                        } else {
                            source_trimmed.to_string()
                        };
                        
                        let subtitle_trimmed = subtitle_raw.trim().to_string();
                        
                        // Usar data de modificação da pasta como published_at
                        let published_at_dt = output_dir
                            .metadata()
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| {
                                t.duration_since(std::time::UNIX_EPOCH)
                                    .ok()
                                    .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
                            })
                            .unwrap_or_else(|| Utc::now());
                        
                        let published_at = published_at_dt.to_rfc3339();
                        let linkedin_post = linkedin_raw.trim();
                        let x_post = x_raw.trim();
                        
                        // Tentar obter source_url do registry se existir, senão usar arxiv_url/pdf_url
                        let source_url = None; // Para artigos não no registry, não temos URL ainda
                        
                        let article = Article {
                            id: article_id.clone(),
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
                            featured: false,
                            hidden: false,
                            linkedin_post: if linkedin_post.is_empty() { None } else { Some(linkedin_post.to_string()) },
                            x_post: if x_post.is_empty() { None } else { Some(x_post.to_string()) },
                            source_url,
                        };
                        
                        articles_with_keys.push((published_at_dt, false, article_id, article));
                    }
                }
            }
        }
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
    
    // Paginação: limit padrão de 50 artigos para melhor performance
    let limit = query.limit.unwrap_or(50).min(200); // Máximo 200 artigos por request
    let offset = query.offset.unwrap_or(0);
    let total = filtered.len();
    
    // Aplicar paginação
    let paginated: Vec<Article> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    eprintln!(
        "[AIResearch API] Returning {}/{} articles (offset: {}, limit: {}), {} featured",
        paginated.len(), total, offset, limit, featured_count
    );

    // Adicionar headers de cache para melhor performance
    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=300, s-maxage=300, stale-while-revalidate=600"),
    );
    headers.insert(
        "Vary",
        HeaderValue::from_static("Accept-Encoding"),
    );
    
    Ok((headers, Json(json!({ 
        "articles": paginated,
        "pagination": {
            "total": total,
            "limit": limit,
            "offset": offset,
            "hasMore": offset + paginated.len() < total
        }
    }))))
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

/// Diagnóstico: Compara artigos na pasta output/AIResearch com o registry
/// Retorna artigos que estão na pasta mas não estão sendo incluídos na API
pub async fn diagnose_articles() -> Result<impl IntoResponse, (StatusCode, String)> {
    let registry_path = path_resolver::resolve_workspace_path("articles_registry.json");
    let output_dir = path_resolver::resolve_workspace_path("output/AIResearch");

    let registry = ArticleRegistry::load(&registry_path).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load registry: {}", err),
        )
    })?;

    // 1. Listar todos os diretórios em output/AIResearch
    let mut folders_in_output: Vec<String> = Vec::new();
    if output_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&output_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(folder_name) = entry.file_name().to_str() {
                            folders_in_output.push(folder_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Analisar artigos no registry que DEVERIAM aparecer
    let mut in_registry_published = 0;
    let mut in_registry_with_output_dir = 0;
    let mut in_registry_visible = 0;
    let mut in_registry_in_airesearch = 0;
    let mut missing_output_dir = Vec::new();
    let mut hidden_articles = Vec::new();
    let mut wrong_status = Vec::new();
    let mut output_dir_not_exists = Vec::new();
    let mut missing_article_file = Vec::new();
    let mut empty_content = Vec::new();
    let mut not_in_airesearch = Vec::new();

    for (id, metadata) in registry.articles.iter() {
        // Contar apenas Published
        if matches!(metadata.status, crate::utils::article_registry::ArticleStatus::Published) {
            in_registry_published += 1;

            // Verificar output_dir
            if let Some(output_path) = &metadata.output_dir {
                in_registry_with_output_dir += 1;
                let resolved_output = path_resolver::resolve_workspace_path(output_path);
                
                if resolved_output.is_dir() {
                    // Verificar se é para airesearch
                    let is_airesearch = metadata
                        .destinations
                        .as_ref()
                        .map(|dests| dests.iter().any(|d| d.eq_ignore_ascii_case("airesearch")))
                        .unwrap_or(false)
                        || resolved_output.to_string_lossy().contains("AIResearch");

                    if is_airesearch {
                        in_registry_in_airesearch += 1;

                        // Verificar se não está hidden
                        if !metadata.hidden.unwrap_or(false) {
                            in_registry_visible += 1;

                            // Verificar se tem article.md
                            if let Some(article_content) = read_first_existing_file(&resolved_output, &ARTICLE_FILES) {
                                if article_content.trim().is_empty() {
                                    empty_content.push(id.clone());
                                }
                            } else {
                                missing_article_file.push(id.clone());
                            }
                        } else {
                            hidden_articles.push(id.clone());
                        }
                    } else {
                        not_in_airesearch.push(id.clone());
                    }
                } else {
                    output_dir_not_exists.push((id.clone(), output_path.to_string_lossy().to_string()));
                }
            } else {
                missing_output_dir.push(id.clone());
            }
        } else {
            wrong_status.push((id.clone(), format!("{:?}", metadata.status)));
        }
    }

    // 3. Encontrar pastas em output que não estão no registry
    let folders_in_registry: HashSet<String> = registry.articles.values()
        .filter_map(|m| m.output_dir.as_ref())
        .filter_map(|p| {
            let resolved = path_resolver::resolve_workspace_path(p);
            resolved.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
        })
        .collect();

    let folders_not_in_registry: Vec<String> = folders_in_output
        .iter()
        .filter(|folder| !folders_in_registry.contains(*folder))
        .cloned()
        .collect();

    Ok(Json(json!({
        "summary": {
            "folders_in_output_dir": folders_in_output.len(),
            "articles_in_registry": registry.articles.len(),
            "articles_published": in_registry_published,
            "articles_with_output_dir": in_registry_with_output_dir,
            "articles_in_airesearch": in_registry_in_airesearch,
            "articles_visible": in_registry_visible,
        },
        "issues": {
            "folders_not_in_registry": folders_not_in_registry,
            "folders_not_in_registry_count": folders_not_in_registry.len(),
            "missing_output_dir": missing_output_dir,
            "hidden_articles": hidden_articles,
            "wrong_status": wrong_status,
            "output_dir_not_exists": output_dir_not_exists,
            "missing_article_file": missing_article_file,
            "empty_content": empty_content,
            "not_in_airesearch": not_in_airesearch,
        },
        "counts": {
            "folders_in_output": folders_in_output.len(),
            "folders_not_in_registry": folders_not_in_registry.len(),
            "missing_output_dir_count": missing_output_dir.len(),
            "hidden_count": hidden_articles.len(),
            "wrong_status_count": wrong_status.len(),
            "output_dir_not_exists_count": output_dir_not_exists.len(),
            "missing_article_file_count": missing_article_file.len(),
            "empty_content_count": empty_content.len(),
            "not_in_airesearch_count": not_in_airesearch.len(),
        }
    })))
}
