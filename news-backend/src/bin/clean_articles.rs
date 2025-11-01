// Binary to clean markdown formatting from existing article.md files
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;

/// Remove formatação markdown indesejada como **Label:** do texto
fn clean_markdown_formatting(content: &str) -> String {
    let mut cleaned = content.to_string();
    
    // Remove padrões **Label:** do início de linhas
    let label_pattern = Regex::new(r"(?m)^\*\*[^:]+:\*\*\s*").unwrap();
    cleaned = label_pattern.replace_all(&cleaned, "").to_string();
    
    // Remove **Label:** no meio do texto
    let inline_label_pattern = Regex::new(r"\*\*([^:]+):\*\*\s+").unwrap();
    cleaned = inline_label_pattern.replace_all(&cleaned, "").to_string();
    
    // Limpar espaços extras entre parágrafos (mais de 2 quebras de linha)
    let extra_newlines = Regex::new(r"\n{3,}").unwrap();
    cleaned = extra_newlines.replace_all(&cleaned, "\n\n").to_string();
    
    // Trim no início e fim
    cleaned.trim().to_string()
}

async fn clean_article_file(file_path: &Path) -> Result<()> {
    let content = tokio_fs::read_to_string(file_path).await?;
    let cleaned = clean_markdown_formatting(&content);
    tokio_fs::write(file_path, cleaned).await?;
    println!("✅ Limpo: {}", file_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    let output_base = PathBuf::from("G:/Hive-Hub/News-main/output");
    
    if args.len() > 1 {
        // Limpar arquivo específico
        let article_dir = PathBuf::from(&args[1]);
        let article_file = article_dir.join("article.md");
        
        if !article_file.exists() {
            eprintln!("❌ Arquivo não encontrado: {}", article_file.display());
            std::process::exit(1);
        }
        
        clean_article_file(&article_file).await?;
    } else {
        // Limpar todos os arquivos article.md em output/
        if !output_base.exists() {
            eprintln!("❌ Diretório não encontrado: {}", output_base.display());
            std::process::exit(1);
        }
        
        let mut count = 0;
        let mut entries = Vec::new();
        
        // Collect all article.md files
        if let Ok(entries_iter) = fs::read_dir(&output_base) {
            for entry in entries_iter.flatten() {
                entries.push(entry.path());
            }
        }
        
        // Recursively find all article.md files
        fn find_article_files(dir: &Path, files: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        find_article_files(&path, files);
                    } else if path.file_name() == Some(std::ffi::OsStr::new("article.md")) {
                        files.push(path);
                    }
                }
            }
        }
        
        let mut article_files = Vec::new();
        find_article_files(&output_base, &mut article_files);
        
        println!("📄 Encontrados {} arquivos article.md", article_files.len());
        
        for file in article_files {
            clean_article_file(&file).await?;
            count += 1;
        }
        
        println!("\n✅ Limpeza concluída! Processados: {} arquivos", count);
    }
    
    Ok(())
}



