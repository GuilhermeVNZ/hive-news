// Utility to sync .env file from system_config.json
// Used after updating writer configurations

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde_json::Value;

/// Sync .env file with API keys from system_config.json
pub fn sync_env_from_config(config_path: &Path) -> Result<()> {
    let env_path = Path::new(".env");

    // Read system_config.json
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "system_config.json not found at: {}",
            config_path.display()
        ));
    }

    let config_content = fs::read_to_string(config_path)
        .context("Failed to read system_config.json")?;
    
    let config: Value = serde_json::from_str(&config_content)
        .context("Failed to parse system_config.json")?;

    // Read existing .env
    let mut env_vars: HashMap<String, String> = HashMap::new();
    
    if env_path.exists() {
        let env_content = fs::read_to_string(env_path)
            .context("Failed to read .env")?;
        
        for line in env_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                // Remove BOM if present
                let key = key.strip_prefix('\u{FEFF}').unwrap_or(&key).to_string();
                let value = value.trim().to_string();
                // Don't overwrite existing (keep first occurrence)
                env_vars.entry(key).or_insert(value);
            }
        }
    }

    // Extract API keys from sites
    let sites = config.get("sites")
        .and_then(|v| v.as_object())
        .context("Missing 'sites' in config")?;

    for (site_id, site_value) in sites {
        if let Some(writer) = site_value.get("writer").and_then(|v| v.as_object()) {
            if let Some(provider) = writer.get("provider").and_then(|v| v.as_str()) {
                if let Some(api_key) = writer.get("api_key")
                    .and_then(|v| v.as_str())
                    .filter(|k| !k.is_empty() && *k != "null") {
                    
                    let env_key = match provider {
                        "deepseek" => "DEEPSEEK_API_KEY",
                        "openai" => "OPENAI_API_KEY",
                        "anthropic" => "ANTHROPIC_API_KEY",
                        _ => continue,
                    };

                    // Update or add the key
                    env_vars.insert(env_key.to_string(), api_key.to_string());
                    println!("   ✅ Updated {} from site: {}", env_key, site_id);
                }
            }
        }
    }

    // Write .env updated
    let mut env_content = String::new();
    let mut sorted_keys: Vec<&String> = env_vars.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        env_content.push_str(&format!("{}={}\n", key, env_vars[key]));
    }
    
    // Remove last newline
    env_content = env_content.trim_end().to_string();

    fs::write(env_path, env_content)
        .context("Failed to write .env file")?;

    println!("   ✅ .env synchronized ({} variables)", env_vars.len());
    Ok(())
}


















































