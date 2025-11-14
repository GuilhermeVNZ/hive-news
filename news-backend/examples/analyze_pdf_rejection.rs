use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 ANÁLISE DE REJEIÇÃO DE PDFs\n");

    // 1. Verificar quantos PDFs foram coletados
    let downloads_dir = Path::new("G:/ClipaAi/clipai/output/downloads");
    let pdf_count = count_pdfs(downloads_dir)?;

    println!("📊 PDFs no diretório de downloads: {}", pdf_count);

    // 2. Verificar o registry
    let registry_path = Path::new("G:/ClipaAi/clipai/articles_registry.json");
    if registry_path.exists() {
        let registry: Value = serde_json::from_str(&fs::read_to_string(registry_path)?)?;

        if let Some(articles) = registry.as_array() {
            let total = articles.len();
            let mut filtered = 0;
            let mut rejected = 0;
            let mut downloaded = 0;
            let mut pdf_articles = 0;

            for article in articles {
                if let Some(source_type) = article.get("source_type").and_then(|s| s.as_str()) {
                    if source_type.contains("arxiv") || source_type.contains("pdf") {
                        pdf_articles += 1;
                    }
                }

                if let Some(status) = article.get("status").and_then(|s| s.as_str()) {
                    match status {
                        "filtered" => filtered += 1,
                        "rejected" => rejected += 1,
                        "downloaded" => downloaded += 1,
                        _ => {}
                    }
                }
            }

            println!("\n📋 REGISTRY STATUS:");
            println!("   Total de artigos: {}", total);
            println!("   Artigos PDF/arXiv: {}", pdf_articles);
            println!("   Status 'downloaded': {}", downloaded);
            println!("   Status 'filtered': {}", filtered);
            println!("   Status 'rejected': {}", rejected);
            println!("\n🔍 ANÁLISE:");

            if rejected > filtered {
                println!(
                    "   ⚠️  PROBLEMA: {} artigos rejeitados vs {} aprovados",
                    rejected, filtered
                );
                println!("   Possíveis causas:");
                println!("   1. Filtro de 'experimental sections' muito rigoroso");
                println!("   2. Fake penalty muito alto (>0.5)");
                println!("   3. Falha na extração de texto do PDF");
            }
        }
    }

    // 3. Verificar se PDFs existem fisicamente mas não foram processados
    println!("\n📁 Verificando PDFs não processados...");
    let unprocessed = check_unprocessed_pdfs(downloads_dir, registry_path)?;

    if unprocessed > 0 {
        println!(
            "   ⚠️  {} PDFs baixados mas não aparecem no registry!",
            unprocessed
        );
    }

    Ok(())
}

fn count_pdfs(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in fs::read_dir(dir)?.flatten() {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("pdf") {
            count += 1;
        }
    }
    Ok(count)
}

fn check_unprocessed_pdfs(downloads_dir: &Path, registry_path: &Path) -> Result<usize> {
    if !downloads_dir.exists() || !registry_path.exists() {
        return Ok(0);
    }

    let registry: Value = serde_json::from_str(&fs::read_to_string(registry_path)?)?;
    let empty_vec = Vec::new();
    let articles = registry.as_array().unwrap_or(&empty_vec);

    let mut registry_ids = std::collections::HashSet::new();
    for article in articles {
        if let Some(id) = article.get("id").and_then(|i| i.as_str()) {
            registry_ids.insert(id.to_string());
        }
    }

    let mut unprocessed = 0;
    for entry in fs::read_dir(downloads_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Extrair arXiv ID do nome do arquivo (ex: 2510.12345.pdf)
                let arxiv_id = stem.replace(".", "");
                if !registry_ids.contains(&arxiv_id) && !registry_ids.contains(stem) {
                    println!("      📄 Não processado: {}", stem);
                    unprocessed += 1;
                }
            }
        }
    }

    Ok(unprocessed)
}
