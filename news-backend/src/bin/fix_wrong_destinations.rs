// Fix articles saved in wrong output directories
// This script checks articles in the registry and moves them to the correct
// output directory based on their destinations field
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

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

#[tokio::main]
async fn main() -> Result<()> {
    let registry_path = resolve_workspace_path("articles_registry.json");

    println!("🔧 Fixing articles in wrong output directories...\n");

    // Load registry
    let registry_content = fs::read_to_string(&registry_path)
        .await
        .context("Failed to read registry file")?;

    let mut registry: serde_json::Value =
        serde_json::from_str(&registry_content).context("Failed to parse registry JSON")?;

    let articles = registry["articles"]
        .as_object_mut()
        .context("Articles not found in registry")?;

    let mut fixed_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;

    println!("📋 Checking {} articles...\n", articles.len());

    for (article_id, article_data) in articles.iter_mut() {
        // Get destinations
        let destinations = article_data["destinations"].as_array().and_then(|d| {
            d.iter()
                .map(|s| s.as_str().map(|s| s.to_string()))
                .collect::<Option<Vec<_>>>()
        });

        let destinations = match destinations {
            Some(d) if !d.is_empty() => d,
            _ => {
                skipped_count += 1;
                continue;
            }
        };

        // Get current output_dir
        let current_output_dir = article_data["output_dir"].as_str().map(PathBuf::from);

        if current_output_dir.is_none() {
            skipped_count += 1;
            continue;
        }

        let current_output_dir = current_output_dir.unwrap();

        // Determine correct output_dir based on first destination
        let first_destination = &destinations[0];
        let correct_site_name = match first_destination.to_lowercase().as_str() {
            "airesearch" => "AIResearch",
            "scienceai" => "ScienceAI",
            _ => first_destination,
        };

        let correct_output_dir = resolve_workspace_path("output")
            .join(correct_site_name)
            .join(article_id);

        // Check if article is in wrong directory
        if current_output_dir != correct_output_dir {
            println!("🔍 Article: {}", article_id);
            println!("   📍 Current:  {}", current_output_dir.display());
            println!("   ✅ Expected: {}", correct_output_dir.display());
            println!("   🎯 Destinations: {:?}", destinations);

            // Check if source directory exists
            if current_output_dir.exists() {
                // Check if target directory already exists
                if correct_output_dir.exists() {
                    println!("   ⚠️  Target directory already exists! Skipping...\n");
                    error_count += 1;
                    continue;
                }

                // Create target directory
                if let Some(parent) = correct_output_dir.parent() {
                    fs::create_dir_all(parent)
                        .await
                        .context("Failed to create target directory")?;
                }

                // Move directory
                println!("   📦 Moving directory...");
                fs::rename(&current_output_dir, &correct_output_dir)
                    .await
                    .context("Failed to move directory")?;

                // Update registry
                article_data["output_dir"] = serde_json::Value::String(
                    correct_output_dir.to_string_lossy().replace("\\", "\\"),
                );

                println!("   ✅ Moved successfully!\n");
                fixed_count += 1;
            } else {
                println!("   ⚠️  Source directory does not exist! Updating registry only...\n");
                // Update registry anyway
                article_data["output_dir"] = serde_json::Value::String(
                    correct_output_dir.to_string_lossy().replace("\\", "\\"),
                );
                fixed_count += 1;
            }
        } else {
            skipped_count += 1;
        }
    }

    // Save updated registry
    if fixed_count > 0 || error_count > 0 {
        let updated_json = serde_json::to_string_pretty(&registry)
            .context("Failed to serialize updated registry")?;

        // Create backup
        let backup_path = format!(
            "{}.backup.{}",
            registry_path.display(),
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        fs::copy(&registry_path, &backup_path)
            .await
            .context("Failed to create backup")?;
        println!("💾 Backup created: {}\n", backup_path);

        // Save updated registry
        fs::write(&registry_path, updated_json)
            .await
            .context("Failed to save updated registry")?;
        println!("💾 Registry updated\n");
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("✅ Fix Complete!");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 Statistics:");
    println!("   ✅ Fixed:    {} articles", fixed_count);
    println!("   ⚠️  Errors:   {} articles", error_count);
    println!("   ⏭️  Skipped:  {} articles", skipped_count);
    println!("═══════════════════════════════════════════════════════════════════\n");

    Ok(())
}
