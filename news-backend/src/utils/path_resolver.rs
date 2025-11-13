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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn resolve_workspace_path_uses_news_base_dir() {
        let original_news_base = env::var("NEWS_BASE_DIR").ok();
        let temp_dir = tempdir().expect("failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        unsafe {
            env::set_var("NEWS_BASE_DIR", &temp_path);
        }

        let workspace = workspace_root();
        assert_eq!(workspace, temp_path);

        let resolved_downloads = resolve_workspace_path(Path::new("downloads").join("raw"));
        assert_eq!(resolved_downloads, temp_path.join("downloads").join("raw"));

        if let Some(value) = original_news_base {
            unsafe {
                env::set_var("NEWS_BASE_DIR", value);
            }
        } else {
            unsafe {
                env::remove_var("NEWS_BASE_DIR");
            }
        }
    }
}
