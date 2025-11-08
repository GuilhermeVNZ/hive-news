// Script para corrigir categorias "unknown" nos artigos já coletados
// Usa a mesma lógica de detect_source_category para identificar e renomear

use anyhow::{Context, Result};
use serde_json;
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

fn main() -> Result<()> {
    let output_base = resolve_workspace_path("output");
    let downloads_base = resolve_workspace_path("downloads/raw");

    println!("🔧 Fixing unknown categories for collected articles...\n");

    // Processar ScienceAI
    let science_ai_dir = output_base.join("ScienceAI");
    if science_ai_dir.exists() {
        println!("📁 Processing ScienceAI articles...");
        fix_articles_in_directory(&science_ai_dir, &downloads_base)?;
    }

    // Processar AIResearch
    let ai_research_dir = output_base.join("AIResearch");
    if ai_research_dir.exists() {
        println!("\n📁 Processing AIResearch articles...");
        fix_articles_in_directory(&ai_research_dir, &downloads_base)?;
    }

    println!("\n✅ Done!");
    Ok(())
}

fn fix_articles_in_directory(site_dir: &Path, downloads_base: &Path) -> Result<()> {
    let mut fixed_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    // Encontrar todas as pastas que começam com data e contêm "unknown" ou "Other"
    let entries = fs::read_dir(site_dir).context("Failed to read site directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read entry")?;
        let article_dir = entry.path();

        if !article_dir.is_dir() {
            continue;
        }

        let dir_name = article_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Verificar se é uma pasta "unknown" ou "Other"
        if !dir_name.contains("unknown") && !dir_name.contains("_Other_") {
            continue;
        }

        // Extrair ID do artigo do nome da pasta
        // Formato: YYYY-MM-DD_unknown_<id> ou YYYY-MM-DD_Other_<id>
        let parts: Vec<&str> = dir_name.split('_').collect();
        if parts.len() < 3 {
            println!("  ⚠️  Skipping invalid folder name: {}", dir_name);
            skipped_count += 1;
            continue;
        }

        let article_id = parts[parts.len() - 1]; // Última parte é o ID
        let date_part = parts[0]; // Primeira parte é a data

        // Tentar encontrar o JSON original
        let json_path = find_json_file(downloads_base, article_id, date_part);

        let (url, title) = if let Some(json_path) = &json_path {
            match read_article_metadata(json_path) {
                Ok((u, t)) => (u, t),
                Err(e) => {
                    println!("  ⚠️  Failed to read JSON for {}: {}", article_id, e);
                    // Tentar ler do source.txt ou title.txt
                    match read_from_output_files(&article_dir) {
                        Ok((u, t)) => (u, t),
                        Err(_) => {
                            println!("  ❌ No metadata found for {}", article_id);
                            error_count += 1;
                            continue;
                        }
                    }
                }
            }
        } else {
            // Tentar ler do source.txt ou title.txt
            match read_from_output_files(&article_dir) {
                Ok((u, t)) => (u, t),
                Err(_) => {
                    println!("  ❌ No metadata found for {}", article_id);
                    error_count += 1;
                    continue;
                }
            }
        };

        // Detectar categoria correta
        let mut correct_category = detect_source_category(&url, &title);

        // Se ainda for "unknown", trocar por "technology"
        if correct_category == "unknown" {
            correct_category = "technology".to_string();
        }

        // Criar novo nome de pasta
        let new_dir_name = format!("{}_{}_{}", date_part, correct_category, article_id);
        let new_article_dir = site_dir.join(&new_dir_name);

        // Verificar se o source.txt atual está como "unknown" ou "Other" e atualizar ANTES de renomear
        let source_file = article_dir.join("source.txt");
        if source_file.exists() {
            let current_source = fs::read_to_string(&source_file)
                .unwrap_or_default()
                .trim()
                .to_string();
            if current_source == "unknown" || current_source == "Other" || current_source.is_empty()
            {
                fs::write(&source_file, &correct_category)
                    .context(format!("Failed to update source.txt in {:?}", article_dir))?;
                println!(
                    "     📝 Updated source.txt: {} → {}",
                    current_source, correct_category
                );
            }
        } else {
            // Criar source.txt se não existir
            fs::write(&source_file, &correct_category)
                .context(format!("Failed to create source.txt in {:?}", article_dir))?;
            println!("     📝 Created source.txt: {}", correct_category);
        }

        // Renomear pasta
        println!("  🔄 Renaming: {} → {}", dir_name, new_dir_name);
        fs::rename(&article_dir, &new_article_dir)
            .context(format!("Failed to rename directory {:?}", article_dir))?;

        // Garantir que source.txt está correto após renomear
        let source_file_after = new_article_dir.join("source.txt");
        fs::write(&source_file_after, &correct_category).context(format!(
            "Failed to update source.txt in {:?}",
            new_article_dir
        ))?;

        println!("     ✅ Updated to category: {}", correct_category);
        fixed_count += 1;
    }

    println!("\n  📊 Results:");
    println!("     ✅ Fixed: {} (renamed to 'technology')", fixed_count);
    println!("     ⏭️  Skipped: {}", skipped_count);
    println!("     ❌ Errors: {}", error_count);

    Ok(())
}

fn find_json_file(downloads_base: &Path, article_id: &str, date: &str) -> Option<PathBuf> {
    // Tentar primeiro na pasta da data
    let date_dir = downloads_base.join(date);
    let json_path = date_dir.join(format!("{}.json", article_id));
    if json_path.exists() {
        return Some(json_path);
    }

    // Se não encontrou, procurar em todas as pastas de data
    if let Ok(entries) = fs::read_dir(downloads_base) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let json_path = path.join(format!("{}.json", article_id));
                    if json_path.exists() {
                        return Some(json_path);
                    }
                }
            }
        }
    }

    None
}

fn read_article_metadata(json_path: &Path) -> Result<(String, String)> {
    let content = fs::read_to_string(json_path).context("Failed to read JSON file")?;

    let article: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse JSON")?;

    let url = article["url"].as_str().unwrap_or("").to_string();

    let title = article["original_title"]
        .as_str()
        .or_else(|| article["title"].as_str())
        .unwrap_or("")
        .to_string();

    Ok((url, title))
}

fn read_from_output_files(article_dir: &Path) -> Result<(String, String)> {
    // Tentar ler URL do source.txt (pode conter URL se foi salvo)
    let source_file = article_dir.join("source.txt");
    let mut url = String::new();

    if let Ok(content) = fs::read_to_string(&source_file) {
        if content.starts_with("http") {
            url = content.trim().to_string();
        }
    }

    // Ler título do title.txt
    let title_file = article_dir.join("title.txt");
    let title = if title_file.exists() {
        fs::read_to_string(&title_file)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        String::new()
    };

    if url.is_empty() && title.is_empty() {
        return Err(anyhow::anyhow!("No metadata found in output files"));
    }

    Ok((url, title))
}

// Copiar a função detect_source_category do news_writer.rs
fn detect_source_category(url: &str, title: &str) -> String {
    let url_lower = url.to_lowercase();
    let title_lower = title.to_lowercase();

    struct CategoryScore {
        name: &'static str,
        score: i32,
    }

    let mut scores = vec![
        CategoryScore {
            name: "openai",
            score: 0,
        },
        CategoryScore {
            name: "nvidia",
            score: 0,
        },
        CategoryScore {
            name: "google",
            score: 0,
        },
        CategoryScore {
            name: "meta",
            score: 0,
        },
        CategoryScore {
            name: "anthropic",
            score: 0,
        },
        CategoryScore {
            name: "alibaba",
            score: 0,
        },
        CategoryScore {
            name: "deepseek",
            score: 0,
        },
        CategoryScore {
            name: "x",
            score: 0,
        },
        CategoryScore {
            name: "mistral",
            score: 0,
        },
        CategoryScore {
            name: "microsoft",
            score: 0,
        },
        CategoryScore {
            name: "apple",
            score: 0,
        },
        CategoryScore {
            name: "berkeley",
            score: 0,
        },
        CategoryScore {
            name: "stanford",
            score: 0,
        },
        CategoryScore {
            name: "inflection",
            score: 0,
        },
        CategoryScore {
            name: "stability",
            score: 0,
        },
        CategoryScore {
            name: "intel",
            score: 0,
        },
        CategoryScore {
            name: "amd",
            score: 0,
        },
        CategoryScore {
            name: "cohere",
            score: 0,
        },
        CategoryScore {
            name: "deepmind",
            score: 0,
        },
        CategoryScore {
            name: "characterai",
            score: 0,
        },
        CategoryScore {
            name: "menlo",
            score: 0,
        },
        CategoryScore {
            name: "science",
            score: 0,
        },
        CategoryScore {
            name: "airesearch",
            score: 0,
        },
        CategoryScore {
            name: "huggingface",
            score: 0,
        },
        CategoryScore {
            name: "techcrunch",
            score: 0,
        },
        CategoryScore {
            name: "perplexity",
            score: 0,
        },
        // Robotics sources
        CategoryScore {
            name: "boston_dynamics",
            score: 0,
        },
        CategoryScore {
            name: "robot_report",
            score: 0,
        },
        CategoryScore {
            name: "robotics_business",
            score: 0,
        },
        CategoryScore {
            name: "robohub",
            score: 0,
        },
        CategoryScore {
            name: "ieee_robotics",
            score: 0,
        },
        CategoryScore {
            name: "robotics_org",
            score: 0,
        },
        CategoryScore {
            name: "abb_robotics",
            score: 0,
        },
        CategoryScore {
            name: "kuka",
            score: 0,
        },
        CategoryScore {
            name: "universal_robots",
            score: 0,
        },
        CategoryScore {
            name: "omron",
            score: 0,
        },
        CategoryScore {
            name: "yaskawa",
            score: 0,
        },
        CategoryScore {
            name: "agility",
            score: 0,
        },
        CategoryScore {
            name: "unitree",
            score: 0,
        },
        // Quantum computing sources
        CategoryScore {
            name: "quantum_computing_report",
            score: 0,
        },
        CategoryScore {
            name: "ibm_quantum",
            score: 0,
        },
        CategoryScore {
            name: "quanta",
            score: 0,
        },
        CategoryScore {
            name: "rigetti",
            score: 0,
        },
        CategoryScore {
            name: "ionq",
            score: 0,
        },
        CategoryScore {
            name: "dwave",
            score: 0,
        },
        CategoryScore {
            name: "quantinuum",
            score: 0,
        },
        CategoryScore {
            name: "pasqal",
            score: 0,
        },
        CategoryScore {
            name: "xanadu",
            score: 0,
        },
        CategoryScore {
            name: "infleqtion",
            score: 0,
        },
        CategoryScore {
            name: "quantum_computing_inc",
            score: 0,
        },
        // AI startups
        CategoryScore {
            name: "adept",
            score: 0,
        },
        CategoryScore {
            name: "assemblyai",
            score: 0,
        },
        CategoryScore {
            name: "replicate",
            score: 0,
        },
        CategoryScore {
            name: "langchain",
            score: 0,
        },
        CategoryScore {
            name: "pinecone",
            score: 0,
        },
        CategoryScore {
            name: "weaviate",
            score: 0,
        },
        CategoryScore {
            name: "together",
            score: 0,
        },
        CategoryScore {
            name: "anyscale",
            score: 0,
        },
        CategoryScore {
            name: "modal",
            score: 0,
        },
        CategoryScore {
            name: "continual",
            score: 0,
        },
        CategoryScore {
            name: "fastai",
            score: 0,
        },
        CategoryScore {
            name: "eleuther",
            score: 0,
        },
    ];

    // Domain-specific matches get highest priority (score 100)
    if url_lower.contains("openai.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "openai")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("nvidia.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "nvidia")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("google.com")
        || url_lower.contains("blog.research.google")
        || url_lower.contains("deepmind.google")
    {
        scores
            .iter_mut()
            .find(|s| s.name == "google")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("about.fb.com")
        || url_lower.contains("facebook.com")
        || url_lower.contains("meta.com")
    {
        scores.iter_mut().find(|s| s.name == "meta").unwrap().score = 100;
    }
    if url_lower.contains("anthropic.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "anthropic")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("alibaba.com") || url_lower.contains("alizila.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "alibaba")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("deepseek.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "deepseek")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("x.ai") || url_lower.contains("x.com") {
        scores.iter_mut().find(|s| s.name == "x").unwrap().score = 100;
    }
    if url_lower.contains("mistral.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "mistral")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("microsoft.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "microsoft")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("machinelearning.apple.com") || url_lower.contains("apple.com") {
        scores.iter_mut().find(|s| s.name == "apple").unwrap().score = 100;
    }
    if url_lower.contains("bair.berkeley.edu") {
        scores
            .iter_mut()
            .find(|s| s.name == "berkeley")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("hai.stanford.edu") {
        scores
            .iter_mut()
            .find(|s| s.name == "stanford")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("inflection.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "inflection")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("stability.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "stability")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("intel.com") {
        scores.iter_mut().find(|s| s.name == "intel").unwrap().score = 100;
    }
    if url_lower.contains("amd.com") {
        scores.iter_mut().find(|s| s.name == "amd").unwrap().score = 100;
    }
    if url_lower.contains("txt.cohere.com") || url_lower.contains("cohere.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "cohere")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("deepmind.google") || url_lower.contains("deepmind.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "deepmind")
            .unwrap()
            .score = 100;
        scores
            .iter_mut()
            .find(|s| s.name == "google")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("blog.character.ai") || url_lower.contains("character.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "characterai")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("menlovc.com") {
        scores.iter_mut().find(|s| s.name == "menlo").unwrap().score = 100;
    }
    if url_lower.contains("science.org") {
        scores
            .iter_mut()
            .find(|s| s.name == "science")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("airesearch.news") {
        scores
            .iter_mut()
            .find(|s| s.name == "airesearch")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("huggingface.co") || url_lower.contains("huggingface.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "huggingface")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("techcrunch.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "techcrunch")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("perplexity.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "perplexity")
            .unwrap()
            .score = 100;
    }

    // Robotics sources
    if url_lower.contains("bostondynamics.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "boston_dynamics")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("therobotreport.com") || url_lower.contains("robotreport.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "robot_report")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("roboticsbusinessreview.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "robotics_business")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("robohub.org") {
        scores
            .iter_mut()
            .find(|s| s.name == "robohub")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("ieee.org")
        && (url_lower.contains("robotics") || url_lower.contains("advancing-technology"))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "ieee_robotics")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("automate.org") && url_lower.contains("robotics") {
        scores
            .iter_mut()
            .find(|s| s.name == "robotics_org")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("abb.com") || url_lower.contains("global.abb") {
        scores
            .iter_mut()
            .find(|s| s.name == "abb_robotics")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("kuka.com") {
        scores.iter_mut().find(|s| s.name == "kuka").unwrap().score = 100;
    }
    if url_lower.contains("universal-robots.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "universal_robots")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("omron.com") && url_lower.contains("automation") {
        scores.iter_mut().find(|s| s.name == "omron").unwrap().score = 100;
    }
    if url_lower.contains("yaskawa.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "yaskawa")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("agilityrobotics.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "agility")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("unitree.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "unitree")
            .unwrap()
            .score = 100;
    }

    // Quantum computing sources
    if url_lower.contains("quantumcomputingreport.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "quantum_computing_report")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("research.ibm.com") && url_lower.contains("quantum") {
        scores
            .iter_mut()
            .find(|s| s.name == "ibm_quantum")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("quantamagazine.org") {
        scores
            .iter_mut()
            .find(|s| s.name == "quanta")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("rigetti.com")
        || (url_lower.contains("globenewswire.com")
            && (url_lower.contains("rigetti") || title_lower.contains("rigetti")))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "rigetti")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("ionq.com") {
        scores.iter_mut().find(|s| s.name == "ionq").unwrap().score = 100;
    }
    if url_lower.contains("dwavequantum.com") || url_lower.contains("d-wave.com") {
        scores.iter_mut().find(|s| s.name == "dwave").unwrap().score = 100;
    }
    if url_lower.contains("quantinuum.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "quantinuum")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("pasqal.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "pasqal")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("xanadu.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "xanadu")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("infleqtion.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "infleqtion")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("quantumcomputinginc.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "quantum_computing_inc")
            .unwrap()
            .score = 100;
    }

    // AI startups
    if url_lower.contains("adept.ai") {
        scores.iter_mut().find(|s| s.name == "adept").unwrap().score = 100;
    }
    if url_lower.contains("assemblyai.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "assemblyai")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("replicate.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "replicate")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("langchain.com") || url_lower.contains("blog.langchain.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "langchain")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("pinecone.io") {
        scores
            .iter_mut()
            .find(|s| s.name == "pinecone")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("weaviate.io") {
        scores
            .iter_mut()
            .find(|s| s.name == "weaviate")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("together.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "together")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("anyscale.com") {
        scores
            .iter_mut()
            .find(|s| s.name == "anyscale")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("modal.com") {
        scores.iter_mut().find(|s| s.name == "modal").unwrap().score = 100;
    }
    if url_lower.contains("continual.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "continual")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("fast.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "fastai")
            .unwrap()
            .score = 100;
    }
    if url_lower.contains("eleuther.ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "eleuther")
            .unwrap()
            .score = 100;
    }

    // Keyword matches (fallback)
    if url_lower.contains("boston") || title_lower.contains("boston dynamics") {
        scores
            .iter_mut()
            .find(|s| s.name == "boston_dynamics")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "boston_dynamics")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("robot")
        && (url_lower.contains("report") || title_lower.contains("robot report"))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "robot_report")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "robot_report")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("robotics") && url_lower.contains("business") {
        scores
            .iter_mut()
            .find(|s| s.name == "robotics_business")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "robotics_business")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("robohub") || title_lower.contains("robohub") {
        scores
            .iter_mut()
            .find(|s| s.name == "robohub")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "robohub").unwrap().score,
            50,
        );
    }
    if url_lower.contains("ieee")
        && (url_lower.contains("robotics") || url_lower.contains("advancing"))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "ieee_robotics")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "ieee_robotics")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("automate.org")
        || (url_lower.contains("automate") && url_lower.contains("robotics"))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "robotics_org")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "robotics_org")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("abb") || title_lower.contains("abb robotics") {
        scores
            .iter_mut()
            .find(|s| s.name == "abb_robotics")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "abb_robotics")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("kuka") || title_lower.contains("kuka") {
        scores.iter_mut().find(|s| s.name == "kuka").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "kuka").unwrap().score, 50);
    }
    if url_lower.contains("universal") && url_lower.contains("robot") {
        scores
            .iter_mut()
            .find(|s| s.name == "universal_robots")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "universal_robots")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("omron") || title_lower.contains("omron") {
        scores.iter_mut().find(|s| s.name == "omron").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "omron").unwrap().score, 50);
    }
    if url_lower.contains("yaskawa") || title_lower.contains("yaskawa") {
        scores
            .iter_mut()
            .find(|s| s.name == "yaskawa")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "yaskawa").unwrap().score,
            50,
        );
    }
    if url_lower.contains("agility") && url_lower.contains("robotics") {
        scores
            .iter_mut()
            .find(|s| s.name == "agility")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "agility").unwrap().score,
            50,
        );
    }
    if url_lower.contains("unitree") || title_lower.contains("unitree") {
        scores
            .iter_mut()
            .find(|s| s.name == "unitree")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "unitree").unwrap().score,
            50,
        );
    }

    // Quantum computing keyword matches
    if url_lower.contains("quantum")
        && url_lower.contains("computing")
        && url_lower.contains("report")
    {
        scores
            .iter_mut()
            .find(|s| s.name == "quantum_computing_report")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "quantum_computing_report")
                .unwrap()
                .score,
            50,
        );
    }
    if (url_lower.contains("ibm") || title_lower.contains("ibm"))
        && (url_lower.contains("quantum") || title_lower.contains("quantum"))
    {
        scores
            .iter_mut()
            .find(|s| s.name == "ibm_quantum")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "ibm_quantum")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("quanta") || title_lower.contains("quanta magazine") {
        scores
            .iter_mut()
            .find(|s| s.name == "quanta")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "quanta").unwrap().score,
            50,
        );
    }
    if url_lower.contains("rigetti") || title_lower.contains("rigetti") {
        scores
            .iter_mut()
            .find(|s| s.name == "rigetti")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "rigetti").unwrap().score,
            50,
        );
    }
    if url_lower.contains("ionq") || title_lower.contains("ionq") {
        scores.iter_mut().find(|s| s.name == "ionq").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "ionq").unwrap().score, 50);
    }
    if url_lower.contains("d-wave") || url_lower.contains("dwave") || title_lower.contains("d-wave")
    {
        scores.iter_mut().find(|s| s.name == "dwave").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "dwave").unwrap().score, 50);
    }
    if url_lower.contains("quantinuum") || title_lower.contains("quantinuum") {
        scores
            .iter_mut()
            .find(|s| s.name == "quantinuum")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "quantinuum")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("pasqal") || title_lower.contains("pasqal") {
        scores
            .iter_mut()
            .find(|s| s.name == "pasqal")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "pasqal").unwrap().score,
            50,
        );
    }
    if url_lower.contains("xanadu") || title_lower.contains("xanadu") {
        scores
            .iter_mut()
            .find(|s| s.name == "xanadu")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "xanadu").unwrap().score,
            50,
        );
    }
    if url_lower.contains("infleqtion") || title_lower.contains("infleqtion") {
        scores
            .iter_mut()
            .find(|s| s.name == "infleqtion")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "infleqtion")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("quantumcomputinginc") || title_lower.contains("quantum computing inc") {
        scores
            .iter_mut()
            .find(|s| s.name == "quantum_computing_inc")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "quantum_computing_inc")
                .unwrap()
                .score,
            50,
        );
    }

    // AI startups keyword matches
    if url_lower.contains("adept") || title_lower.contains("adept") {
        scores.iter_mut().find(|s| s.name == "adept").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "adept").unwrap().score, 50);
    }
    if url_lower.contains("assemblyai")
        || title_lower.contains("assemblyai")
        || title_lower.contains("assembly ai")
    {
        scores
            .iter_mut()
            .find(|s| s.name == "assemblyai")
            .unwrap()
            .score = std::cmp::max(
            scores
                .iter()
                .find(|s| s.name == "assemblyai")
                .unwrap()
                .score,
            50,
        );
    }
    if url_lower.contains("replicate") || title_lower.contains("replicate") {
        scores
            .iter_mut()
            .find(|s| s.name == "replicate")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "replicate").unwrap().score,
            50,
        );
    }
    if url_lower.contains("langchain") || title_lower.contains("langchain") {
        scores
            .iter_mut()
            .find(|s| s.name == "langchain")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "langchain").unwrap().score,
            50,
        );
    }
    if url_lower.contains("pinecone") || title_lower.contains("pinecone") {
        scores
            .iter_mut()
            .find(|s| s.name == "pinecone")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "pinecone").unwrap().score,
            50,
        );
    }
    if url_lower.contains("weaviate") || title_lower.contains("weaviate") {
        scores
            .iter_mut()
            .find(|s| s.name == "weaviate")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "weaviate").unwrap().score,
            50,
        );
    }
    if url_lower.contains("together") && url_lower.contains("ai") {
        scores
            .iter_mut()
            .find(|s| s.name == "together")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "together").unwrap().score,
            50,
        );
    }
    if url_lower.contains("anyscale") || title_lower.contains("anyscale") {
        scores
            .iter_mut()
            .find(|s| s.name == "anyscale")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "anyscale").unwrap().score,
            50,
        );
    }
    if url_lower.contains("modal") || title_lower.contains("modal") {
        scores.iter_mut().find(|s| s.name == "modal").unwrap().score =
            std::cmp::max(scores.iter().find(|s| s.name == "modal").unwrap().score, 50);
    }
    if url_lower.contains("continual") || title_lower.contains("continual") {
        scores
            .iter_mut()
            .find(|s| s.name == "continual")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "continual").unwrap().score,
            50,
        );
    }
    if url_lower.contains("fast.ai")
        || title_lower.contains("fast.ai")
        || title_lower.contains("fastai")
    {
        scores
            .iter_mut()
            .find(|s| s.name == "fastai")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "fastai").unwrap().score,
            50,
        );
    }
    if url_lower.contains("eleuther") || title_lower.contains("eleuther") {
        scores
            .iter_mut()
            .find(|s| s.name == "eleuther")
            .unwrap()
            .score = std::cmp::max(
            scores.iter().find(|s| s.name == "eleuther").unwrap().score,
            50,
        );
    }

    // Return category with highest score
    if let Some(winner) = scores.iter().max_by_key(|s| s.score) {
        if winner.score > 0 {
            return winner.name.to_string();
        }
    }

    "technology".to_string()
}
