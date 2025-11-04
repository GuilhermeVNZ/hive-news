use axum::{response::Json, extract::Extension};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::db::connection::Database;
use chrono::Utc;

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

/// Get system configuration including loop_config
pub async fn get_system_config(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    let config_path = get_system_config_path();
    
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    let loop_config = json.get("loop_config").cloned().unwrap_or_else(|| json!({
                        "interval_minutes": 30,
                        "filter_score_min": 0.4,
                        "max_cycles": null,
                        "enabled": false
                    }));
                    
                    Json(json!({
                        "success": true,
                        "loop_config": loop_config
                    }))
                }
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to parse config: {}", e),
                        "loop_config": {
                            "interval_minutes": 30,
                            "filter_score_min": 0.4,
                            "max_cycles": null,
                            "enabled": false
                        }
                    }))
                }
            }
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to read config: {}", e),
                "loop_config": {
                    "interval_minutes": 30,
                    "filter_score_min": 0.4,
                    "max_cycles": null,
                    "enabled": false
                }
            }))
        }
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
        PathBuf::from("G:/Hive-Hub/News-main/news-backend/system_config.json"),
        PathBuf::from("G:/Hive-Hub/News-main/system_config.json"),
    ];
    
    possible_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("system_config.json"))
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
    
    // First, check if start.exe is already running
    let start_running = check_start_running();
    if start_running {
        // If already running, just update the config
        return update_loop_config_only(&config_path, &config).await;
    }
    
    // Read current config
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            let mut json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| json!({}));
            
            // Update loop_config
            json["loop_config"] = json!({
                "interval_minutes": config.interval_minutes,
                "filter_score_min": config.filter_score_min,
                "max_cycles": config.max_cycles,
                "enabled": true
            });
            
            // Write back
            match fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
                Ok(_) => {
                    // Now start start.exe in a new PowerShell window
                    match start_start_exe() {
                        Ok(_) => {
                            Json(json!({
                                "success": true,
                                "message": "Loop configuration updated and started. start.exe launched in new window."
                            }))
                        }
                        Err(e) => {
                            Json(json!({
                                "success": false,
                                "error": format!("Config updated but failed to start start.exe: {}", e)
                            }))
                        }
                    }
                }
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to write config: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to read config: {}", e)
            }))
        }
    }
}

/// Check if start.exe is already running
fn check_start_running() -> bool {
    use std::process::Command;
    
    // Check for start.exe process
    let output = Command::new("powershell")
        .args(&["-Command", "Get-Process | Where-Object { $_.ProcessName -eq 'start' -or $_.Path -like '*start.exe' } | Measure-Object | Select-Object -ExpandProperty Count"])
        .output();
    
    match output {
        Ok(output) => {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                let count = stdout.trim().parse::<u32>().unwrap_or(0);
                count > 0
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Update loop config only (without starting)
async fn update_loop_config_only(
    config_path: &PathBuf,
    config: &LoopConfigRequest,
) -> Json<Value> {
    match fs::read_to_string(config_path) {
        Ok(content) => {
            let mut json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| json!({}));
            
            // Update loop_config
            json["loop_config"] = json!({
                "interval_minutes": config.interval_minutes,
                "filter_score_min": config.filter_score_min,
                "max_cycles": config.max_cycles,
                "enabled": true
            });
            
            // Write back
            match fs::write(config_path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
                Ok(_) => {
                    Json(json!({
                        "success": true,
                        "message": "Loop configuration updated. start.exe is already running."
                    }))
                }
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to write config: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to read config: {}", e)
            }))
        }
    }
}

/// Start start.exe in a new PowerShell window
fn start_start_exe() -> Result<(), String> {
    use std::process::Command;
    
    // Try to find start.exe in release or debug build
    // start.exe is built from News-main/Cargo.toml, so it's in News-main/target/
    let start_paths = vec![
        "G:\\Hive-Hub\\News-main\\target\\release\\start.exe",
        "G:\\Hive-Hub\\News-main\\target\\debug\\start.exe",
        "G:\\Hive-Hub\\News-main\\target\\release\\news-system.exe", // Alternative name
        "G:\\Hive-Hub\\News-main\\target\\debug\\news-system.exe", // Alternative name
    ];
    
    let start_path = start_paths.iter()
        .find(|p| Path::new(p).exists())
        .ok_or_else(|| {
            "start.exe not found. Please build it first: cd News-main && cargo build --bin start".to_string()
        })?;
    
    // Execute start.exe in a new PowerShell window
    // Using Start-Process to open in a new window
    let cmd = format!(
        "Start-Process powershell -ArgumentList '-NoExit', '-Command', 'cd G:\\Hive-Hub\\News-main; Write-Host \"News System Pipeline Loop\" -ForegroundColor Cyan; Write-Host \"=====================================\" -ForegroundColor Cyan; Write-Host \"\"; {} start'",
        start_path
    );
    
    match Command::new("powershell")
        .arg("-Command")
        .arg(&cmd)
        .spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to execute start.exe: {}", e)),
    }
}

/// Stop loop
pub async fn stop_loop(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    let config_path = get_system_config_path();
    
    // First, try to stop the start.exe process
    let stop_result = stop_start_exe();
    
    // Read current config
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            let mut json: Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| json!({}));
            
            // Update loop_config to disabled
            if let Some(loop_config) = json.get_mut("loop_config") {
                if let Some(obj) = loop_config.as_object_mut() {
                    obj.insert("enabled".to_string(), json!(false));
                }
            } else {
                json["loop_config"] = json!({
                    "interval_minutes": 30,
                    "filter_score_min": 0.4,
                    "max_cycles": null,
                    "enabled": false
                });
            }
            
            // Write back
            match fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
                Ok(_) => {
                    let message = if stop_result {
                        "Loop stopped and start.exe process terminated"
                    } else {
                        "Loop configuration updated to disabled. start.exe may still be running."
                    };
                    
                    Json(json!({
                        "success": true,
                        "message": message
                    }))
                }
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to write config: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to read config: {}", e)
            }))
        }
    }
}

/// Stop start.exe process
fn stop_start_exe() -> bool {
    use std::process::Command;
    
    // Try to stop start.exe process
    let output = Command::new("powershell")
        .args(&["-Command", "Get-Process | Where-Object { $_.ProcessName -eq 'start' -or $_.Path -like '*start.exe' } | Stop-Process -Force -ErrorAction SilentlyContinue"])
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Refresh servers (execute servers.exe)
pub async fn refresh_servers(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
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
pub async fn get_collection_status(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    let config_path = get_system_config_path();
    
    // Read loop config
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    if let Some(loop_config) = json.get("loop_config") {
                        let interval = loop_config.get("interval_minutes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(30);
                        let filter_score = loop_config.get("filter_score_min")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.4);
                        let max_cycles = loop_config.get("max_cycles")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32);
                        let enabled = loop_config.get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        // Calculate cooldown
                        // Try to find last cycle completion time from stats file
                        let stats_path = Path::new("G:/Hive-Hub/News-main/loop_stats.json");
                        let cooldown_remaining = if enabled && stats_path.exists() {
                            if let Ok(stats_content) = fs::read_to_string(stats_path) {
                                if let Ok(stats_json) = serde_json::from_str::<Value>(&stats_content) {
                                    if let Some(last_cycle) = stats_json.get("last_cycle_completed_at") {
                                        if let Some(timestamp) = last_cycle.as_str() {
                                            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(timestamp) {
                                                let now = Utc::now();
                                                let last_utc = last_time.with_timezone(&Utc);
                                                let elapsed = now.signed_duration_since(last_utc);
                                                let total_seconds = (interval * 60) as i64;
                                                let remaining = total_seconds - elapsed.num_seconds();
                                                if remaining > 0 {
                                                    remaining
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
    Json(json!({
        "success": true,
        "interval_minutes": 30,
        "filter_score_min": 0.4,
        "max_cycles": null,
        "enabled": false,
        "cooldown_remaining_seconds": 0,
        "cooldown_total_seconds": 1800,
    }))
}

/// Get loop statistics (articles by source, articles written by site, tokens)
pub async fn get_loop_stats(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
    let stats_path = Path::new("G:/Hive-Hub/News-main/loop_stats.json");
    
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
    
    match fs::read_to_string(stats_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    Json(json!({
                        "success": true,
                        "current_cycle": json.get("current_cycle").unwrap_or(&json!(0)),
                        "articles_by_source": json.get("articles_by_source").unwrap_or(&json!({})),
                        "articles_written_by_site": json.get("articles_written_by_site").unwrap_or(&json!({})),
                        "tokens_total": json.get("tokens_total").unwrap_or(&json!(0)),
                        "tokens_saved": json.get("tokens_saved").unwrap_or(&json!(0)),
                        "tokens_used": json.get("tokens_used").unwrap_or(&json!(0)),
                        "last_cycle_completed_at": json.get("last_cycle_completed_at").unwrap_or(&json!(null)),
                    }))
                }
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to parse stats: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to read stats: {}", e)
            }))
        }
    }
}

/// Get services status (backend and frontends)
pub async fn get_services_status(
    Extension(_db): Extension<Arc<Database>>,
) -> Json<Value> {
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
        "vectorizer": {
            "name": "Vectorizer",
            "url": "http://localhost:15002",
            "online": check_port(15002),
        },
        "synap": {
            "name": "Synap",
            "url": "http://localhost:15500",
            "online": check_port(15500),
        },
    });
    
    Json(json!({
        "success": true,
        "services": services,
    }))
}











































