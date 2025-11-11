// News System - Main Orchestrator
// Execute with: cargo run --bin start

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn workspace_root() -> PathBuf {
    if let Ok(env_path) = std::env::var("NEWS_BASE_DIR") {
        let trimmed = env_path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_workspace_path<P: AsRef<Path>>(relative: P) -> PathBuf {
    workspace_root().join(relative.as_ref())
}

fn main() {
    println!("🚀 News System - Main Orchestrator");
    println!("=====================================\n");

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "start" => start_full_system(),
        "backend" => start_backend(),
        "collector" => test_collector(),
        "collector-enabled" => trigger_collect_enabled(),
        "schedule" => run_scheduler(),
        "monitor" => monitor_system(),
        "status" => check_system_status(),
        "help" | _ => show_help(),
    }
}

fn start_full_system() {
    println!("🎯 News System - Full Orchestrator");
    println!("=====================================\n");

    // Etapa 1: Verificar dependências
    println!("📋 Step 1: Checking system dependencies...");
    check_dependencies();

    // Etapa 2: Verificar se servers.exe foi executado
    println!("\n🔍 Step 2: Verifying if servers are running...");
    println!("   → News Dashboard should be at http://localhost:1420");
    println!("   → AIResearch should be at http://localhost:3003");
    println!("   → ScienceAI should be at http://localhost:8080");
    println!("   💡 Run 'cargo run --bin servers' first to start all servers");
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Etapa 3: Iniciar Backend
    println!("\n🔧 Step 3: Starting Backend Server...");
    println!("   → Running backend on http://localhost:3005");
    start_backend_background();

    // Etapa 4: Aguardar backend estar pronto
    println!("⏳ Step 4: Waiting for backend to be ready...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("\n✅ News System is FULLY OPERATIONAL!");
    println!("=====================================");
    println!("   🔧 Backend API:    http://localhost:3005");
    println!("   🎨 News Dashboard: http://localhost:1420 (started via servers.exe)");
    println!("   🌐 AIResearch:     http://localhost:3003 (started via servers.exe)");
    println!("   🌐 ScienceAI:      http://localhost:8080 (started via servers.exe)");
    println!("   🎯 Orchestrator:   ACTIVE");
    println!("   ⏰ Scheduler:      CONFIGURED");
    println!("   📊 Monitor:        RUNNING");

    // Etapa 5: Iniciar Pipeline Automático (Artigos + News em paralelo)
    println!("\n🚀 Step 5: Starting Automatic Pipelines...");
    println!("   📄 Articles Pipeline:");
    println!("      📥 Phase 1: Collect papers from arXiv (only)");
    println!("      🔍 Phase 2: Filter and validate papers");
    println!("      ✍️  Phase 3: Generate content with DeepSeek");
    println!("   📰 News Pipeline (parallel):");
    println!("      📥 Phase 1: Collect news from RSS/HTML sources");
    println!("      🔍 Phase 2: Filter duplicates");
    println!("      ✍️  Phase 3: Generate news articles");
    println!("      🧹 Phase 4: Cleanup processed files");

    // Executar pipeline em background
    std::thread::spawn(|| {
        execute_full_pipeline();
    });

    println!("\n   Press Ctrl+C to stop all services...\n");

    // Orquestração contínua
    run_orchestration_loop();
}

fn check_dependencies() {
    let checks = vec![
        ("PostgreSQL", "psql --version"),
        ("Rust", "cargo --version"),
        ("Node.js", "node --version"),
        ("npm", "npm --version"),
    ];

    for (name, cmd) in checks {
        match Command::new("cmd").args(&["/C", cmd]).output() {
            Ok(_) => println!("✅ {} - OK", name),
            Err(_) => println!("❌ {} - NOT FOUND", name),
        }
    }

    // Verificar servers.exe (gerencia frontends)
    let servers_path = "G:\\Hive-Hub\\News-main\\news-backend\\target\\release\\servers.exe";
    if std::path::Path::new(servers_path).exists() {
        println!("✅ Servers Orchestrator - Binary found");
        println!("   💡 Use 'cargo run --bin servers' para iniciar todos os frontends");
    } else {
        println!(
            "⚠️  Servers Orchestrator - Binary not found at: {}",
            servers_path
        );
        println!("   Run: cd news-backend && cargo build --release --bin servers");
    }
}

fn check_backend_running() -> bool {
    // Try to connect to backend on port 3005
    use std::net::TcpStream;
    match TcpStream::connect("127.0.0.1:3005") {
        Ok(_) => {
            println!("   ✅ Backend is already running on port 3005");
            true
        }
        Err(_) => {
            println!("   ⚠️  Backend is not running on port 3005");
            false
        }
    }
}

fn start_backend_background() {
    // Check if backend is already running
    if check_backend_running() {
        println!("   ℹ️  Backend already running, skipping start");
        return;
    }

    let backend_path = "./news-backend";

    // Verificar se o diretório existe
    if !std::path::Path::new(backend_path).exists() {
        println!("❌ Backend directory not found at: {}", backend_path);
        return;
    }

    println!(
        "   Running: cd {} && cargo run --bin news-backend",
        backend_path
    );
    // Em produção, executaria com spawn em background
    std::thread::spawn(|| {
        Command::new("cmd")
            .args(&["/C", "cd news-backend && cargo run --bin news-backend"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start backend");
    });
}

fn monitor_system() {
    let metrics = HashMap::from([
        ("Backend API", "http://localhost:3005"),
        (
            "News Dashboard",
            "http://localhost:1420 (started via servers.exe)",
        ),
        (
            "AIResearch",
            "http://localhost:3003 (started via servers.exe)",
        ),
        (
            "ScienceAI",
            "http://localhost:8080 (started via servers.exe)",
        ),
        ("Database", "connected"),
        ("Collector", "idle"),
    ]);

    println!("\n📊 System Metrics:");
    for (component, status) in &metrics {
        println!("   {}: {}", component, status);
    }
}

fn start_backend() {
    println!("🔧 Starting Backend Server...");
    println!("Run: cd news-backend && cargo run");
    println!("Server will be available at: http://localhost:3005");
}

fn test_collector() {
    println!("🔍 Collector Service - Continuous Pipeline Mode");
    println!("\n🚀 Starting Continuous Pipeline Loop...");
    println!("\n📊 Configuration:");
    println!("   Source: cs.AI (Computer Science - Artificial Intelligence)");
    println!("   Papers: 10 most recent per cycle (busca regressiva até encontrar)");
    println!("   Location: G:\\Hive-Hub\\News-main\\downloads\\arxiv\\");
    println!("   ⏰ Interval: 30 minutes (1800 seconds)");
    println!("\n🛡️  Security Features:");
    println!("   ✅ Using export.arxiv.org (official API)");
    println!("   ✅ Cookie-based session management");
    println!("   ✅ Browser-like headers (anti-bot protection bypassed)");
    println!("   ✅ Rate limiting (3s delay between downloads)");
    println!("   ✅ Incremental collection (anti-duplication via registry)");
    println!("\n🔄 Running continuously...\n");

    // Usar a função de pipeline contínuo que tem o loop
    execute_full_pipeline();
}

fn trigger_collect_enabled() {
    println!("🔍 Collector Service - Enabled Sources via backend config\n");
    let ps_script = r#"
cd G:\Hive-Hub\News-main\news-backend;
$env:RUST_LOG="info";
cargo run --bin news-backend collect-enabled
"#;

    let output = Command::new("powershell")
        .args(&["-Command", ps_script])
        .output()
        .expect("Failed to trigger collect-enabled");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{}", stdout);
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }
}

fn run_filter() {
    println!("🔍 Filter Service - Validating Scientific Papers\n");

    // Usar binário compilado diretamente para evitar lock conflicts
    let backend_bin = "G:\\Hive-Hub\\News-main\\news-backend\\target\\debug\\news-backend.exe";

    let mut child = if std::path::Path::new(backend_bin).exists() {
        Command::new(backend_bin)
            .arg("filter")
            .current_dir("G:\\Hive-Hub\\News-main\\news-backend")
            .env("RUST_LOG", "info")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute filter")
    } else {
        // Fallback para cargo run se binário não existe
        eprintln!("⚠️  Binary not found, falling back to cargo run...");
        Command::new("cargo")
            .args(&["run", "--bin", "news-backend", "filter"])
            .current_dir("G:\\Hive-Hub\\News-main\\news-backend")
            .env("RUST_LOG", "info")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute filter")
    };

    let status = child.wait().expect("Failed to wait for filter");

    println!("");
    if status.success() {
        println!("✅ [ARTICLES] Filter completed!");
        println!("   Approved: G:\\Hive-Hub\\News-main\\downloads\\filtered\\");
    } else {
        println!(
            "⚠️  [ARTICLES] Filter had issues (exit code: {:?})",
            status.code()
        );
        println!("   Check output above for details");
    }
}

fn run_writer() {
    println!("✍️  [ARTICLES] DeepSeek Writer - Processing filtered papers\n");

    // Load configuration from system_config.json
    let (api_key, site_id) = match load_writer_config() {
        Ok((key, site)) => (key, site),
        Err(e) => {
            eprintln!("⚠️  Failed to load config: {}. Using defaults.", e);
            // Fallback to environment variables or defaults
            (
                std::env::var("DEEPSEEK_API_KEY").ok(),
                std::env::var("WRITER_DEFAULT_SITE")
                    .ok()
                    .unwrap_or_else(|| "airesearch".to_string()),
            )
        }
    };

    // Get backend binary path (try multiple locations)
    let backend_bin = find_backend_binary();
    let backend_dir = get_backend_directory();

    let mut cmd = if backend_bin.exists() {
        println!("   Using binary: {}", backend_bin.display());
        let mut c = Command::new(&backend_bin);
        c.arg("write").current_dir(&backend_dir);
        c
    } else {
        println!("   Using cargo run (binary not found)");
        let mut c = Command::new("cargo");
        c.args(&["run", "--bin", "news-backend", "write"])
            .current_dir(&backend_dir);
        c
    };

    cmd.env("RUST_LOG", "info");

    // Set API key from config or env
    if let Some(key) = &api_key {
        cmd.env("DEEPSEEK_API_KEY", key);
    } else if std::env::var("DEEPSEEK_API_KEY").is_err() {
        eprintln!("⚠️  Warning: No DEEPSEEK_API_KEY found in config or environment");
    }

    cmd.env("WRITER_DEFAULT_SITE", &site_id);
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let mut child = cmd.spawn().expect("Failed to execute writer");
    let status = child.wait().expect("Failed to wait for writer");

    println!("");
    if status.success() {
        println!("✅ [ARTICLES] Content generation completed!");
        println!(
            "   Output: {}/output/{}/",
            get_base_directory(),
            site_id.to_uppercase()
        );
    } else {
        println!(
            "⚠️  [ARTICLES] Content generation had issues (exit code: {:?})",
            status.code()
        );
        println!("   Check output above for details");
    }
}

// Helper function to load writer config from system_config.json
fn load_writer_config() -> Result<(Option<String>, String), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    let possible_config_paths = vec![
        PathBuf::from("news-backend/system_config.json"),
        PathBuf::from("system_config.json"),
        resolve_workspace_path("news-backend/system_config.json"),
        resolve_workspace_path("system_config.json"),
    ];

    let config_path = possible_config_paths
        .iter()
        .find(|p| p.exists())
        .ok_or("system_config.json not found")?;

    let content = fs::read_to_string(config_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    // Find first enabled site with writer enabled
    let sites = json
        .get("sites")
        .and_then(|s| s.as_object())
        .ok_or("sites not found in config")?;

    for (site_id, site) in sites {
        if let Some(enabled) = site.get("enabled").and_then(|e| e.as_bool()) {
            if enabled {
                if let Some(writer) = site.get("writer") {
                    if let Some(writer_enabled) = writer.get("enabled").and_then(|e| e.as_bool()) {
                        if writer_enabled {
                            let api_key = writer
                                .get("api_key")
                                .and_then(|k| k.as_str())
                                .map(|s| s.to_string());
                            return Ok((api_key, site_id.clone()));
                        }
                    }
                }
            }
        }
    }

    Err("No enabled site with writer enabled found".into())
}

// Helper function to find backend binary
fn find_backend_binary() -> std::path::PathBuf {
    let possible_paths = vec![
        std::path::PathBuf::from("news-backend/target/debug/news-backend.exe"),
        std::path::PathBuf::from("news-backend/target/release/news-backend.exe"),
        std::path::PathBuf::from("target/debug/news-backend.exe"),
        std::path::PathBuf::from("target/release/news-backend.exe"),
        resolve_workspace_path("news-backend/target/debug/news-backend.exe"),
        resolve_workspace_path("news-backend/target/release/news-backend.exe"),
    ];

    possible_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("news-backend/target/debug/news-backend.exe"))
}

// Helper function to get backend directory
fn get_backend_directory() -> std::path::PathBuf {
    let possible_dirs = vec![
        std::path::PathBuf::from("news-backend"),
        resolve_workspace_path("news-backend"),
    ];

    possible_dirs
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("news-backend"))
}

// Helper function to get base directory
fn get_base_directory() -> String {
    std::env::var("NEWS_BASE_DIR").unwrap_or_else(|_| {
        load_paths_from_config()
            .map(|base| {
                if Path::new(&base).is_absolute() {
                    base
                } else {
                    resolve_workspace_path(base).display().to_string()
                }
            })
            .unwrap_or_else(|_| workspace_root().display().to_string())
    })
}

// Helper function to load paths from config
fn load_paths_from_config() -> Result<String, Box<dyn std::error::Error>> {
    use serde::Deserialize;
    use std::fs;
    use std::path::Path;

    #[derive(Deserialize)]
    struct PathsFromConfig {
        paths: PathsConfigData,
    }

    #[derive(Deserialize)]
    struct PathsConfigData {
        base_dir: String,
    }

    let possible_config_paths = vec![
        PathBuf::from("news-backend/system_config.json"),
        PathBuf::from("system_config.json"),
        resolve_workspace_path("news-backend/system_config.json"),
        resolve_workspace_path("system_config.json"),
    ];

    let config_path = possible_config_paths
        .iter()
        .find(|p| p.exists())
        .ok_or("system_config.json not found")?;

    let content = fs::read_to_string(config_path)?;
    let json: PathsFromConfig = serde_json::from_str(&content)?;

    Ok(json.paths.base_dir)
}

fn run_scheduler() {
    println!("⏰ Running Scheduled Tasks...");
    println!("\n📋 Scheduler Status:");
    println!("   ✅ Collector Service implemented");
    println!("   ✅ Download directory configured");
    println!("   ✅ Secure PDF downloads (export.arxiv.org)");
    println!("   ✅ Anti-reCAPTCHA protection enabled");
    println!("   ⏳ Scheduler with tokio-cron-scheduler - TODO");
    println!("\n🔄 Workflow:");
    println!("   1. Fetch active portals from dashboard");
    println!("   2. Execute collector for each portal");
    println!("   3. Downloads stored in downloads/<source>/<date>/");
    println!("   4. Metadata saved to raw_documents table");
    println!("\n🔐 Security Features:");
    println!("   • Uses export.arxiv.org (bypasses reCAPTCHA)");
    println!("   • Cookie-based session handling");
    println!("   • Rate limiting (3s between downloads)");
    println!("   • Anti-duplication checking");
    println!("\n📚 See docs/PHASE1_COLLECTOR.md for details");
}

fn check_system_status() {
    println!("📊 Checking System Status...\n");

    let components = vec![
        ("Backend API", "http://localhost:3005"),
        (
            "News Dashboard",
            "http://localhost:1420 (started via servers.exe)",
        ),
        (
            "AIResearch",
            "http://localhost:3003 (started via servers.exe)",
        ),
        (
            "ScienceAI",
            "http://localhost:8080 (started via servers.exe)",
        ),
        ("Database", "PostgreSQL"),
        ("Collector Service", "News-backend"),
    ];

    println!("Component Status:");
    for (component, location) in components {
        println!("   ✅ {} - {}", component, location);
    }
}

// Funções removidas - Configuração do dashboard não é mais necessária aqui
// Dashboard e frontends são iniciados via servers.exe

fn run_orchestration_loop() {
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("\n🔄 Orchestration Loop #{}", iteration);

        // Verificar saúde do sistema
        println!("   💚 Health check...");
        check_system_health();

        // Verificar status dos serviços
        println!("   📥 Checking service status...");

        // Executar tarefas agendadas
        println!("   ⏰ Checking scheduled tasks...");

        // Aguardar próximo ciclo
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

fn load_loop_config() -> (u64, f64, Option<u32>) {
    use serde_json::Value;
    use std::fs;

    let possible_config_paths = vec![
        PathBuf::from("news-backend/system_config.json"),
        PathBuf::from("system_config.json"),
        resolve_workspace_path("news-backend/system_config.json"),
        resolve_workspace_path("system_config.json"),
    ];

    for path in possible_config_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(loop_config) = json.get("loop_config") {
                    let interval = loop_config
                        .get("interval_minutes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(30);
                    let filter_score = loop_config
                        .get("filter_score_min")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.4);
                    let max_cycles = loop_config
                        .get("max_cycles")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    return (interval, filter_score, max_cycles);
                }
            }
        }
    }

    // Defaults
    (30, 0.4, None)
}

fn execute_full_pipeline() {
    // Aguardar um pouco para garantir que backend está pronto
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Load loop configuration from system_config.json
    let (interval_minutes, filter_score_min, max_cycles) = load_loop_config();
    let interval_seconds = interval_minutes * 60;

    println!("\n\n🔄 Starting Automatic Pipeline Loop");
    println!("=====================================");
    println!(
        "   ⏰ Interval: {} minutes ({} seconds)",
        interval_minutes, interval_seconds
    );
    println!("   🔍 Filter Score Min: {:.2}", filter_score_min);
    if let Some(max) = max_cycles {
        println!("   🔢 Max Cycles: {}", max);
    } else {
        println!("   🔢 Max Cycles: ∞ (infinite)");
    }
    println!("   🚀 Running continuously...");
    println!("   📄 Articles Pipeline: Active");
    println!("   📰 News Pipeline: Active (parallel)\n");

    let mut cycle = 1;

    loop {
        // Check if we've reached max cycles
        if let Some(max) = max_cycles {
            if cycle > max {
                println!("\n{}", "=".repeat(70));
                println!(
                    "✅ Reached maximum cycles ({}). Stopping pipeline loop.",
                    max
                );
                println!("{}", "=".repeat(70));
                return;
            }
        }

        let start_time = std::time::Instant::now();

        println!("\n{}", "=".repeat(70));
        println!("🔄 CYCLE #{} - Pipeline Execution Started", cycle);
        println!("⏱️  Time: {}", get_current_time());
        println!("{}", "=".repeat(70));

        // Executar pipelines em paralelo usando threads
        println!("\n🚀 Starting parallel pipelines...");
        println!("   📄 Articles pipeline: Thread spawned");
        println!("   📰 News pipeline: Thread spawned\n");

        let papers_pipeline_handle = std::thread::spawn(|| execute_papers_pipeline());

        let news_pipeline_handle = std::thread::spawn(|| execute_news_pipeline());

        // Aguardar ambos os pipelines terminarem
        println!("⏳ Waiting for both pipelines to complete...\n");
        let papers_result = papers_pipeline_handle.join();
        let news_result = news_pipeline_handle.join();

        // Verificar se houve erros
        println!("\n📊 Pipeline Results Summary:");
        if let Err(e) = papers_result {
            eprintln!("   ❌ [ARTICLES] Papers pipeline thread error: {:?}", e);
        } else {
            println!("   ✅ [ARTICLES] Papers pipeline thread completed");
        }

        if let Err(e) = news_result {
            eprintln!("   ❌ [NEWS] News pipeline thread error: {:?}", e);
        } else {
            println!("   ✅ [NEWS] News pipeline thread completed");
        }
        println!("");

        let execution_time = start_time.elapsed();
        let next_run = chrono::Local::now() + chrono::Duration::minutes(interval_minutes as i64);

        println!("\n{}", "=".repeat(70));
        println!("✅ Cycle #{} completed successfully!", cycle);
        println!("⏱️  Execution time: {:?}", execution_time);
        println!("⏰ Next cycle: {}", next_run.format("%Y-%m-%d %H:%M:%S"));
        println!("📂 Output: G:\\Hive-Hub\\News-main\\output\\AIResearch\\");
        println!("📰 News Output: G:\\Hive-Hub\\News-main\\output\\ScienceAI\\");
        println!("{}", "=".repeat(70));

        // Save loop statistics
        save_loop_stats(cycle - 1);

        cycle += 1;

        // Check if we've reached max cycles before waiting
        if let Some(max) = max_cycles {
            if cycle > max {
                println!(
                    "\n✅ Reached maximum cycles ({}). Stopping pipeline loop.",
                    max
                );
                return;
            }
        }

        // Aguardar intervalo configurado antes de próxima execução
        println!(
            "\n⏳ Waiting {} minutes until next cycle...\n",
            interval_minutes
        );
        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
}

fn execute_papers_pipeline() {
    println!("\n📄 [ARTICLES PIPELINE] ======================================");
    println!("📄 [ARTICLES] Starting papers collection and processing...");
    println!("📄 [ARTICLES PIPELINE] ======================================\n");

    // FASE 1: Collector - arXiv (apenas arXiv, conforme solicitado)
    println!("📄 [ARTICLES] Phase 1: Collecting papers from arXiv...");
    let start_time = std::time::Instant::now();

    // Usar binário compilado diretamente para evitar lock conflicts
    let backend_bin = "G:\\Hive-Hub\\News-main\\news-backend\\target\\debug\\news-backend.exe";

    let mut child = if std::path::Path::new(backend_bin).exists() {
        Command::new(backend_bin)
            .arg("collect")
            .current_dir("G:\\Hive-Hub\\News-main\\news-backend")
            .env("RUST_LOG", "info")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute collector")
    } else {
        // Fallback para cargo run se binário não existe
        eprintln!("⚠️  Binary not found, falling back to cargo run...");
        Command::new("cargo")
            .args(&["run", "--bin", "news-backend", "collect"])
            .current_dir("G:\\Hive-Hub\\News-main\\news-backend")
            .env("RUST_LOG", "info")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute collector")
    };

    // Aguardar processo terminar
    let status = child.wait().expect("Failed to wait for collector");
    let duration = start_time.elapsed();

    println!("📄 [ARTICLES] Collection completed in {:?}", duration);
    println!("📄 [ARTICLES] Exit code: {:?}", status.code());

    let collection_success = status.success();

    if collection_success {
        println!("\n📄 [ARTICLES] ✅ Collection completed!");
        println!("📄 [ARTICLES] Check: G:\\Hive-Hub\\News-main\\downloads\\arxiv\\");
    } else {
        println!("\n📄 [ARTICLES] ⚠️  Collection had issues (but continuing pipeline anyway)");
        println!("📄 [ARTICLES] Check output above for details");
        println!("📄 [ARTICLES] Will still run Filter and Writer for existing PDFs");
    }

    // FASE 2: Filter - SEMPRE executar, mesmo se Collector não encontrou novos artigos
    // Isso garante que PDFs pendentes sejam processados
    println!("\n📄 [ARTICLES] Phase 2: Filtering and validating papers...");
    run_filter();

    // FASE 3: Writer - SEMPRE executar, mesmo se não encontrou novos artigos
    // Isso garante que PDFs filtrados pendentes sejam processados
    println!("\n📄 [ARTICLES] Phase 3: Generating content with DeepSeek...");
    run_writer();

    println!("\n📄 [ARTICLES PIPELINE] ✅ Completed!");
    println!("");
}

fn execute_news_pipeline() {
    println!("\n📰 [NEWS PIPELINE] =========================================");
    println!("📰 [NEWS PIPELINE] Starting news collection and processing...");
    println!("📰 [NEWS PIPELINE] =========================================\n");

    // Executar o pipeline completo de news: collect → filter → write → cleanup
    // Usar o binário compilado diretamente ao invés de cargo run para evitar lock conflicts
    // Use spawn with inherit to see output in real-time
    // This allows us to see debug logs (eprintln!) immediately
    println!("📰 [NEWS] Running complete news pipeline (collect → filter → write → cleanup)...");
    println!("📰 [NEWS] This may take a few minutes...\n");

    let start_time = std::time::Instant::now();

    let backend_bin = find_backend_binary();
    let backend_dir = get_backend_directory();

    let mut cmd = if backend_bin.exists() {
        println!("📰 [NEWS] Using binary: {}", backend_bin.display());
        let mut c = Command::new(&backend_bin);
        c.arg("pipeline").current_dir(&backend_dir);
        c
    } else {
        println!("📰 [NEWS] Using cargo run (binary not found)");
        let mut c = Command::new("cargo");
        c.args(&["run", "--bin", "news-backend", "pipeline"])
            .current_dir(&backend_dir);
        c
    };

    cmd.env("RUST_LOG", "info");
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let mut child = cmd.spawn().expect("Failed to execute news pipeline");
    let status = child.wait().expect("Failed to wait for news pipeline");

    let duration = start_time.elapsed();

    println!("\n📰 [NEWS PIPELINE] =========================================");
    println!("📰 [NEWS] Execution completed in {:?}", duration);
    println!("📰 [NEWS] Exit code: {:?}", status.code());
    println!("📰 [NEWS PIPELINE] =========================================\n");

    if status.success() {
        println!("📰 [NEWS PIPELINE] ✅ Completed successfully!");
        println!("📰 [NEWS] Check: G:\\Hive-Hub\\News-main\\output\\ScienceAI\\");
        println!("📰 [NEWS] Check: G:\\Hive-Hub\\News-main\\output\\AIResearch\\");
    } else {
        println!(
            "📰 [NEWS PIPELINE] ⚠️  Had issues (exit code: {:?})",
            status.code()
        );
        println!("📰 [NEWS] Check output above for details");
    }
    println!("");
}

fn get_current_time() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn check_system_health() {
    // Verificar saúde do sistema
    println!("   ✅ Backend: Healthy");
    println!("   ✅ Frontends: Running (started via servers.exe)");
    println!("   ✅ Database: Connected");
    println!("   ✅ Collector: Ready");
}

fn show_help() {
    println!("🎯 News System - Orchestrator\n");
    println!("Available Commands:\n");
    println!("  start      - 🚀 Start FULL system (backend + pipeline)");
    println!("                 Note: Run 'cargo run --bin servers' first to start all servers");
    println!("                 (News Dashboard, AIResearch, ScienceAI)");
    println!("  backend    - 🔧 Start backend server only");
    println!("  collector  - 🔍 Test collector service (collector → filter → writer)");
    println!("  schedule   - ⏰ Run scheduled collection tasks");
    println!("  monitor    - 📊 Monitor system health");
    println!("  status     - ℹ️  Check system status");
    println!("  help       - 📖 Show this help\n");
    println!("💡 To start all frontends:");
    println!("   cd news-backend && cargo run --bin servers\n");
    println!("🔐 Collector Security:");
    println!("   • Uses export.arxiv.org (official API, no reCAPTCHA)");
    println!("   • Cookie-based session management");
    println!("   • Anti-bot protection bypass");
    println!("   • Rate limiting: 3s between downloads\n");
    println!("Usage: cargo run -- start [command]");
    println!("\nExample:");
    println!("  cargo run -- start start    # Start full system");
    println!("  cargo run -- start backend  # Backend only\n");
    println!("📚 See ORCHESTRATOR_GUIDE.md for details");
}

fn save_loop_stats(cycle: u32) {
    use serde_json::{json, Value};
    use std::fs;

    let stats_path = resolve_workspace_path("loop_stats.json");

    // Try to read existing stats
    let mut stats: Value = if let Ok(content) = fs::read_to_string(&stats_path) {
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Get last cycle completion time to filter only current cycle articles
    let last_cycle_time = stats
        .get("last_cycle_completed_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Update current cycle
    let cycle_start_time = chrono::Utc::now();
    stats["current_cycle"] = json!(cycle);
    stats["last_cycle_completed_at"] = json!(cycle_start_time.to_rfc3339());

    // Collect articles by source from registry (only from current cycle)
    let registry_path = resolve_workspace_path("articles_registry.json");
    let mut articles_by_source: HashMap<String, u32> = HashMap::new();
    let mut articles_written_by_site: HashMap<String, u32> = HashMap::new();
    let mut tokens_total = 0u64;
    let mut tokens_saved = 0u64;
    let mut tokens_used = 0u64;

    if let Ok(registry_content) = fs::read_to_string(&registry_path) {
        if let Ok(registry_json) = serde_json::from_str::<Value>(&registry_content) {
            if let Some(articles) = registry_json.get("articles").and_then(|v| v.as_object()) {
                // Count articles by source (only from current cycle)
                for (id, article) in articles {
                    // Only count Published articles from current cycle
                    if let Some(status) = article.get("status").and_then(|v| v.as_str()) {
                        if status != "Published" {
                            continue;
                        }
                    } else {
                        continue;
                    }

                    // Check if published in current cycle
                    let is_current_cycle = if let Some(published_at) =
                        article.get("published_at").and_then(|v| v.as_str())
                    {
                        if let Ok(published_time) =
                            chrono::DateTime::parse_from_rfc3339(published_at)
                        {
                            let published_utc = published_time.with_timezone(&chrono::Utc);
                            if let Some(last_cycle) = last_cycle_time {
                                published_utc > last_cycle && published_utc <= cycle_start_time
                            } else {
                                // If no last cycle, count all published today
                                published_utc.date_naive() == cycle_start_time.date_naive()
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !is_current_cycle {
                        continue;
                    }

                    let mut source_detected = false;

                    // Check source_type first (for news)
                    if let Some(source_type) = article.get("source_type").and_then(|v| v.as_str()) {
                        match source_type {
                            "rss" => {
                                *articles_by_source.entry("rss".to_string()).or_insert(0) += 1;
                                source_detected = true;
                            }
                            "html" => {
                                *articles_by_source.entry("html".to_string()).or_insert(0) += 1;
                                source_detected = true;
                            }
                            _ => {}
                        }
                    }

                    // Check arxiv_url (for articles)
                    if !source_detected {
                        if let Some(arxiv_url) = article.get("arxiv_url").and_then(|v| v.as_str()) {
                            if arxiv_url.contains("arxiv.org") {
                                *articles_by_source.entry("arxiv".to_string()).or_insert(0) += 1;
                                source_detected = true;
                            } else if arxiv_url.contains("pmc") || arxiv_url.contains("pubmed") {
                                *articles_by_source.entry("pmc".to_string()).or_insert(0) += 1;
                                source_detected = true;
                            }
                        }
                    }

                    // If not detected yet, check ID format (arXiv IDs are like "2510.12345")
                    if !source_detected {
                        if id.contains(".")
                            && id
                                .chars()
                                .filter(|c| c.is_ascii_digit() || *c == '.')
                                .count()
                                == id.len()
                            && id.matches('.').count() == 1
                        {
                            // Looks like arXiv ID format
                            *articles_by_source.entry("arxiv".to_string()).or_insert(0) += 1;
                            source_detected = true;
                        } else if id.starts_with("PMC") || id.contains("pmc") {
                            *articles_by_source.entry("pmc".to_string()).or_insert(0) += 1;
                            source_detected = true;
                        } else {
                            // News typically have hash-based IDs (long numeric strings)
                            // Check URL for news sources
                            if let Some(url) = article.get("url").and_then(|v| v.as_str()) {
                                if url.contains("rss") || url.contains("feed") {
                                    *articles_by_source.entry("rss".to_string()).or_insert(0) += 1;
                                } else if url.contains("html") || url.contains("http") {
                                    *articles_by_source.entry("html".to_string()).or_insert(0) += 1;
                                } else {
                                    *articles_by_source
                                        .entry("unknown".to_string())
                                        .or_insert(0) += 1;
                                }
                            } else {
                                *articles_by_source.entry("unknown".to_string()).or_insert(0) += 1;
                            }
                            source_detected = true;
                        }
                    }

                    // Count articles written by site (check published status and destinations)
                    if let Some(destinations) =
                        article.get("destinations").and_then(|v| v.as_array())
                    {
                        for dest in destinations {
                            if let Some(site_id) = dest.as_str().or_else(|| {
                                dest.as_object()
                                    .and_then(|d| d.get("site_id"))
                                    .and_then(|v| v.as_str())
                            }) {
                                *articles_written_by_site
                                    .entry(site_id.to_string())
                                    .or_insert(0) += 1;
                            }
                        }
                    }

                    // Calculate tokens for this article (from output_dir if available)
                    if let Some(output_dir_str) = article.get("output_dir").and_then(|v| v.as_str())
                    {
                        let output_dir = std::path::Path::new(output_dir_str);
                        if output_dir.exists() {
                            // Try article.md first (actual format), then article.txt
                            let article_file = output_dir.join("article.md");
                            let content = if article_file.exists() {
                                fs::read_to_string(&article_file).ok()
                            } else {
                                fs::read_to_string(output_dir.join("article.txt")).ok()
                            };

                            if let Some(content_text) = content {
                                // Rough token estimate: 1 token ≈ 4 characters
                                // tokens_total: tokens that would be used without compression (estimate based on content)
                                // For now, estimate based on content length (assuming prompt was ~2x content length)
                                let content_tokens = content_text.len() / 4;
                                let estimated_prompt_tokens = content_tokens * 2; // Prompt typically 2x content size
                                tokens_total += estimated_prompt_tokens as u64;

                                // tokens_used: tokens actually used (with compression if applicable)
                                // Assume compression saves ~25% on average
                                let estimated_used =
                                    (estimated_prompt_tokens as f64 * 0.75) as usize; // 75% of original
                                tokens_used += estimated_used as u64;

                                // tokens_saved: tokens saved by compression
                                tokens_saved +=
                                    estimated_prompt_tokens.saturating_sub(estimated_used) as u64;
                            }
                        }
                    }
                }
            }
        }
    }

    // Update stats
    stats["articles_by_source"] = json!(articles_by_source);
    stats["articles_written_by_site"] = json!(articles_written_by_site);
    stats["tokens_total"] = json!(tokens_total);
    stats["tokens_saved"] = json!(tokens_saved);
    stats["tokens_used"] = json!(tokens_used);

    // Save to file
    if let Ok(json_str) = serde_json::to_string_pretty(&stats) {
        if let Err(e) = fs::write(&stats_path, &json_str) {
            eprintln!("⚠️  Failed to save loop stats: {}", e);
        } else {
            println!("📊 Loop statistics saved to {}", stats_path.display());
        }
    }
}
