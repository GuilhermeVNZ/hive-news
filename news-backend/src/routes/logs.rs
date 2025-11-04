use axum::{extract::{Extension, Path, Query}, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{Utc};
use std::path::{Path as FsPath, PathBuf};
use std::fs;

use crate::db::connection::Database;
use crate::utils::article_registry::{RegistryManager, ArticleStatus};
use crate::utils::article_registry::ArticleRegistry;
use std::collections::HashMap;

// Helper function to get registry path (same logic as main.rs)
fn get_registry_path() -> std::path::PathBuf {
    use std::path::PathBuf;
    use std::env;
    
    // Try multiple possible locations
    let possible_paths = vec![
        PathBuf::from("articles_registry.json"),
        PathBuf::from("../articles_registry.json"),
        PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json"),
        env::current_dir()
            .ok()
            .and_then(|d| d.parent().map(|p| p.join("articles_registry.json")))
            .unwrap_or_else(|| PathBuf::from("articles_registry.json")),
    ];
    
    // Try to find existing file
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    // Default to absolute path
    PathBuf::from("G:/Hive-Hub/News-main/articles_registry.json")
}

#[derive(Debug, Serialize)]
struct ArticleLogItem {
    id: String,
    title: String,
    created_at: String,
    age_seconds: i64,
    source: String,
    destinations: Vec<DestinationInfo>,
    hidden: bool,
    featured: bool,
}

#[derive(Debug, Serialize)]
struct DestinationInfo { site_id: String, site_name: String, url: String }

#[derive(Debug, Deserialize)]
pub struct LogsQuery { 
    pub q: Option<String>, 
    pub limit: Option<usize>, 
    pub offset: Option<usize>,
    pub featured: Option<bool>, // Filter by featured status
}

pub async fn list_logs(Extension(_db): Extension<std::sync::Arc<Database>>,
    query: Option<Query<LogsQuery>>
) -> Json<Value> {
    let params = query.map(|q| q.0).unwrap_or(LogsQuery{ 
        q: None, 
        limit: None, 
        offset: None,
        featured: None,
    });

    // Carrega registry
    let registry_path = get_registry_path();
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => return Json(serde_json::json!({"success": false, "error": format!("{}", e)})),
    };

    let mut all = manager.get_all_articles()
        .into_iter()
        .filter(|m| m.status == ArticleStatus::Published)
        .collect::<Vec<_>>();

    // Filter by featured status if requested
    if let Some(featured_only) = params.featured {
        if featured_only {
            let before_count = all.len();
            all.retain(|m| {
                // Handle both boolean and string representations of true
                let is_featured = match m.featured {
                    Some(true) => true,
                    Some(false) => false,
                    None => false,
                };
                eprintln!("[Logs API] Article {} featured check: featured={:?}, result={}", 
                    m.id, m.featured, is_featured);
                is_featured
            });
            let after_count = all.len();
            eprintln!("[Logs API] Featured filter: before={}, after={}, filtered={}", 
                before_count, after_count, before_count - after_count);
        }
    }

    if let Some(q) = params.q.as_ref() {
        // Normalize robustly: lowercase, collapse whitespace, scrub punctuation to spaces,
        // keep only ASCII letters/digits/spaces to avoid hidden unicode differences.
        fn normalize(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            let mut last_space = false;
            for ch in s.chars().flat_map(|c| c.to_lowercase()) {
                let mapped = if ch.is_ascii_alphanumeric() { Some(ch) }
                    else if ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '\'' | '"' | '’' | '“' | '”' | '—' | '–' | '/' | '\\' | ':' | ';' | ',' | '.' | '(' | ')' | '[' | ']' | '{' | '}' | '|') { Some(' ') }
                    else { None };
                if let Some(m) = mapped {
                    if m == ' ' {
                        if !last_space { out.push(' '); last_space = true; }
                    } else {
                        out.push(m);
                        last_space = false;
                    }
                }
            }
            out.trim().to_string()
        }
        let qn = normalize(q);
        all.retain(|m| {
            let idn = normalize(&m.id);
            let titlen = normalize(&m.title);
            idn.contains(&qn) || titlen.contains(&qn)
        });
    }

    // Helper function to generate slug from title (same logic as AIResearch ArticleCard.tsx)
    // TypeScript: title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-')
    // \w in JavaScript includes: letters, digits, and underscore
    // So we need to keep: alphanumeric, underscore, spaces, and hyphens
    // Then replace one or more spaces with a single hyphen
    fn title_to_slug(title: &str) -> String {
        // Step 1: toLowerCase()
        let mut slug = title.to_lowercase();
        
        // Step 2: .replace(/[^\w\s-]/g, '') - remove all chars that are NOT:
        //   \w = alphanumeric or underscore
        //   \s = whitespace
        //   - = hyphen
        slug = slug
            .chars()
            .filter(|&c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '-')
            .collect::<String>();
        
        // Step 3: .replace(/\s+/g, '-') - replace one or more spaces with single hyphen
        // Use regex-like behavior: split by whitespace, join with hyphens
        // This handles multiple spaces correctly
        slug = slug
            .split_whitespace()
            .filter(|s| !s.is_empty()) // Remove empty strings from split
            .collect::<Vec<&str>>()
            .join("-");
        
        slug
    }
    
    let mut items: Vec<(i64, ArticleLogItem)> = all.into_iter().filter_map(|m| {
        let now = Utc::now();
        let ts = m.published_at.unwrap_or_else(|| m.collected_at.unwrap_or(now));
        let age = (now - ts).num_seconds();
        // Helper function to extract arXiv ID from article ID or folder name
        // Handles formats: "2510.26038" or "2025-10-29_unknown_2510.26038" -> "2510.26038"
        fn extract_arxiv_id(id: &str) -> Option<String> {
            // Try to find arXiv ID pattern (YYYY.NNNNN or YYYY.NNNNNN)
            if let Some(captures) = regex::Regex::new(r"(\d{4}\.\d{4,6})").ok() {
                if let Some(mat) = captures.find(id) {
                    return Some(mat.as_str().to_string());
                }
            }
            // Fallback: if the ID itself looks like an arXiv ID
            if id.matches('.').count() == 1 && id.len() >= 9 && id.len() <= 12 {
                if let Some(_) = id.find('.') {
                    return Some(id.to_string());
                }
            }
            None
        }
        
        // Use generated_title (from title.txt) if available, otherwise try to read from filesystem
        // This ensures titles in logs match what users see in the frontend
        let mut actual_title = m.generated_title.clone()
            .filter(|t| !t.is_empty())
            .or_else(|| m.original_title.clone())
            .filter(|t| !t.is_empty());
        
        // If generated_title is not available in registry, try to read from title.txt in filesystem
        if actual_title.is_none() {
            if let Some(output_dir) = &m.output_dir {
                let title_txt_path = output_dir.join("title.txt");
                if let Ok(title_content) = fs::read_to_string(&title_txt_path) {
                    let title = title_content.trim().to_string();
                    if !title.is_empty() {
                        actual_title = Some(title);
                    }
                }
            }
        }
        
        // Fallback to title field if nothing else is available
        let actual_title = actual_title.unwrap_or_else(|| m.title.clone());
        
        let mut destinations: Vec<DestinationInfo> = Vec::new();
        
        // Use destinations field from metadata if available (more reliable)
        if let Some(dest_sites) = &m.destinations {
            if !dest_sites.is_empty() {
                for site_id in dest_sites {
                    let site_id_lower = site_id.to_lowercase();
                    // Determine correct localhost port based on site
                    let base_url = if site_id_lower == "scienceai" {
                        "http://localhost:8080"
                    } else if site_id_lower == "airesearch" {
                        "http://localhost:3003"
                    } else {
                        eprintln!("[Logs API] Unknown site_id: {}, skipping...", site_id_lower);
                        continue; // Skip unknown sites
                    };
                    
                    // Generate URL using actual title from filesystem (matches what AIResearch shows)
                    let slug = title_to_slug(&actual_title);
                    let url = format!("{}/article/{}", base_url, slug);
                    
                    eprintln!("[Logs API] Article {}: site_id={}, base_url={}, slug={}, url={}", 
                        m.id, site_id_lower, base_url, slug, url);
                    
                    // Get site name from site_id
                    let site_name = if site_id_lower == "scienceai" {
                        "ScienceAI"
                    } else if site_id_lower == "airesearch" {
                        "AIResearch"
                    } else {
                        site_id
                    };
                    
                    destinations.push(DestinationInfo{ 
                        site_id: site_id_lower.clone(), 
                        site_name: site_name.to_string(), 
                        url 
                    });
                }
            }
        }
        
        // Fallback: if no destinations were created, try output_dir
        if destinations.is_empty() {
            if let Some(dir) = &m.output_dir {
                // Fallback: try to extract from output_dir path
                // Handle Windows path with mixed separators (G:/Hive-Hub/News-main/output\AIResearch\2510.27413)
                let dir_str = dir.to_string_lossy().to_string();
                let normalized_dir = dir_str.replace('\\', "/");
                let dir_path = PathBuf::from(&normalized_dir);
                
                if let Some(site_os) = dir_path.parent().and_then(|p| p.file_name()) {
                    let site_name = site_os.to_string_lossy().to_string();
                    let site_id_lower = site_name.to_lowercase();
                    
                    eprintln!("[Logs API] Fallback: Article {}: output_dir={}, extracted site_name={}, site_id_lower={}", 
                        m.id, dir_str, site_name, site_id_lower);
                    
                    // Determine correct localhost port based on site
                    let base_url = if site_id_lower == "scienceai" {
                        "http://localhost:8080"
                    } else if site_id_lower == "airesearch" {
                        "http://localhost:3003"
                    } else {
                        eprintln!("[Logs API] Unknown site from output_dir: {}, defaulting to dashboard", site_id_lower);
                        "http://localhost:1420" // Default to dashboard
                    };
                    
                    // Generate URL using actual title from filesystem (matches what AIResearch shows)
                    let slug = title_to_slug(&actual_title);
                    let url = format!("{}/article/{}", base_url, slug);
                    
                    eprintln!("[Logs API] Fallback URL: base_url={}, slug={}, url={}", base_url, slug, url);
                    
                    destinations.push(DestinationInfo{ 
                        site_id: site_id_lower, 
                        site_name, 
                        url 
                    });
                } else {
                    eprintln!("[Logs API] Failed to extract site from output_dir: {}", dir_str);
                }
            }
        }
        
        // Filter destinations: only include if output directory exists
        // This prevents showing links to articles that don't exist in filesystem
        // Note: We need to match by arXiv ID because folder names changed from "ID" to "date_source_ID"
        let arxiv_id = extract_arxiv_id(&m.id);
        let mut valid_destinations: Vec<DestinationInfo> = Vec::new();
        for dest in destinations {
            // Determine the site output directory
            let site_output_dir = if dest.site_id == "airesearch" {
                FsPath::new("G:/Hive-Hub/News-main/output/AIResearch")
            } else if dest.site_id == "scienceai" {
                FsPath::new("G:/Hive-Hub/News-main/output/ScienceAI")
            } else {
                eprintln!("[Logs API] Unknown site_id: {}, skipping...", dest.site_id);
                continue;
            };
            
            // Check if article exists in filesystem
            let output_dir_exists = if let Some(ref arxiv) = arxiv_id {
                // Search for folder containing this arXiv ID
                // Format can be: "2510.26038" or "2025-10-29_unknown_2510.26038"
                let mut found = false;
                if let Ok(entries) = fs::read_dir(site_output_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let folder_name = entry.file_name().to_string_lossy().to_string();
                            // Check if folder name contains the arXiv ID
                            if folder_name.contains(arxiv) {
                                let folder_path = entry.path();
                                if folder_path.is_dir() {
                                    // Verify that required files exist
                                    let title_txt = folder_path.join("title.txt");
                                    let article_md = folder_path.join("article.md");
                                    if title_txt.exists() && article_md.exists() {
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                found
            } else {
                // For non-arXiv articles, try to match by exact folder name or partial match
                if let Some(ref output_dir) = m.output_dir {
                    let dir_path = FsPath::new(output_dir);
                    if dir_path.exists() && dir_path.is_dir() {
                        let title_txt = dir_path.join("title.txt");
                        let article_md = dir_path.join("article.md");
                        title_txt.exists() && article_md.exists()
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            
            if output_dir_exists {
                valid_destinations.push(dest);
            } else {
                eprintln!("[Logs API] Skipping destination {} for article {} (arXiv ID: {:?}) - not found in filesystem", 
                    dest.site_name, m.id, arxiv_id);
            }
        }
        
        // Debug: Log final destinations
        if valid_destinations.is_empty() {
            eprintln!("[Logs API] WARNING: Article {} has no valid destinations! title={}, output_dir={:?}, destinations={:?}", 
                m.id, m.title, m.output_dir, m.destinations);
        } else {
            eprintln!("[Logs API] Article {} final destinations count: {}", m.id, valid_destinations.len());
            for dest in &valid_destinations {
                eprintln!("[Logs API]   - site_id: {}, site_name: {}, url: {}", dest.site_id, dest.site_name, dest.url);
            }
        }
        
        // Only include articles that have at least one valid destination
        // This filters out articles that exist in registry but not in filesystem
        if valid_destinations.is_empty() {
            return None; // Skip this article - it doesn't exist in filesystem
        }
        
        let item = ArticleLogItem{
            id: m.id,
            title: actual_title, // Use title from filesystem (title.txt) to match AIResearch
            created_at: ts.to_rfc3339(),
            age_seconds: age,
            source: if m.arxiv_url.contains("arxiv") {"arXiv"} else {"Source"}.to_string(),
            destinations: valid_destinations,
            hidden: m.hidden.unwrap_or(false),
            featured: m.featured.unwrap_or(false),
        };
        Some((ts.timestamp(), item))
    }).collect::<Vec<_>>();

    // Sort by timestamp descending (newest first)
    items.sort_by(|a,b| b.0.cmp(&a.0));
    
    // Apply limit and offset
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    let items: Vec<ArticleLogItem> = items.into_iter().skip(offset).take(limit).map(|(_,it)| it).collect();

    Json(serde_json::json!({"success": true, "items": items}))
}

// No query params for now; soft-hide only

pub async fn hide_article(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let registry_path = get_registry_path();
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => return Json(serde_json::json!({"success": false, "error": format!("{}", e)})),
    };
    // mark hidden
    let mut all = manager.get_all_articles();
    if let Some(meta) = all.iter_mut().find(|m| m.id == id) {
        meta.hidden = Some(true);
        // write back
        // Quick save path: rebuild and save
        use crate::utils::article_registry::ArticleRegistry;
        use std::collections::HashMap;
        let mut map: HashMap<String, _> = HashMap::new();
        for m in all.into_iter() { map.insert(m.id.clone(), m); }
        let reg = ArticleRegistry{ articles: map };
        if let Err(e) = reg.save(&registry_path) {
            return Json(serde_json::json!({"success": false, "error": format!("{}", e)}))
        }
    }
    Json(serde_json::json!({"success": true}))
}

#[derive(Debug, Deserialize)]
pub struct HiddenUpdate { pub hidden: bool }

pub async fn set_hidden(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(body): Json<HiddenUpdate>,
) -> Json<Value> {
    // Validar ID
    if id.is_empty() {
        return Json(serde_json::json!({"success": false, "error": "Article ID is required"}));
    }

    let registry_path = get_registry_path();
    
    // Criar manager thread-safe (usa Mutex internamente)
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load registry: {}", e);
            return Json(serde_json::json!({"success": false, "error": format!("Failed to load registry: {}", e)}));
        },
    };
    
    // Helper function to extract arXiv ID (same as in list_logs)
    fn extract_arxiv_id(id: &str) -> Option<String> {
        if let Some(captures) = regex::Regex::new(r"(\d{4}\.\d{4,6})").ok() {
            if let Some(mat) = captures.find(id) {
                return Some(mat.as_str().to_string());
            }
        }
        if id.matches('.').count() == 1 && id.len() >= 9 && id.len() <= 12 {
            if let Some(_) = id.find('.') {
                return Some(id.to_string());
            }
        }
        None
    }
    
    // Try to find article by matching title from filesystem
    // This ensures we update the correct article even if titles differ
    let arxiv_id = extract_arxiv_id(&id);
    let mut found_id: Option<String> = None;
    
    // Search for article in filesystem to get actual title, then find matching registry entry
    let site_dirs = vec![
        FsPath::new("G:/Hive-Hub/News-main/output/AIResearch"),
        FsPath::new("G:/Hive-Hub/News-main/output/ScienceAI"),
    ];
    
    let mut actual_title_from_fs: Option<String> = None;
    
    if let Some(ref arxiv) = arxiv_id {
        for site_output_dir in site_dirs {
            if let Ok(entries) = fs::read_dir(site_output_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        if folder_name.contains(arxiv) {
                            let folder_path = entry.path();
                            if folder_path.is_dir() {
                                let title_txt = folder_path.join("title.txt");
                                if title_txt.exists() {
                                    if let Ok(title_content) = fs::read_to_string(&title_txt) {
                                        actual_title_from_fs = Some(title_content.trim().to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if actual_title_from_fs.is_some() {
                break;
            }
        }
    }
    
    // If we found the title from filesystem, try to find matching article in registry
    // by comparing with the actual title from filesystem
    if let Some(ref _fs_title) = actual_title_from_fs {
        if let Some(ref arxiv) = arxiv_id {
            let all_articles = manager.get_all_articles();
            for article in all_articles {
                // Extract arXiv ID from registry article ID
                let reg_arxiv_id = extract_arxiv_id(&article.id);
                if reg_arxiv_id == arxiv_id {
                    // Found matching arXiv ID, verify by checking title in filesystem
                    // We already have the title from filesystem, so use the registry ID
                    found_id = Some(article.id.clone());
                    eprintln!("[set_hidden] Found article by arXiv ID: {} -> registry ID: {}", arxiv, article.id);
                    break;
                }
            }
        }
    }
    
    // Use found_id if available, otherwise fall back to provided id
    let article_id_to_update = found_id.unwrap_or(id);
    
    eprintln!("[set_hidden] Updating hidden status: article_id={}, hidden={}, fs_title={:?}", 
        article_id_to_update, body.hidden, actual_title_from_fs);
    
    // Usar método thread-safe do RegistryManager
    match manager.set_hidden(&article_id_to_update, body.hidden) {
        Ok(_) => {
            tracing::info!("Successfully updated hidden status for article: {} (fs_title: {:?})", 
                article_id_to_update, actual_title_from_fs);
            Json(serde_json::json!({"success": true}))
        },
        Err(e) => {
            tracing::error!("Failed to update hidden status for article {}: {}", article_id_to_update, e);
            Json(serde_json::json!({"success": false, "error": format!("{}", e)}))
        }
    }
}

#[derive(Debug, Serialize)]
struct EnrichResult { updated: usize, skipped: usize, errors: usize }

// Enrich titles for articles whose title is "Untitled" by querying arXiv metadata
pub async fn enrich_titles_from_arxiv(
    Extension(_db): Extension<std::sync::Arc<Database>>,
) -> Json<Value> {
    let registry_path = get_registry_path();
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => return Json(serde_json::json!({"success": false, "error": format!("{}", e)})),
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build() {
            Ok(c) => c,
            Err(e) => return Json(serde_json::json!({"success": false, "error": format!("reqwest: {}", e)})),
        };

    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    let mut all = manager.get_all_articles();
    for meta in all.iter_mut() {
        if meta.title != "Untitled" { skipped += 1; continue; }
        if !meta.arxiv_url.contains("arxiv.org/abs/") { skipped += 1; continue; }

        // Extract arXiv ID
        let id = meta
            .arxiv_url
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split('v')
            .next()
            .unwrap_or("")
            .to_string();
        if id.is_empty() { skipped += 1; continue; }

        match fetch_arxiv_title(&client, &id).await {
            Ok(Some(title)) => {
                let clean = title.trim();
                if !clean.is_empty() && clean != "Untitled" {
                    meta.title = clean.to_string();
                    updated += 1;
                } else {
                    skipped += 1;
                }
            }
            Ok(None) => { skipped += 1; }
            Err(_) => { errors += 1; }
        }
        // Be gentle with API
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // Save back
    let mut map: HashMap<String, _> = HashMap::new();
    for m in all.into_iter() { map.insert(m.id.clone(), m); }
    let reg = ArticleRegistry{ articles: map };
    if let Err(e) = reg.save(&registry_path) {
        return Json(serde_json::json!({"success": false, "error": format!("save: {}", e)}));
    }

    Json(serde_json::json!({"success": true, "result": EnrichResult{ updated, skipped, errors }}))
}

async fn fetch_arxiv_title(client: &reqwest::Client, id: &str) -> Result<Option<String>, anyhow::Error> {
    let url = format!("https://export.arxiv.org/api/query?id_list={}", id);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() { return Ok(None); }
    let xml = resp.text().await?;

    // Very small XML extraction: find first <entry> ... </entry>, then its <title> ... </title>
    if let (Some(entry_start), Some(entry_end)) = (xml.find("<entry>"), xml.find("</entry>")) {
        let entry = &xml[entry_start..=entry_end];
        if let (Some(t1), Some(t2)) = (entry.find("<title>"), entry.find("</title>")) {
            let title = &entry[t1 + 7..t2];
            // arXiv titles may contain newlines/extra spaces
            let norm = title.replace('\n', " ").replace("  ", " ").trim().to_string();
            return Ok(Some(norm));
        }
    }
    Ok(None)
}

#[derive(Debug, Deserialize)]
pub struct FeaturedUpdate { pub featured: bool }

pub async fn set_featured(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(body): Json<FeaturedUpdate>,
) -> Json<Value> {
    // Validar ID
    if id.is_empty() {
        return Json(serde_json::json!({"success": false, "error": "Article ID is required"}));
    }

    let registry_path = get_registry_path();
    
    // Criar manager thread-safe (usa Mutex internamente)
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load registry: {}", e);
            return Json(serde_json::json!({"success": false, "error": format!("Failed to load registry: {}", e)}));
        },
    };
    
    // Helper function to extract arXiv ID (same as in list_logs)
    fn extract_arxiv_id(id: &str) -> Option<String> {
        if let Some(captures) = regex::Regex::new(r"(\d{4}\.\d{4,6})").ok() {
            if let Some(mat) = captures.find(id) {
                return Some(mat.as_str().to_string());
            }
        }
        if id.matches('.').count() == 1 && id.len() >= 9 && id.len() <= 12 {
            if let Some(_) = id.find('.') {
                return Some(id.to_string());
            }
        }
        None
    }
    
    // Try to find article by matching title from filesystem
    // This ensures we update the correct article even if titles differ
    let arxiv_id = extract_arxiv_id(&id);
    let mut found_id: Option<String> = None;
    
    // Search for article in filesystem to get actual title, then find matching registry entry
    let site_dirs = vec![
        FsPath::new("G:/Hive-Hub/News-main/output/AIResearch"),
        FsPath::new("G:/Hive-Hub/News-main/output/ScienceAI"),
    ];
    
    let mut actual_title_from_fs: Option<String> = None;
    
    if let Some(ref arxiv) = arxiv_id {
        for site_output_dir in site_dirs {
            if let Ok(entries) = fs::read_dir(site_output_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        if folder_name.contains(arxiv) {
                            let folder_path = entry.path();
                            if folder_path.is_dir() {
                                let title_txt = folder_path.join("title.txt");
                                if title_txt.exists() {
                                    if let Ok(title_content) = fs::read_to_string(&title_txt) {
                                        actual_title_from_fs = Some(title_content.trim().to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if actual_title_from_fs.is_some() {
                break;
            }
        }
    }
    
    // If we found the title from filesystem, try to find matching article in registry
    // by comparing with the actual title from filesystem
    if let Some(ref _fs_title) = actual_title_from_fs {
        if let Some(ref arxiv) = arxiv_id {
            let all_articles = manager.get_all_articles();
            for article in all_articles {
                // Extract arXiv ID from registry article ID
                let reg_arxiv_id = extract_arxiv_id(&article.id);
                if reg_arxiv_id == arxiv_id {
                    // Found matching arXiv ID, verify by checking title in filesystem
                    // We already have the title from filesystem, so use the registry ID
                    found_id = Some(article.id.clone());
                    eprintln!("[set_featured] Found article by arXiv ID: {} -> registry ID: {}", arxiv, article.id);
                    break;
                }
            }
        }
    }
    
    // Use found_id if available, otherwise fall back to provided id
    let article_id_to_update = found_id.unwrap_or(id);
    
    eprintln!("[set_featured] Updating featured status: article_id={}, featured={}, fs_title={:?}", 
        article_id_to_update, body.featured, actual_title_from_fs);
    
    // Usar método thread-safe do RegistryManager
    match manager.set_featured(&article_id_to_update, body.featured) {
        Ok(_) => {
            tracing::info!("Successfully updated featured status for article: {} (fs_title: {:?})", 
                article_id_to_update, actual_title_from_fs);
            Json(serde_json::json!({"success": true}))
        },
        Err(e) => {
            tracing::error!("Failed to update featured status for article {}: {}", article_id_to_update, e);
            Json(serde_json::json!({"success": false, "error": format!("{}", e)}))
        }
    }
}
