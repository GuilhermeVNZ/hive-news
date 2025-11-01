// Script para migrar artigos já processados para o registry
// Executa: cargo run --bin migrate-registry

// Importar módulos do crate principal
include!("../../src/utils/article_registry.rs");

// Re-exportar tipos/tipos específicos que precisamos
// Result, Utc, fs, Path, PathBuf já estão incluídos via article_registry.rs

fn main() -> Result<()> {
    // Simular estrutura do módulo
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    
    println!("🔄 Article Registry Migration");
    println!("============================\n");

    let registry_path = Path::new("G:/Hive-Hub/News-main/articles_registry.json");
    
    // Carregar registry existente (ou criar novo)
    let mut registry = match ArticleRegistry::load(registry_path) {
        Ok(r) => r,
        Err(_) => {
            println!("   Creating new registry...");
            ArticleRegistry {
                articles: std::collections::HashMap::new(),
            }
        }
    };

    println!("📂 Scanning existing articles...\n");

    let base_dir = Path::new("G:/Hive-Hub/News-main/downloads");
    let output_dir = Path::new("G:/Hive-Hub/News-main/output/AIResearch");
    
    let mut migrated_count = 0;
    let mut skipped_count = 0;

    // 1. Migrar artigos de output/ (já publicados)
    if output_dir.exists() {
        println!("📄 Scanning published articles in output/AIResearch/...");
        
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let article_dir = entry.path();
            
            if article_dir.is_dir() {
                let article_id = article_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Verificar se já tem article.md (publicado)
                let article_file = article_dir.join("article.md");
                if article_file.exists() {
                    // Verificar se título existe
                    let title_file = article_dir.join("title.txt");
                    let title = if title_file.exists() {
                        fs::read_to_string(&title_file).unwrap_or_else(|_| "Untitled".to_string())
                    } else {
                        "Untitled".to_string()
                    };

                    let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);

                    // Verificar se já está no registry
                    if !registry.is_article_registered(article_id) {
                        println!("  ✅ Migrating published: {} - {}", article_id, title);
                        
                        // Criar metadata completo
                        let mut metadata = ArticleMetadata {
                            id: article_id.to_string(),
                            title: title.trim().to_string(),
                            arxiv_url: arxiv_url.clone(),
                            pdf_url: pdf_url.clone(),
                            status: ArticleStatus::Published,
                            filter_score: None, // Não temos score histórico
                            category: None,
                            rejection_reason: None,
                            collected_at: None,
                            filtered_at: None,
                            rejected_at: None,
                            published_at: Some(Utc::now()), // Aproximação
                            output_dir: Some(article_dir.clone()),
                            hidden: Some(false),
                            featured: Some(false),
                            destinations: None,
                        };

                        // Tentar ler categoria se existir
                        let category_file = article_dir.join("image_categories.txt");
                        if category_file.exists() {
                            if let Ok(cats) = fs::read_to_string(&category_file) {
                                if let Some(first_cat) = cats.lines().next() {
                                    metadata.category = Some(first_cat.trim().to_string());
                                }
                            }
                        }

                        registry.articles.insert(article_id.to_string(), metadata);
                        migrated_count += 1;
                    } else {
                        skipped_count += 1;
                    }
                }
            }
        }
    }

    // 2. Migrar artigos de downloads/filtered/ (filtrados mas não publicados)
    let filtered_dir = base_dir.join("filtered");
    if filtered_dir.exists() {
        println!("\n🔍 Scanning filtered articles in downloads/filtered/...");
        
        for entry in fs::read_dir(&filtered_dir)? {
            let entry = entry?;
            let category_dir = entry.path();
            
            if category_dir.is_dir() {
                let category = category_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                for pdf_entry in fs::read_dir(&category_dir)? {
                    let pdf_entry = pdf_entry?;
                    let pdf_path = pdf_entry.path();
                    
                    if pdf_path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                        let article_id = pdf_path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        
                        if !registry.is_article_registered(article_id) {
                            println!("  ✅ Migrating filtered: {} → {}", article_id, category);
                            
                            let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                            let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);
                            
                            let metadata = ArticleMetadata {
                                id: article_id.to_string(),
                                title: "Untitled (from migration)".to_string(),
                                arxiv_url,
                                pdf_url,
                                status: ArticleStatus::Filtered,
                                filter_score: None,
                                category: Some(category.to_string()),
                                rejection_reason: None,
                                collected_at: None,
                                filtered_at: Some(Utc::now()),
                                rejected_at: None,
                                published_at: None,
                                output_dir: None,
                                hidden: Some(false),
                                featured: Some(false),
                                destinations: None,
                            };
                            
                            registry.articles.insert(article_id.to_string(), metadata);
                            migrated_count += 1;
                        } else {
                            skipped_count += 1;
                        }
                    }
                }
            }
        }
    }

    // 3. Migrar artigos de downloads/rejected/ (rejeitados)
    let rejected_dir = base_dir.join("rejected");
    if rejected_dir.exists() {
        println!("\n❌ Scanning rejected articles in downloads/rejected/...");
        
        for entry in fs::read_dir(&rejected_dir)? {
            let entry = entry?;
            let pdf_path = entry.path();
            
            if pdf_path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                let article_id = pdf_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if !registry.is_article_registered(article_id) {
                    println!("  ✅ Migrating rejected: {}", article_id);
                    
                    let arxiv_url = format!("https://arxiv.org/abs/{}", article_id);
                    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", article_id);
                    
                    let metadata = ArticleMetadata {
                        id: article_id.to_string(),
                        title: "Untitled (from migration)".to_string(),
                        arxiv_url,
                        pdf_url,
                        status: ArticleStatus::Rejected,
                        filter_score: None,
                        category: None,
                        rejection_reason: Some("Migrated from rejected/".to_string()),
                        collected_at: None,
                        filtered_at: None,
                        rejected_at: Some(Utc::now()),
                        published_at: None,
                        output_dir: None,
                        hidden: Some(false),
                        featured: Some(false),
                        destinations: None,
                    };
                    
                    registry.articles.insert(article_id.to_string(), metadata);
                    migrated_count += 1;
                } else {
                    skipped_count += 1;
                }
            }
        }
    }

    // Salvar registry
    println!("\n💾 Saving registry...");
    registry.save(registry_path)?;

    println!("\n✅ Migration completed!");
    println!("   Migrated: {} articles", migrated_count);
    println!("   Skipped (already in registry): {} articles", skipped_count);
    println!("   Total in registry: {} articles", registry.articles.len());

    Ok(())
}

