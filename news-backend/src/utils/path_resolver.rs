use std::path::{Path, PathBuf};

/// Returns the workspace root directory.
///
/// Preference order:
/// 1. `NEWS_BASE_DIR` environment variable (if set and not empty)
/// 2. Current working directory (fallback)
pub fn workspace_root() -> PathBuf {
    if let Ok(env_path) = std::env::var("NEWS_BASE_DIR") {
        let trimmed = env_path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Resolves a path relative to the workspace root.
pub fn resolve_workspace_path<P: AsRef<Path>>(relative: P) -> PathBuf {
    workspace_root().join(relative.as_ref())
}

