use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, bail};
use lazy_static::lazy_static;
use serde_json::json;
use tokio::{
    process::Command,
    sync::{Mutex, Notify, RwLock},
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, error, info, warn};

/// Configuration for the background pipeline loop.
#[derive(Clone, Debug)]
pub struct LoopConfig {
    pub interval_minutes: u64,
    pub filter_score_min: f64,
    pub max_cycles: Option<u32>,
}

struct LoopHandle {
    stop: Arc<AtomicBool>,
    notify: Arc<Notify>,
    join: JoinHandle<()>,
    config: Arc<RwLock<LoopConfig>>,
}

struct LoopManagerInner {
    state: Mutex<Option<LoopHandle>>,
}

impl LoopManagerInner {
    fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }
}

#[derive(Clone)]
pub struct LoopManager {
    inner: Arc<LoopManagerInner>,
}

lazy_static! {
    static ref LOOP_MANAGER: LoopManager = LoopManager {
        inner: Arc::new(LoopManagerInner::new())
    };
}

impl LoopManager {
    pub fn global() -> &'static LoopManager {
        &LOOP_MANAGER
    }

    /// Starts or updates the loop. Returns `Ok(true)` if a new loop was started,
    /// and `Ok(false)` if an existing loop was updated with new configuration.
    pub async fn start(&self, config: LoopConfig) -> Result<bool> {
        // Clean up finished task if necessary
        {
            let mut guard = self.inner.state.lock().await;
            let finished_handle = if guard
                .as_ref()
                .is_some_and(|handle| handle.join.is_finished())
            {
                guard.take()
            } else {
                None
            };
            drop(guard);
            if let Some(handle) = finished_handle
                && let Err(e) = handle.join.await
            {
                warn!("Pipeline loop task terminated with error: {:?}", e);
            }
        }

        let mut guard = self.inner.state.lock().await;

        if let Some(handle) = guard.as_ref() {
            // Update configuration only
            {
                let mut current = handle.config.write().await;
                *current = config;
            }
            info!("Updated pipeline loop configuration");
            return Ok(false);
        }

        let config_arc = Arc::new(RwLock::new(config));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());
        let config_clone = Arc::clone(&config_arc);
        let inner_clone = Arc::clone(&self.inner);
        let stop_clone = Arc::clone(&stop_flag);
        let notify_clone = Arc::clone(&notify);

        let join = tokio::spawn(async move {
            run_loop(inner_clone, config_clone, stop_clone, notify_clone).await;
        });

        *guard = Some(LoopHandle {
            stop: stop_flag,
            notify,
            join,
            config: config_arc,
        });

        info!("Pipeline loop started");
        Ok(true)
    }

    /// Stops the loop if running. Returns true if a loop was running.
    pub async fn stop(&self) -> Result<bool> {
        let handle = {
            let mut guard = self.inner.state.lock().await;
            guard.take()
        };

        if let Some(handle) = handle {
            handle.stop.store(true, Ordering::SeqCst);
            handle.notify.notify_waiters();
            if let Err(e) = handle.join.await {
                warn!("Pipeline loop task terminated with error: {:?}", e);
            }
            info!("Pipeline loop stopped");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if the loop is currently running.
    pub async fn is_running(&self) -> bool {
        let guard = self.inner.state.lock().await;
        guard
            .as_ref()
            .map(|handle| !handle.join.is_finished())
            .unwrap_or(false)
    }
}

async fn run_loop(
    manager_inner: Arc<LoopManagerInner>,
    config: Arc<RwLock<LoopConfig>>,
    stop: Arc<AtomicBool>,
    notify: Arc<Notify>,
) {
    info!("Pipeline loop task initialized");

    // Small grace period to allow services to settle
    let initial_delay = sleep(Duration::from_secs(5));
    tokio::pin!(initial_delay);
    tokio::select! {
        _ = initial_delay.as_mut() => {}
        _ = notify.notified() => {
            if stop.load(Ordering::Relaxed) {
                info!("Pipeline loop cancelled before first iteration");
                cleanup_handle(manager_inner, &config).await;
                return;
            }
        }
    }

    if stop.load(Ordering::Relaxed) {
        info!("Pipeline loop cancelled before first iteration");
        cleanup_handle(manager_inner, &config).await;
        return;
    }

    let mut cycle: u32 = 1;
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let current_config = config.read().await.clone();
        debug!(
            interval_minutes = current_config.interval_minutes,
            filter_score_min = current_config.filter_score_min,
            max_cycles = ?current_config.max_cycles,
            "Pipeline loop configuration loaded"
        );
        if let Some(max) = current_config.max_cycles
            && cycle > max
        {
            info!("Pipeline loop reached configured max cycles ({})", max);
            break;
        }

        info!("Starting pipeline cycle {}", cycle);

        if let Err(e) = run_articles_pipeline(&stop).await {
            error!("Articles pipeline failed: {:?}", e);
        }

        if stop.load(Ordering::Relaxed) {
            break;
        }

        if let Err(e) = run_news_pipeline(&stop).await {
            error!("News pipeline failed: {:?}", e);
        }

        if stop.load(Ordering::Relaxed) {
            break;
        }

        if let Err(e) = save_loop_stats(cycle) {
            warn!("Failed to save loop statistics: {:?}", e);
        }

        cycle = cycle.saturating_add(1);

        if let Some(max) = current_config.max_cycles
            && cycle > max
        {
            info!("Pipeline loop reached configured max cycles ({})", max);
            break;
        }

        let wait_seconds = current_config.interval_minutes.saturating_mul(60);
        info!(
            "Waiting {} minutes before next cycle",
            current_config.interval_minutes
        );
        let wait_sleep = sleep(Duration::from_secs(wait_seconds));
        tokio::pin!(wait_sleep);
        tokio::select! {
            _ = wait_sleep.as_mut() => {}
            _ = notify.notified() => {
                if stop.load(Ordering::Relaxed) {
                    info!("Pipeline loop cancellation requested during cooldown");
                    break;
                }
            }
        }
    }

    info!("Pipeline loop task finished");
    cleanup_handle(manager_inner, &config).await;
}

async fn cleanup_handle(manager_inner: Arc<LoopManagerInner>, config: &Arc<RwLock<LoopConfig>>) {
    let mut guard = manager_inner.state.lock().await;
    if guard
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(&current.config, config))
    {
        guard.take();
    }
}

async fn run_articles_pipeline(stop: &Arc<AtomicBool>) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }

    info!("Running articles pipeline (collect → filter → write)");

    run_backend_command("collect", &["collect"], stop).await?;
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }

    run_backend_command("filter", &["filter"], stop).await?;
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }

    run_backend_command("write", &["write"], stop).await?;
    Ok(())
}

async fn run_news_pipeline(stop: &Arc<AtomicBool>) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }

    info!("Running news pipeline");
    run_backend_command("news pipeline", &["pipeline"], stop).await
}

async fn run_backend_command(label: &str, args: &[&str], stop: &Arc<AtomicBool>) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        info!("Pipeline loop cancelled before first iteration");
        return Ok(());
    }

    let exe = std::env::current_exe().context("Unable to determine backend executable path")?;
    let workspace = workspace_root();

    info!(
        "Executing backend command '{}' with args {:?} (cwd: {})",
        label,
        args,
        workspace.display()
    );

    let mut command = Command::new(&exe);
    command
        .args(args)
        .current_dir(&workspace)
        .env(
            "RUST_LOG",
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let status = command
        .status()
        .await
        .with_context(|| format!("Failed to execute backend command '{}'", label))?;

    if !status.success() {
        bail!(
            "Backend command '{}' exited with status code {:?}",
            label,
            status.code()
        );
    }

    Ok(())
}

fn save_loop_stats(cycle: u32) -> Result<()> {
    let stats_path = workspace_root().join("loop_stats.json");
    let now = chrono::Utc::now();

    let stats = json!({
        "success": true,
        "current_cycle": cycle,
        "last_cycle_completed_at": now.to_rfc3339(),
        "articles_by_source": {},
        "articles_written_by_site": {},
        "tokens_total": 0,
        "tokens_saved": 0,
        "tokens_used": 0,
    });

    std::fs::write(&stats_path, serde_json::to_string_pretty(&stats)?)
        .with_context(|| format!("Failed to write loop stats to {}", stats_path.display()))?;

    Ok(())
}

fn workspace_root() -> PathBuf {
    if let Ok(dir) = std::env::var("NEWS_BASE_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return path;
        }
    }

    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if current.join("system_config.json").exists() {
            return current;
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    PathBuf::from(".")
}
