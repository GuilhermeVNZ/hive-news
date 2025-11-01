use axum::{extract::{Extension, Path, Query}, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{Utc};
use std::path::Path as FsPath;

use crate::db::connection::Database;
use crate::utils::article_registry::{RegistryManager, ArticleStatus};
use crate::utils::article_registry::ArticleRegistry;
use std::collections::HashMap;

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
pub struct LogsQuery { pub q: Option<String>, pub limit: Option<usize>, pub offset: Option<usize> }

pub async fn list_logs(Extension(_db): Extension<std::sync::Arc<Database>>,
    query: Option<Query<LogsQuery>>
) -> Json<Value> {
    let params = query.map(|q| q.0).unwrap_or(LogsQuery{ q: None, limit: None, offset: None });

    // Carrega registry
    let registry_path = FsPath::new("../articles_registry.json");
    let manager = match RegistryManager::new(registry_path) {
        Ok(m) => m,
        Err(e) => return Json(serde_json::json!({"success": false, "error": format!("{}", e)})),
    };

    let mut all = manager.get_all_articles()
        .into_iter()
        .filter(|m| m.status == ArticleStatus::Published)
        .collect::<Vec<_>>();

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

    let mut items: Vec<(i64, ArticleLogItem)> = all.into_iter().map(|m| {
        let now = Utc::now();
        let ts = m.published_at.unwrap_or_else(|| m.collected_at.unwrap_or(now));
        let age = (now - ts).num_seconds();
        let mut destinations: Vec<DestinationInfo> = Vec::new();
        if let Some(dir) = m.output_dir.clone() {
            if let Some(site_os) = dir.parent().and_then(|p| p.file_name()) {
                let site_name = site_os.to_string_lossy().to_string();
                let url = format!("/articles/{}", m.id);
                destinations.push(DestinationInfo{ site_id: site_name.clone().to_lowercase(), site_name, url });
            }
        }
        let item = ArticleLogItem{
            id: m.id,
            title: m.title,
            created_at: ts.to_rfc3339(),
            age_seconds: age,
            source: if m.arxiv_url.contains("arxiv") {"arXiv"} else {"Source"}.to_string(),
            destinations,
            hidden: m.hidden.unwrap_or(false),
            featured: m.featured.unwrap_or(false),
        };
        (ts.timestamp(), item)
    }).collect();

    // Newest first
    items.sort_by(|a,b| b.0.cmp(&a.0));
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
    let registry_path = FsPath::new("../articles_registry.json");
    let manager = match RegistryManager::new(registry_path) {
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
        if let Err(e) = reg.save(registry_path) {
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

    let registry_path = FsPath::new("../articles_registry.json");
    
    // Criar manager thread-safe (usa Mutex internamente)
    let manager = match RegistryManager::new(registry_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load registry: {}", e);
            return Json(serde_json::json!({"success": false, "error": format!("Failed to load registry: {}", e)}));
        },
    };
    
    // Usar método thread-safe do RegistryManager
    match manager.set_hidden(&id, body.hidden) {
        Ok(_) => {
            tracing::info!("Successfully updated hidden status for article: {}", id);
            Json(serde_json::json!({"success": true}))
        },
        Err(e) => {
            tracing::error!("Failed to update hidden status: {}", e);
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
    let registry_path = FsPath::new("../articles_registry.json");
    let manager = match RegistryManager::new(registry_path) {
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
    if let Err(e) = reg.save(registry_path) {
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

    let registry_path = FsPath::new("../articles_registry.json");
    
    // Criar manager thread-safe (usa Mutex internamente)
    let manager = match RegistryManager::new(registry_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load registry: {}", e);
            return Json(serde_json::json!({"success": false, "error": format!("Failed to load registry: {}", e)}));
        },
    };
    
    // Usar método thread-safe do RegistryManager
    match manager.set_featured(&id, body.featured) {
        Ok(_) => {
            tracing::info!("Successfully updated featured status for article: {}", id);
            Json(serde_json::json!({"success": true}))
        },
        Err(e) => {
            tracing::error!("Failed to update featured status: {}", e);
            Json(serde_json::json!({"success": false, "error": format!("{}", e)}))
        }
    }
}
