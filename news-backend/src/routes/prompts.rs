use axum::{extract::Extension, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use anyhow::{Context, Result};

#[derive(Debug, Serialize)]
struct PromptFile {
    filename: String,
    size: u64,
    modified: String,
}

/// List all article prompts
pub async fn list_article_prompts(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    match get_prompt_files("article") {
        Ok(files) => Json(json!({
            "success": true,
            "prompts": files,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to list prompts: {}", e),
            "prompts": [],
        })),
    }
}

/// List all news prompts
pub async fn list_news_prompts(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
) -> Json<Value> {
    match get_prompt_files("news") {
        Ok(files) => Json(json!({
            "success": true,
            "prompts": files,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to list prompts: {}", e),
            "prompts": [],
        })),
    }
}

/// Get content of a specific article prompt
pub async fn get_article_prompt_content(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<Value> {
    match get_prompt_content("article", &filename) {
        Ok(content) => Json(json!({
            "success": true,
            "filename": filename,
            "content": content,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to read prompt: {}", e),
        })),
    }
}

/// Get content of a specific news prompt
pub async fn get_news_prompt_content(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<Value> {
    match get_prompt_content("news", &filename) {
        Ok(content) => Json(json!({
            "success": true,
            "filename": filename,
            "content": content,
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to read prompt: {}", e),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub filename: String,
    pub content: String,
}

/// Create a new article prompt
pub async fn create_article_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Json(request): axum::extract::Json<CreatePromptRequest>,
) -> Json<Value> {
    match create_prompt_file("article", &request.filename, &request.content) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' created successfully", request.filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to create prompt: {}", e),
        })),
    }
}

/// Create a new news prompt
pub async fn create_news_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Json(request): axum::extract::Json<CreatePromptRequest>,
) -> Json<Value> {
    match create_prompt_file("news", &request.filename, &request.content) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' created successfully", request.filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to create prompt: {}", e),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub content: String,
}

/// Update an existing article prompt
pub async fn update_article_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
    axum::extract::Json(request): axum::extract::Json<UpdatePromptRequest>,
) -> Json<Value> {
    match update_prompt_file("article", &filename, &request.content) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' updated successfully", filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update prompt: {}", e),
        })),
    }
}

/// Update an existing news prompt
pub async fn update_news_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
    axum::extract::Json(request): axum::extract::Json<UpdatePromptRequest>,
) -> Json<Value> {
    match update_prompt_file("news", &filename, &request.content) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' updated successfully", filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to update prompt: {}", e),
        })),
    }
}

/// Delete an article prompt
pub async fn delete_article_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<Value> {
    match delete_prompt_file("article", &filename) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' deleted successfully", filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to delete prompt: {}", e),
        })),
    }
}

/// Delete a news prompt
pub async fn delete_news_prompt(
    Extension(_db): Extension<Arc<crate::db::connection::Database>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Json<Value> {
    match delete_prompt_file("news", &filename) {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("Prompt '{}' deleted successfully", filename),
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to delete prompt: {}", e),
        })),
    }
}

// Helper functions

fn get_prompt_dir(prompt_type: &str) -> Result<PathBuf> {
    match prompt_type {
        "article" => {
            // Use the same logic as prompts.rs
            let current_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            
            let possible_paths = [
                PathBuf::from("/app/news-backend/src/writer/prompts/article_randomizer"),
                current_dir.join("src/writer/prompts/article_randomizer"),
                current_dir.join("news-backend/src/writer/prompts/article_randomizer"),
                crate::utils::path_resolver::resolve_workspace_path("news-backend/src/writer/prompts/article_randomizer"),
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.join("src/writer/prompts/article_randomizer")))
                    .unwrap_or_else(|| PathBuf::from("src/writer/prompts/article_randomizer")),
            ];
            
            for path in &possible_paths {
                if path.exists() && path.is_dir() {
                    return Ok(path.clone());
                }
            }
            
            Ok(possible_paths[0].clone())
        }
        "news" => {
            let current_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            
            let possible_paths = [
                current_dir.join("src/writer/prompts/news_randomizer"),
                current_dir.join("news-backend/src/writer/prompts/news_randomizer"),
                crate::utils::path_resolver::resolve_workspace_path("news-backend/src/writer/prompts/news_randomizer"),
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.join("src/writer/prompts/news_randomizer")))
                    .unwrap_or_else(|| PathBuf::from("src/writer/prompts/news_randomizer")),
            ];
            
            for path in &possible_paths {
                if path.exists() && path.is_dir() {
                    return Ok(path.clone());
                }
            }
            
            Ok(possible_paths[0].clone())
        }
        _ => Err(anyhow::anyhow!("Invalid prompt type: {}", prompt_type)),
    }
}

fn get_prompt_files(prompt_type: &str) -> Result<Vec<PromptFile>> {
    let prompt_dir = get_prompt_dir(prompt_type)?;
    let mut files = Vec::new();
    
    if !prompt_dir.exists() {
        // Try to create directory if it doesn't exist
        fs::create_dir_all(&prompt_dir)
            .with_context(|| format!("Failed to create prompt directory: {}", prompt_dir.display()))?;
    }
    
    if prompt_dir.is_dir() {
        let entries = fs::read_dir(&prompt_dir)
            .with_context(|| format!("Failed to read prompt directory: {}", prompt_dir.display()))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "txt" {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        
                        let metadata = fs::metadata(&path)?;
                        let modified = metadata.modified()
                            .ok()
                            .and_then(|t| {
                                t.duration_since(std::time::UNIX_EPOCH)
                                    .ok()
                                    .map(|d| d.as_secs().to_string())
                            })
                            .unwrap_or_else(|| "0".to_string());
                        
                        files.push(PromptFile {
                            filename,
                            size: metadata.len(),
                            modified,
                        });
                    }
                }
            }
        }
    }
    
    // Sort by filename
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    
    Ok(files)
}

fn get_prompt_content(prompt_type: &str, filename: &str) -> Result<String> {
    // Validate filename to prevent directory traversal
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(anyhow::anyhow!("Invalid filename: {}", filename));
    }
    
    let prompt_dir = get_prompt_dir(prompt_type)?;
    let file_path = prompt_dir.join(filename);
    
    // Ensure the file is within the prompt directory
    if !file_path.starts_with(&prompt_dir) {
        return Err(anyhow::anyhow!("Invalid file path"));
    }
    
    fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read prompt file: {}", file_path.display()))
}

fn create_prompt_file(prompt_type: &str, filename: &str, content: &str) -> Result<()> {
    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(anyhow::anyhow!("Invalid filename: {}", filename));
    }
    
    // Ensure .txt extension
    let filename = if filename.ends_with(".txt") {
        filename.to_string()
    } else {
        format!("{}.txt", filename)
    };
    
    let prompt_dir = get_prompt_dir(prompt_type)?;
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&prompt_dir)
        .with_context(|| format!("Failed to create prompt directory: {}", prompt_dir.display()))?;
    
    let file_path = prompt_dir.join(&filename);
    
    // Check if file already exists
    if file_path.exists() {
        return Err(anyhow::anyhow!("Prompt file '{}' already exists", filename));
    }
    
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write prompt file: {}", file_path.display()))?;
    
    Ok(())
}

fn update_prompt_file(prompt_type: &str, filename: &str, content: &str) -> Result<()> {
    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(anyhow::anyhow!("Invalid filename: {}", filename));
    }
    
    let prompt_dir = get_prompt_dir(prompt_type)?;
    let file_path = prompt_dir.join(filename);
    
    // Ensure the file is within the prompt directory
    if !file_path.starts_with(&prompt_dir) {
        return Err(anyhow::anyhow!("Invalid file path"));
    }
    
    // Check if file exists
    if !file_path.exists() {
        return Err(anyhow::anyhow!("Prompt file '{}' does not exist", filename));
    }
    
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to update prompt file: {}", file_path.display()))?;
    
    Ok(())
}

fn delete_prompt_file(prompt_type: &str, filename: &str) -> Result<()> {
    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(anyhow::anyhow!("Invalid filename: {}", filename));
    }
    
    let prompt_dir = get_prompt_dir(prompt_type)?;
    let file_path = prompt_dir.join(filename);
    
    // Ensure the file is within the prompt directory
    if !file_path.starts_with(&prompt_dir) {
        return Err(anyhow::anyhow!("Invalid file path"));
    }
    
    // Check if file exists
    if !file_path.exists() {
        return Err(anyhow::anyhow!("Prompt file '{}' does not exist", filename));
    }
    
    fs::remove_file(&file_path)
        .with_context(|| format!("Failed to delete prompt file: {}", file_path.display()))?;
    
    Ok(())
}

