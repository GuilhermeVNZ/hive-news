use axum::{extract::Extension, response::Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    db::connection::Database,
    services::loop_manager::{LoopConfig, LoopManager},
    utils::path_resolver::{resolve_workspace_path, workspace_root},
};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub success: bool,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { success: true })
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub success: bool,
    pub output_size_bytes: u64,
    pub images_size_bytes: u64,
}

fn dir_size(start: &Path, filter_ext: Option<&[&str]>) -> u64 {
    let mut total: u64 = 0;
    if !start.exists() {
        return 0;
    }
    let mut stack: Vec<PathBuf> = vec![start.to_path_buf()];
    while let Some(p) = stack.pop() {
        if let Ok(rd) = fs::read_dir(p) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    if let Ok(md) = e.metadata() {
                        if let Some(exts) = filter_ext {
                            let ok = path
                                .extension()
                                .and_then(|s| s.to_str())
                                .map(|s| {
                                    let lower = s.to_lowercase();
                                    exts.iter().any(|x| *x == lower)
                                })
                                .unwrap_or(false);
                            if ok {
                                total = total.saturating_add(md.len());
                            }
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
    Json(SystemStatus {
        success: true,
        output_size_bytes: out_size,
        images_size_bytes: img_size,
    })
}

/// Get system configuration including loop_config
pub async fn get_system_config(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    let config_path = get_system_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(json) => {
                let loop_config = json.get("loop_config").cloned().unwrap_or_else(|| {
                    json!({
                        "interval_minutes": 30,
                        "filter_score_min": 0.4,
                        "max_cycles": null,
                        "enabled": false
                    })
                });

                Json(json!({
                    "success": true,
                    "loop_config": loop_config
                }))
            }
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to parse config: {}", e),
                "loop_config": {
                    "interval_minutes": 30,
                    "filter_score_min": 0.4,
                    "max_cycles": null,
                    "enabled": false
                }
            })),
        },
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to read config: {}", e),
            "loop_config": {
                "interval_minutes": 30,
                "filter_score_min": 0.4,
                "max_cycles": null,
                "enabled": false
            }
        })),
    }
}

/// Helper to get system config path
fn get_system_config_path() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let possible_paths = vec![
        current_dir.join("system_config.json"),
        current_dir.join("news-backend/system_config.json"),
        PathBuf::from("system_config.json"),
        PathBuf::from("news-backend/system_config.json"),
        resolve_workspace_path("system_config.json"),
        resolve_workspace_path("news-backend/system_config.json"),
    ];

    for path in possible_paths {
        if path.exists() {
            return path;
        }
    }

    workspace_root().join("news-backend/system_config.json")
}

#[derive(Debug, Deserialize)]
pub struct LoopConfigRequest {
    pub interval_minutes: u64,
    pub filter_score_min: f64,
    pub max_cycles: Option<u32>,
}

/// Start loop with configuration
pub async fn start_loop(
    Extension(_db): Extension<Arc<Database>>,
    axum::extract::Json(config): axum::extract::Json<LoopConfigRequest>,
) -> Json<Value> {
    let config_path = get_system_config_path();

    if let Err(e) = write_loop_config(&config_path, &config, true) {
        return Json(json!({
            "success": false,
            "error": e,
        }));
    }

    match LoopManager::global().start(LoopConfig::from(&config)).await {
        Ok(true) => Json(json!({
            "success": true,
            "message": "Loop configuration updated and pipeline loop started."
        })),
        Ok(false) => Json(json!({
            "success": true,
            "message": "Loop configuration updated. Pipeline loop already running."
        })),
        Err(e) => {
            if let Err(err) = write_loop_config_enabled_only(&config_path, false) {
                tracing::warn!(
                    "Failed to revert loop enabled flag after start error: {}",
                    err
                );
            }
            Json(json!({
                "success": false,
                "error": format!("Failed to start pipeline loop: {}", e)
            }))
        }
    }
}

/// Stop loop
pub async fn stop_loop(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    let config_path = get_system_config_path();

    if let Err(e) = write_loop_config_enabled_only(&config_path, false) {
        return Json(json!({
            "success": false,
            "error": e,
        }));
    }

    let stopped = match LoopManager::global().stop().await {
        Ok(result) => result,
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to stop pipeline loop: {}", e)
            }));
        }
    };

    let message = if stopped {
        "Pipeline loop stopped."
    } else {
        "Pipeline loop was not running."
    };

    Json(json!({
        "success": true,
        "message": message
    }))
}

/// Refresh servers (execute servers.exe)
pub async fn refresh_servers(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    use std::process::Command;

    let servers_path = "G:\\Hive-Hub\\News-main\\news-backend\\target\\release\\servers.exe";
    let servers_path_debug = "G:\\Hive-Hub\\News-main\\news-backend\\target\\debug\\servers.exe";

    let path = if Path::new(servers_path).exists() {
        servers_path
    } else if Path::new(servers_path_debug).exists() {
        servers_path_debug
    } else {
        return Json(json!({
            "success": false,
            "error": "servers.exe not found. Please build it first: cd news-backend && cargo build --bin servers"
        }));
    };

    // Execute servers.exe in a new PowerShell window
    match Command::new("powershell")
        .args(&["-Command", &format!("Start-Process powershell -ArgumentList '-NoExit', '-Command', 'cd G:\\Hive-Hub\\News-main; {}'", path)])
        .spawn() {
        Ok(_) => {
            Json(json!({
                "success": true,
                "message": "Servers refresh initiated"
            }))
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to execute servers.exe: {}", e)
            }))
        }
    }
}

/// Get collection status with cooldown
pub async fn get_collection_status(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    let config_path = get_system_config_path();

    // Read loop config
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    if let Some(loop_config) = json.get("loop_config") {
                        let mut interval = loop_config
                            .get("interval_minutes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(30);
                        if interval == 0 {
                            interval = 1;
                        }

                        let filter_score = loop_config
                            .get("filter_score_min")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.4);
                        let max_cycles = loop_config
                            .get("max_cycles")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);
                        let mut enabled = loop_config
                            .get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if !enabled && LoopManager::global().is_running().await {
                            enabled = true;
                        }

                        // Calculate cooldown
                        // Try to find last cycle completion time from stats file
                        let stats_path = loop_stats_path();
                        let cooldown_remaining = if enabled && stats_path.exists() {
                            if let Ok(stats_content) = fs::read_to_string(stats_path) {
                                if let Ok(stats_json) =
                                    serde_json::from_str::<Value>(&stats_content)
                                {
                                    if let Some(last_cycle) =
                                        stats_json.get("last_cycle_completed_at")
                                    {
                                        if let Some(timestamp) = last_cycle.as_str() {
                                            if let Ok(last_time) =
                                                chrono::DateTime::parse_from_rfc3339(timestamp)
                                            {
                                                let now = Utc::now();
                                                let last_utc = last_time.with_timezone(&Utc);
                                                let elapsed = now.signed_duration_since(last_utc);
                                                let total_seconds = (interval * 60) as i64;
                                                let remaining =
                                                    total_seconds - elapsed.num_seconds();
                                                if remaining > 0 { remaining } else { 0 }
                                            } else {
                                                0
                                            }
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        return Json(json!({
                            "success": true,
                            "interval_minutes": interval,
                            "filter_score_min": filter_score,
                            "max_cycles": max_cycles,
                            "enabled": enabled,
                            "cooldown_remaining_seconds": cooldown_remaining,
                            "cooldown_total_seconds": interval * 60,
                        }));
                    }
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    };

    // Defaults
    let running = LoopManager::global().is_running().await;

    Json(json!({
        "success": true,
        "interval_minutes": 30,
        "filter_score_min": 0.4,
        "max_cycles": null,
        "enabled": running,
        "cooldown_remaining_seconds": 0,
        "cooldown_total_seconds": 1800,
    }))
}

/// Get loop statistics (articles by source, articles written by site, tokens)
pub async fn get_loop_stats(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    let stats_path = loop_stats_path();

    if !stats_path.exists() {
        return Json(json!({
            "success": true,
            "current_cycle": 0,
            "articles_by_source": {},
            "articles_written_by_site": {},
            "tokens_total": 0,
            "tokens_saved": 0,
            "tokens_used": 0,
            "last_cycle_completed_at": null,
        }));
    }

    match fs::read_to_string(&stats_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(json) => Json(json!({
                "success": true,
                "current_cycle": json.get("current_cycle").unwrap_or(&json!(0)),
                "articles_by_source": json.get("articles_by_source").unwrap_or(&json!({})),
                "articles_written_by_site": json.get("articles_written_by_site").unwrap_or(&json!({})),
                "tokens_total": json.get("tokens_total").unwrap_or(&json!(0)),
                "tokens_saved": json.get("tokens_saved").unwrap_or(&json!(0)),
                "tokens_used": json.get("tokens_used").unwrap_or(&json!(0)),
                "last_cycle_completed_at": json.get("last_cycle_completed_at").unwrap_or(&json!(null)),
            })),
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to parse stats: {}", e)
            })),
        },
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to read stats: {}", e)
        })),
    }
}

/// Get services status (backend and frontends)
pub async fn get_services_status(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    use std::net::TcpStream;

    // Check backend (port 3005) - if request reached here, backend is online
    let backend_online = true;

    // Check frontends - if request reached here, dashboard is online (it made the request!)
    // For other services, try to connect
    let check_port = |port: u16| -> bool {
        // Quick check with timeout
        if let Ok(stream) = TcpStream::connect(format!("127.0.0.1:{}", port)) {
            drop(stream);
            true
        } else {
            false
        }
    };

    let services = json!({
        "backend": {
            "name": "News Backend",
            "url": "http://localhost:3005",
            "online": backend_online,
        },
        "news_dashboard": {
            "name": "News Dashboard",
            "url": "http://localhost:1420",
            "online": true, // If request reached here, dashboard is definitely online!
        },
        "airesearch": {
            "name": "AIResearch",
            "url": "http://localhost:3003",
            "online": check_port(3003),
        },
        "scienceai": {
            "name": "ScienceAI",
            "url": "http://localhost:8080",
            "online": check_port(8080),
        },
    });

    Json(json!({
        "success": true,
        "services": services,
    }))
}

/// Get count of articles published today
pub async fn get_articles_today_count(Extension(_db): Extension<Arc<Database>>) -> Json<Value> {
    use crate::utils::article_registry::{ArticleStatus, RegistryManager};
    use chrono::Utc;

    let registry_path = registry_file_path();

    if !registry_path.exists() {
        return Json(json!({
            "success": true,
            "count": 0,
        }));
    }

    match RegistryManager::new(&registry_path) {
        Ok(manager) => {
            let today = Utc::now().date_naive();
            let count = manager
                .get_all_articles()
                .iter()
                .filter(|article| {
                    // Only count Published articles
                    if !matches!(article.status, ArticleStatus::Published) {
                        return false;
                    }

                    // Check if published_at is today
                    if let Some(published_at) = &article.published_at {
                        return published_at.date_naive() == today;
                    }

                    // Fallback: check collected_at if published_at is not available
                    if let Some(collected_at) = &article.collected_at {
                        return collected_at.date_naive() == today;
                    }

                    false
                })
                .count();

            Json(json!({
                "success": true,
                "count": count,
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to load registry: {}", e),
            "count": 0,
        })),
    }
}

fn workspace_root_from_config() -> PathBuf {
    let config_path = get_system_config_path();
    config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn loop_stats_path() -> PathBuf {
    let root = workspace_root_from_config();
    root.join("loop_stats.json")
}

fn registry_file_path() -> PathBuf {
    let root = workspace_root_from_config();
    root.join("articles_registry.json")
}

fn write_loop_config(
    config_path: &Path,
    config: &LoopConfigRequest,
    enabled: bool,
) -> Result<(), String> {
    let mut json = read_or_create_config(config_path)?;
    if !json.is_object() {
        json = json!({});
    }

    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "loop_config".to_string(),
            json!({
                "interval_minutes": config.interval_minutes,
                "filter_score_min": config.filter_score_min,
                "max_cycles": config.max_cycles,
                "enabled": enabled
            }),
        );
    }

    write_config(config_path, &json)
}

fn write_loop_config_enabled_only(config_path: &Path, enabled: bool) -> Result<(), String> {
    let mut json = read_or_create_config(config_path)?;
    if !json.is_object() {
        json = json!({});
    }

    if let Some(obj) = json.as_object_mut() {
        let loop_config = obj.entry("loop_config".to_string()).or_insert_with(|| {
            json!({
                "interval_minutes": 30,
                "filter_score_min": 0.4,
                "max_cycles": Value::Null,
                "enabled": false
            })
        });

        if let Some(loop_obj) = loop_config.as_object_mut() {
            loop_obj.insert("enabled".to_string(), json!(enabled));
        } else {
            obj.insert(
                "loop_config".to_string(),
                json!({
                    "interval_minutes": 30,
                    "filter_score_min": 0.4,
                    "max_cycles": Value::Null,
                    "enabled": enabled
                }),
            );
        }
    }

    write_config(config_path, &json)
}

fn read_or_create_config(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }

    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config: {}", e))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config JSON: {}", e))
        })
}

fn write_config(path: &Path, json_value: &Value) -> Result<(), String> {
    serde_json::to_string_pretty(json_value)
        .map_err(|e| format!("Failed to serialize config: {}", e))
        .and_then(|content| {
            fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))
        })
}

impl From<&LoopConfigRequest> for LoopConfig {
    fn from(value: &LoopConfigRequest) -> Self {
        Self {
            interval_minutes: value.interval_minutes.max(1),
            filter_score_min: value.filter_score_min,
            max_cycles: value.max_cycles,
        }
    }
}
