// Script para renomear pastas de artigos para o novo padrão DATA_SOURCE_ID
// Executa: cargo run --bin rename-article-folders

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use regex::Regex;
use serde_json;

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct ArticleMetadata {
    id: String,
    title: String,
    url: String,
}

fn main() -> Result<()> {
    println!("🔄 Renomeando pastas de artigos para novo padrão...");
    println!("================================================\n");

    let output_base = Path::new("G:/Hive-Hub/News-main/output");
    let sites = vec!["ScienceAI", "AIResearch"];

    let mut total_renamed = 0;
    let mut total_skipped = 0;
    let mut total_errors = 0;

    for site_name in sites {
        let site_dir = output_base.join(site_name);
        
        if !site_dir.exists() {
            println!("⚠️  Diretório não encontrado: {}", site_dir.display());
            continue;
        }

        println!("📂 Processando site: {}", site_name);
        println!("   Diretório: {}\n", site_dir.display());

        match process_site_directory(&site_dir)? {
            (renamed, skipped, errors) => {
                total_renamed += renamed;
                total_skipped += skipped;
                total_errors += errors;
                println!("   ✅ Renomeados: {}", renamed);
                println!("   ⏭️  Ignorados: {}", skipped);
                if errors > 0 {
                    println!("   ❌ Erros: {}\n", errors);
                } else {
                    println!();
                }
            }
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Processo concluído!");
    println!("   📊 Total renomeados: {}", total_renamed);
    println!("   📊 Total ignorados: {}", total_skipped);
    if total_errors > 0 {
        println!("   ⚠️  Total erros: {}", total_errors);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

fn process_site_directory(site_dir: &Path) -> Result<(usize, usize, usize)> {
    let mut renamed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    let entries = fs::read_dir(site_dir)
        .context("Failed to read site directory")?;

    // Primeiro, coletar todas as pastas e detectar formato
    let mut folders: Vec<(PathBuf, String, String, String, String)> = Vec::new();

    for entry in entries {
        let entry = entry?;
        let article_dir = entry.path();
        
        if !article_dir.is_dir() {
            continue;
        }

        let folder_name = article_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // NÃO fazer verificação prematura - processar todas as pastas
        // A verificação se está no formato correto será feita depois,
        // quando tivermos todas as informações (source, date, id)

        // Ler title.txt para obter o título
        let title_file = article_dir.join("title.txt");
        let title = if title_file.exists() {
            fs::read_to_string(&title_file)
                .ok()
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            println!("   ⚠️  title.txt não encontrado em: {}", folder_name);
            skipped += 1;
            continue;
        };

        // Detectar source category do source.txt, JSON raw, ou nome da pasta
        let source = detect_source_from_article(&article_dir);

        // Tentar extrair data da pasta raw ou usar data de modificação
        let collection_date = extract_collection_date(&article_dir, &folder_name)?;

        folders.push((article_dir, title, source, collection_date, folder_name));
    }

    // Agora renomear as pastas - FORÇAR renomeação se não estiver no formato correto
    for (article_dir, _title, source, collection_date, old_folder_name) in folders {
        // Extrair article_id do nome original da pasta (sem o prefixo do source)
        let article_id = extract_article_id(&article_dir, &old_folder_name, &source)?;

        // Criar novo nome: DATA_SOURCE_ID
        // Formato esperado: YYYY-MM-DD_source_id
        // Exemplo: 2025-11-03_huggingface_526b5300bcd37de8
        let new_folder_name = format!("{}_{}_{}", collection_date, source, article_id);
        
        // Debug: mostrar comparação para pastas problemáticas
        if old_folder_name.contains("unknown") || old_folder_name != new_folder_name {
            println!("   🔍 [DEBUG] Renomeando: {} → {}", old_folder_name, new_folder_name);
            println!("       source={}, date={}, id={}", source, collection_date, article_id);
        }
        
        // FORÇAR renomeação se não corresponder exatamente ao formato DATA_source_ID
        // Só pular se o nome for EXATAMENTE igual ao novo formato esperado
        if old_folder_name == new_folder_name {
            skipped += 1;
            continue;
        }

        let new_path = article_dir.parent().unwrap().join(&new_folder_name);

        // Verificar se destino já existe
        if new_path.exists() {
            println!("   ⚠️  Destino já existe: {} → {}", old_folder_name, new_folder_name);
            skipped += 1;
            continue;
        }

        println!("   🔄 Renomeando: {} → {}", old_folder_name, new_folder_name);

        match fs::rename(&article_dir, &new_path) {
            Ok(_) => {
                println!("   ✅ Sucesso!");
                renamed += 1;
            }
            Err(e) => {
                eprintln!("   ❌ Erro ao renomear: {}", e);
                errors += 1;
            }
        }
    }

    Ok((renamed, skipped, errors))
}

fn detect_source_from_article(article_dir: &Path) -> String {
    // Tentar ler source.txt primeiro
    let source_file = article_dir.join("source.txt");
    if source_file.exists() {
        if let Ok(source) = fs::read_to_string(&source_file) {
            let source_trimmed = source.trim().to_string();
            if !source_trimmed.is_empty() && source_trimmed != "unknown" {
                return source_trimmed;
            }
        }
    }

    // Tentar ler URL do JSON raw se existir
    let raw_base = Path::new("G:/Hive-Hub/News-main/downloads/raw");
    if raw_base.exists() {
        // Procurar em todas as pastas de data
        if let Ok(date_entries) = fs::read_dir(raw_base) {
            for date_entry in date_entries {
                if let Ok(date_entry) = date_entry {
                    let date_dir = date_entry.path();
                    if date_dir.is_dir() {
                        // Tentar encontrar JSON com o mesmo ID
                        // O nome da pasta pode conter o ID original
                        let folder_name = article_dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        // Tentar diferentes variações do ID
                        let possible_ids = vec![
                            folder_name.to_string(),
                            folder_name.split('_').last().unwrap_or("").to_string(),
                        ];

                        for possible_id in possible_ids {
                            let json_file = date_dir.join(format!("{}.json", possible_id));
                            if json_file.exists() {
                                if let Ok(json_content) = fs::read_to_string(&json_file) {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_content) {
                                        if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
                                            return detect_source_from_url(url);
                                        }
                                        if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                                            return detect_source_from_url_title("", title);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: detectar do nome da pasta
    let folder_name = article_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    detect_source_from_folder_name(folder_name)
}

fn detect_source_from_folder_name(folder_name: &str) -> String {
    let folder_lower = folder_name.to_lowercase();
    
    let sources = vec![
        "openai", "nvidia", "google", "meta", "anthropic", "alibaba", 
        "deepseek", "x", "mistral", "microsoft", "apple", "berkeley",
        "stanford", "inflection", "stability", "intel", "amd", "cohere",
        "deepmind", "character", "menlo", "science", "airesearch", "huggingface",
        "techcrunch", "perplexity"
    ];

    for source in sources {
        if folder_lower.contains(source) {
            return source.to_string();
        }
    }

    "unknown".to_string()
}

fn detect_source_from_url(url: &str) -> String {
    detect_source_from_url_title(url, "")
}

fn detect_source_from_url_title(url: &str, title: &str) -> String {
    let url_lower = url.to_lowercase();
    let title_lower = title.to_lowercase();
    
    // Mesma lógica simplificada de detect_source_category
    if url_lower.contains("openai.com") || title_lower.contains("openai") {
        return "openai".to_string();
    }
    if url_lower.contains("nvidia.com") || title_lower.contains("nvidia") {
        return "nvidia".to_string();
    }
    if url_lower.contains("google.com") || url_lower.contains("deepmind") || title_lower.contains("google") || title_lower.contains("deepmind") {
        return "google".to_string();
    }
    if url_lower.contains("meta.com") || url_lower.contains("facebook.com") || title_lower.contains("meta") || title_lower.contains("facebook") {
        return "meta".to_string();
    }
    if url_lower.contains("anthropic.com") || url_lower.contains("claude") || title_lower.contains("anthropic") {
        return "anthropic".to_string();
    }
    if url_lower.contains("microsoft.com") || title_lower.contains("microsoft") {
        return "microsoft".to_string();
    }
    if url_lower.contains("apple.com") || url_lower.contains("machinelearning.apple.com") || title_lower.contains("apple") {
        return "apple".to_string();
    }
    if url_lower.contains("bair.berkeley.edu") || title_lower.contains("berkeley") {
        return "berkeley".to_string();
    }
    if url_lower.contains("stanford.edu") || url_lower.contains("hai.stanford") || title_lower.contains("stanford") {
        return "stanford".to_string();
    }
    if url_lower.contains("inflection.ai") || title_lower.contains("inflection") {
        return "inflection".to_string();
    }
    if url_lower.contains("intel.com") || title_lower.contains("intel") {
        return "intel".to_string();
    }
    if url_lower.contains("amd.com") || title_lower.contains("amd") {
        return "amd".to_string();
    }
    if url_lower.contains("cohere.com") || title_lower.contains("cohere") {
        return "cohere".to_string();
    }
    if url_lower.contains("character.ai") || title_lower.contains("character") {
        return "character".to_string();
    }
    if url_lower.contains("science.org") || title_lower.contains("science") {
        return "science".to_string();
    }
    if url_lower.contains("huggingface.co") || url_lower.contains("huggingface.com") || title_lower.contains("huggingface") || title_lower.contains("hugging face") {
        return "huggingface".to_string();
    }
    if url_lower.contains("techcrunch.com") || title_lower.contains("techcrunch") {
        return "techcrunch".to_string();
    }
    if url_lower.contains("perplexity.ai") || title_lower.contains("perplexity") {
        return "perplexity".to_string();
    }

    "unknown".to_string()
}

fn extract_collection_date(article_dir: &Path, folder_name: &str) -> Result<String> {
    // Tentar ler de metadata.json se existir
    let metadata_file = article_dir.join("metadata.json");
    if metadata_file.exists() {
        if let Ok(content) = fs::read_to_string(&metadata_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(date_str) = json.get("collected_at").and_then(|v| v.as_str()) {
                    // Parse ISO date and format as YYYY-MM-DD
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
                        return Ok(dt.format("%Y-%m-%d").to_string());
                    }
                }
            }
        }
    }

    // Tentar extrair do nome da pasta se contém data
    let date_pattern = Regex::new(r"\b(\d{4}-\d{2}-\d{2})\b").ok();
    if let Some(re) = date_pattern {
        if let Some(captures) = re.captures(folder_name) {
            if let Some(date_match) = captures.get(1) {
                return Ok(date_match.as_str().to_string());
            }
        }
    }

    // Usar data de modificação do arquivo article.md
    let article_file = article_dir.join("article.md");
    if article_file.exists() {
        if let Ok(metadata) = fs::metadata(&article_file) {
            if let Ok(modified) = metadata.modified() {
                let system_time = std::time::SystemTime::from(modified);
                let datetime: chrono::DateTime<Utc> = system_time.into();
                return Ok(datetime.format("%Y-%m-%d").to_string());
            }
        }
    }

    // Fallback: usar data atual
    Ok(Utc::now().format("%Y-%m-%d").to_string())
}

fn extract_article_id(_article_dir: &Path, folder_name: &str, _detected_source: &str) -> Result<String> {
    // Primeiro, tentar encontrar o JSON raw correspondente para obter o ID original
    let raw_base = Path::new("G:/Hive-Hub/News-main/downloads/raw");
    if raw_base.exists() {
        // Procurar em todas as pastas de data
        if let Ok(date_entries) = fs::read_dir(raw_base) {
            for date_entry in date_entries {
                if let Ok(date_entry) = date_entry {
                    let date_dir = date_entry.path();
                    if date_dir.is_dir() {
                        // Tentar diferentes variações do ID para encontrar o JSON
                        // Extrair possíveis IDs do nome da pasta
                        let parts: Vec<&str> = folder_name.split('_').collect();
                        
                        // Tentar IDs possíveis:
                        // 1. Último segmento (hash numérico/alphanumérico)
                        // 2. Últimos 2 segmentos (título_hash)
                        // 3. Últimos 3 segmentos
                        let possible_ids = if parts.len() > 0 {
                            let mut ids = vec![
                                parts.last().unwrap().to_string(), // Último segmento
                            ];
                            
                            if parts.len() >= 2 {
                                ids.push(parts[parts.len() - 2..].join("_"));
                            }
                            if parts.len() >= 3 {
                                ids.push(parts[parts.len() - 3..].join("_"));
                            }
                            
                            ids
                        } else {
                            vec![folder_name.to_string()]
                        };

                        for possible_id in possible_ids {
                            let json_file = date_dir.join(format!("{}.json", possible_id));
                            if json_file.exists() {
                                if let Ok(json_content) = fs::read_to_string(&json_file) {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_content) {
                                        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                            // Usar o ID do JSON (é o ID original correto)
                                            return Ok(id.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Se não encontrar JSON, extrair ID do nome da pasta
    // Estratégia: remover data e source, pegar o resto
    let parts: Vec<&str> = folder_name.split('_').collect();
    
    // Se começa com data (YYYY-MM-DD), remover a data
    let id_start = if parts.len() >= 1 && parts[0].matches('-').count() == 2 && parts[0].len() == 10 {
        // Formato: YYYY-MM-DD_source_...resto
        // Pular data e source, pegar o resto
        if parts.len() >= 3 {
            parts[2..].join("_")
        } else if parts.len() >= 2 {
            parts[1..].join("_")
        } else {
            folder_name.to_string()
        }
    } else {
        // Sem data no início, pode ser source_...resto ou apenas ...resto
        if parts.len() >= 2 {
            parts[1..].join("_")
        } else {
            folder_name.to_string()
        }
    };
    
    // Remover sources conhecidos do início do ID
    let sources = vec![
        "openai", "nvidia", "google", "meta", "anthropic", "alibaba", 
        "deepseek", "x", "mistral", "microsoft", "apple", "berkeley",
        "stanford", "inflection", "stability", "intel", "amd", "cohere",
        "deepmind", "character", "menlo", "science", "airesearch", "huggingface",
        "techcrunch", "perplexity", "unknown"
    ];
    
    let mut id_clean = id_start;
    let mut changed = true;
    while changed {
        changed = false;
        let id_lower = id_clean.to_lowercase();
        
        for source in &sources {
            if id_lower.starts_with(&format!("{}_", source)) {
                id_clean = id_clean[source.len() + 1..].to_string();
                changed = true;
                break;
            }
        }
    }
    
    // Extrair apenas o ID numérico/hash do final
    // Estratégia: o ID final geralmente é um hash alfanumérico longo (16+ caracteres)
    // ou um número. Procurar pelo último segmento que seja um hash válido.
    
    // Se o id_clean ainda contém underscores, pegar apenas o último segmento
    // que seja um hash (alfanumérico longo)
    let final_parts: Vec<&str> = id_clean.split('_').collect();
    
    // Procurar pelo último segmento que seja um hash válido (16+ caracteres alfanuméricos)
    for part in final_parts.iter().rev() {
        // Verificar se é um hash válido (alfanumérico, 16+ caracteres)
        if part.len() >= 16 && part.chars().all(|c| c.is_alphanumeric()) {
            return Ok(part.to_string());
        }
        // Ou se é puramente numérico (ID numérico)
        if part.chars().all(|c| c.is_ascii_digit()) && part.len() >= 8 {
            return Ok(part.to_string());
        }
    }
    
    // Se não encontrar hash, usar o último segmento
    if let Some(last) = final_parts.last() {
        // Limpar o último segmento removendo qualquer prefixo de source
        let mut clean_last = last.to_string();
        for source in &sources {
            let source_with_underscore = format!("{}_", source);
            if clean_last.to_lowercase().starts_with(&source_with_underscore) {
                clean_last = clean_last[source_with_underscore.len()..].to_string();
            }
        }
        Ok(clean_last)
    } else if let Some(last) = parts.last() {
        Ok(last.to_string())
    } else {
        Ok(folder_name.to_string())
    }
}

