// Sync .env file from system_config.json
// Usage: cargo run --bin sync-env-from-config

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde_json::Value;

fn main() -> Result<()> {
    let config_path = Path::new("system_config.json");
    let env_path = Path::new(".env");

    println!("🔄 Sincronizando .env a partir de system_config.json...");
    println!("   Config: {}", config_path.display());
    println!("   Env: {}", env_path.display());
    println!();

    // Ler system_config.json
    if !config_path.exists() {
        anyhow::bail!("❌ system_config.json não encontrado em: {}", config_path.display());
    }

    let config_content = fs::read_to_string(config_path)
        .context("Failed to read system_config.json")?;
    
    let config: Value = serde_json::from_str(&config_content)
        .context("Failed to parse system_config.json")?;

    // Ler .env existente
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
                // Remover BOM se presente
                let key = key.strip_prefix('\u{FEFF}').unwrap_or(&key).to_string();
                let value = value.trim().to_string();
                // Não sobrescrever se já existe (manter primeira ocorrência)
                env_vars.entry(key).or_insert(value);
            }
        }
        println!("✅ .env existente encontrado ({} variáveis)", env_vars.len());
    } else {
        println!("⚠️  .env não existe, será criado");
    }

    // Extrair API keys dos sites
    let sites = config.get("sites")
        .and_then(|v| v.as_object())
        .context("Missing 'sites' in config")?;

    let mut updated = 0;

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

                    // Atualizar ou adicionar a chave
                    if env_vars.insert(env_key.to_string(), api_key.to_string()).is_some() {
                        println!("  ✅ {} atualizada do site: {}", env_key, site_id);
                    } else {
                        println!("  ✅ {} adicionada do site: {}", env_key, site_id);
                    }
                    updated += 1;
                }
            }
        }
    }

    // Escrever .env atualizado
    let mut env_content = String::new();
    let mut sorted_keys: Vec<&String> = env_vars.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        env_content.push_str(&format!("{}={}\n", key, env_vars[key]));
    }
    
    // Remover última newline
    env_content = env_content.trim_end().to_string();

    fs::write(env_path, env_content)
        .context("Failed to write .env file")?;

    println!();
    println!("✅ .env atualizado com sucesso!");
    println!("   Total de variáveis: {}", env_vars.len());
    println!("   Variáveis atualizadas: {}", updated);
    println!();
    println!("📄 Conteúdo do .env:");
    let final_content = fs::read_to_string(env_path)?;
    for line in final_content.lines() {
        println!("   {}", line);
    }

    Ok(())
}

