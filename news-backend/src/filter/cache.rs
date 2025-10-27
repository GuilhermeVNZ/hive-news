use std::fs;
use std::path::PathBuf;

pub fn cache_dir() -> PathBuf {
    PathBuf::from("G:/Hive-Hub/News-main/downloads/cache")
}

pub fn save_doi_cache(doi: &str, valid: bool) {
    if let Ok(dir) = fs::create_dir_all(cache_dir()) {
        let safe_doi = doi.replace("/", "_").replace(":", "_");
        let cache_file = cache_dir().join(format!("doi_{}.json", safe_doi));
        let data = serde_json::json!({ "doi": doi, "valid": valid });
        let _ = fs::write(cache_file, data.to_string());
    }
}

pub fn load_doi_cache(doi: &str) -> Option<bool> {
    let safe_doi = doi.replace("/", "_").replace(":", "_");
    let cache_file = cache_dir().join(format!("doi_{}.json", safe_doi));

    if let Ok(content) = fs::read_to_string(cache_file) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            return data.get("valid")?.as_bool();
        }
    }
    None
}

pub fn save_author_cache(author: &str, valid: bool) {
    if let Ok(dir) = fs::create_dir_all(cache_dir()) {
        let safe_author = author.replace(" ", "_");
        let cache_file = cache_dir().join(format!("author_{}.json", safe_author));
        let data = serde_json::json!({ "author": author, "valid": valid });
        let _ = fs::write(cache_file, data.to_string());
    }
}

pub fn load_author_cache(author: &str) -> Option<bool> {
    let safe_author = author.replace(" ", "_");
    let cache_file = cache_dir().join(format!("author_{}.json", safe_author));

    if let Ok(content) = fs::read_to_string(cache_file) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            return data.get("valid")?.as_bool();
        }
    }
    None
}
