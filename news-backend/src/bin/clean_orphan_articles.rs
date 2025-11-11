use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArticleMetadata {
    pub id: String,
    pub title: String,
    pub output_dir: Option<PathBuf>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct ArticleRegistry {
    articles: HashMap<String, ArticleMetadata>,
}

fn main() -> Result<()> {
    println!("🧹 Cleaning Orphan Articles from Registry");
    println!("==========================================\n");

    // Load registry
    let registry_path = resolve_workspace_path("articles_registry.json");
    println!("📄 Loading registry from: {}", registry_path.display());
    
    let registry_content = fs::read_to_string(&registry_path)?;
    let mut registry: ArticleRegistry = serde_json::from_str(&registry_content)?;
    
    let total_count = registry.articles.len();
    println!("📊 Total articles in registry: {}\n", total_count);

    // Check each article for orphan status
    let mut orphan_ids = Vec::new();
    let mut valid = 0;
    let mut missing_output_dir = 0;

    println!("🔍 Scanning for orphan articles...\n");

    for (id, article) in registry.articles.iter() {
        if let Some(output_dir) = &article.output_dir {
            // Resolve path relative to workspace
            let full_path = resolve_workspace_path(output_dir);
            
            // Check if directory exists and has required files
            let is_valid = if full_path.exists() && full_path.is_dir() {
                let title_txt = full_path.join("title.txt");
                let article_md = full_path.join("article.md");
                title_txt.exists() && article_md.exists()
            } else {
                false
            };

            if is_valid {
                valid += 1;
            } else {
                orphan_ids.push(id.clone());
                
                // Show first 10 orphans as examples
                if orphan_ids.len() <= 10 {
                    println!("  ❌ {} → {}", article.id, full_path.display());
                    if !full_path.exists() {
                        println!("     (folder does not exist)");
                    } else {
                        println!("     (missing title.txt or article.md)");
                    }
                }
            }
        } else {
            missing_output_dir += 1;
            orphan_ids.push(id.clone());
            
            if missing_output_dir <= 5 {
                println!("  ⚠️  {} → (no output_dir field)", article.id);
            }
        }
    }

    if orphan_ids.len() > 10 {
        println!("  ... and {} more orphans", orphan_ids.len() - 10);
    }

    println!("\n📈 Summary:");
    println!("  ✅ Valid articles: {}", valid);
    println!("  ❌ Orphan articles: {}", orphan_ids.len());
    println!("  ⚠️  Missing output_dir: {}", missing_output_dir);

    if orphan_ids.is_empty() {
        println!("\n✨ No orphans found! Registry is clean.");
        return Ok(());
    }

    // Ask for confirmation
    println!("\n⚠️  WARNING: This will remove {} articles from the registry!", orphan_ids.len());
    println!("   The original registry will be backed up.");
    print!("\n   Continue? [y/N]: ");
    
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("\n❌ Aborted. No changes made.");
        return Ok(());
    }

    // Backup registry
    let backup_path = format!("{}.backup.orphan_cleanup", registry_path.display());
    println!("\n💾 Creating backup: {}", backup_path);
    fs::copy(&registry_path, &backup_path)?;

    // Remove orphans from HashMap
    println!("🧹 Removing orphans from registry...");
    
    for id in &orphan_ids {
        registry.articles.remove(id);
    }

    // Save cleaned registry
    let cleaned_json = serde_json::to_string_pretty(&registry)?;
    fs::write(&registry_path, cleaned_json)?;

    println!("\n✅ Cleanup complete!");
    println!("   Removed: {} orphan articles", orphan_ids.len());
    println!("   Remaining: {} valid articles", valid);
    println!("   Backup saved to: {}", backup_path);

    Ok(())
}
