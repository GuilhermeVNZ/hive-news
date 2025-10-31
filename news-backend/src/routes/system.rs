use axum::response::Json;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct HealthResponse { pub success: bool }

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse{ success: true })
}

#[derive(Debug, Serialize)]
pub struct SystemStatus { pub success: bool, pub output_size_bytes: u64, pub images_size_bytes: u64 }

fn dir_size(start: &Path, filter_ext: Option<&[&str]>) -> u64 {
    let mut total: u64 = 0;
    if !start.exists() { return 0; }
    let mut stack: Vec<PathBuf> = vec![start.to_path_buf()];
    while let Some(p) = stack.pop() {
        if let Ok(rd) = fs::read_dir(p) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() { stack.push(path); } else {
                    if let Ok(md) = e.metadata() {
                        if let Some(exts) = filter_ext {
                            let ok = path.extension().and_then(|s| s.to_str()).map(|s| {
                                let lower = s.to_lowercase();
                                exts.iter().any(|x| *x == lower)
                            }).unwrap_or(false);
                            if ok { total = total.saturating_add(md.len()); }
                        } else {
                            total = total.saturating_add(md.len());
                        }
                    }
                }
            }
        }
    }
    total
}

pub async fn system_status() -> Json<SystemStatus> {
    let output = Path::new("../output");
    let images_ext = ["png", "jpg", "jpeg", "webp", "gif"]; // count only images
    let out_size = dir_size(output, None);
    let img_size = dir_size(output, Some(&images_ext));
    Json(SystemStatus{ success: true, output_size_bytes: out_size, images_size_bytes: img_size })
}






