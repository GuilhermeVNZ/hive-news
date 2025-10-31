# Collectors de Artigos Científicos

Esta pasta contém os collectors (coletores) de artigos de diferentes fontes de dados.

## 📋 Collectors Disponíveis

- **`arxiv_collector.rs`**: Coleta artigos do arXiv
- **`pmc_collector.rs`**: Coleta artigos do PubMed Central (PMC)
- **`course_collector.rs`**: Coleta informações de cursos
- **`template_collector.rs`**: Template para criar novos collectors

## ➕ Como Adicionar um Novo Collector

### Passo 1: Criar o arquivo do collector

Copie o template:

```bash
cd src/collectors
cp template_collector.rs meu_novo_collector.rs
```

### Passo 2: Implementar os métodos principais

Edite `meu_novo_collector.rs` e implemente:

1. **`fetch_recent_papers`**: Busca artigos da API
2. **`download_pdf`**: Faz download do PDF do artigo

**Exemplo de busca:**

```rust
pub async fn fetch_recent_papers(
    &self,
    category: &str,
    max_results: usize,
) -> Result<Vec<ArticleMetadata>> {
    let url = format!(
        "https://api.exemplo.com/v1/papers?category={}&limit={}",
        category, max_results
    );

    let response = self.client
        .get(&url)
        .header("Authorization", "Bearer sua-chave")
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;

    let mut articles = Vec::new();
    for paper in json["results"].as_array().unwrap_or(&vec![]) {
        articles.push(ArticleMetadata {
            id: paper["id"].as_str().unwrap().to_string(),
            title: paper["title"].as_str().unwrap().to_string(),
            authors: extract_authors(paper),
            abstract_text: paper["abstract"].as_str().unwrap_or("").to_string(),
            published_date: paper["date"].as_str().unwrap_or("").to_string(),
            doi: paper["doi"].as_str().map(|s| s.to_string()),
            pdf_url: paper["pdf_url"].as_str().unwrap_or("").to_string(),
            url: paper["url"].as_str().unwrap_or("").to_string(),
            categories: vec![category.to_string()],
        });
    }

    Ok(articles)
}
```

### Passo 3: Registrar no `mod.rs`

```rust
// src/collectors/mod.rs
pub mod arxiv_collector;
pub mod pmc_collector;
pub mod meu_novo_collector;  // Adicionar esta linha

pub use arxiv_collector::*;
pub use pmc_collector::*;
pub use meu_novo_collector::*;  // Adicionar esta linha
```

### Passo 4: Criar função de coleta no `main.rs`

```rust
// Em src/main.rs, adicionar função similar a run_arxiv_collection_direct:

async fn run_meu_novo_collection_direct() -> anyhow::Result<()> {
    use crate::collectors::meu_novo_collector::MeuNovoCollector;
    use crate::utils::article_registry::RegistryManager;
    use std::path::Path;

    // Inicializar registry
    let registry_path = Path::new("/opt/news-system/articles_registry.json");
    let registry = RegistryManager::new(registry_path)?;

    // Inicializar collector
    let base_dir = Path::new("/opt/news-system/downloads");
    let temp_dir = base_dir.join("temp");
    let collector = MeuNovoCollector::new(temp_dir);

    println!("📡 Fetching papers from Nova API...");

    // Buscar artigos
    let articles = collector.fetch_recent_papers("ai", 10).await?;
    println!("✅ Found {} papers", articles.len());

    // Processar cada artigo
    for article in articles {
        // Verificar se já existe
        if registry.is_article_registered(&article.id) {
            println!("⏭️  Skipping {} (already in registry)", article.id);
            continue;
        }

        // Download PDF
        let pdf_dir = base_dir.join("novo-collector");
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = pdf_dir.join(&date);
        tokio::fs::create_dir_all(&date_dir).await?;

        let pdf_path = date_dir.join(&format!("{}.pdf", article.id));
        
        if !article.pdf_url.is_empty() {
            collector.download_pdf(&article.id, &article.pdf_url, &pdf_path).await?;
            println!("⬇️  Downloaded: {}", article.id);
        } else {
            println!("⚠️  No PDF URL for {}", article.id);
            continue;
        }

        // Registrar no registry
        registry.register_collected(&article.id, &article.title)?;

        println!("✅ Processed: {} - {}", article.id, article.title);
    }

    Ok(())
}
```

### Passo 5: Adicionar comando no `main.rs`

```rust
// Em src/main.rs, na função main():

let args: Vec<String> = std::env::args().collect();
let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

match command {
    "collect" => {
        // Opção 1: Manter apenas arXiv
        run_arxiv_collection_direct().await?;
        
        // Opção 2: Executar múltiplos collectors
        run_arxiv_collection_direct().await?;
        run_meu_novo_collection_direct().await?;
    }
    "collect-arxiv" => run_arxiv_collection_direct().await?,
    "collect-novo" => run_meu_novo_collection_direct().await?,
    // ...
}
```

### Passo 6: Integrar no `start.rs` (opcional)

Se quiser que o novo collector rode automaticamente:

```rust
// Em start.rs, na função execute_full_pipeline():

println!("\n📥 Phase 1: Collecting papers...");

// Coletar do arXiv
run_arxiv_collection();

// Coletar do novo collector
run_meu_novo_collection();  // Adicionar esta chamada
```

## 🔍 Estrutura de ArticleMetadata

Todos os collectors devem retornar `ArticleMetadata`:

```rust
pub struct ArticleMetadata {
    pub id: String,                 // ID único (ex: "2510.12345" ou "10.1234/abc")
    pub title: String,               // Título do artigo
    pub authors: Vec<String>,        // Lista de autores
    pub abstract_text: String,       // Resumo/abstract
    pub published_date: String,       // Data de publicação (ISO format)
    pub doi: Option<String>,          // DOI (se disponível)
    pub pdf_url: String,             // URL do PDF para download
    pub url: String,                 // URL do artigo na origem
    pub categories: Vec<String>,      // Categorias/tags
}
```

## 📚 APIs Recomendadas para Novos Collectors

### 1. Semantic Scholar API
- **URL**: `https://api.semanticscholar.org`
- **Documentação**: https://www.semanticscholar.org/product/api
- **Vantagens**: Gratuita, sem autenticação (limitado), muitos metadados

### 2. Crossref API
- **URL**: `https://api.crossref.org`
- **Documentação**: https://www.crossref.org/documentation/retrieve-metadata/
- **Vantagens**: Grandes publishers, DOI lookup

### 3. PubMed Central (PMC)
- **Já implementado**: `pmc_collector.rs`
- **Documentação**: https://www.ncbi.nlm.nih.gov/pmc/tools/developers/

### 4. HAL (HAL Archives-Ouvertes)
- **URL**: https://api.archives-ouvertes.fr
- **Vantagens**: Acesso aberto, API REST simples

### 5. CORE API
- **URL**: https://core.ac.uk
- **Documentação**: https://core.ac.uk/developers/api
- **Vantagens**: Grande base de artigos open access

## ✅ Checklist de Implementação

- [ ] Criar arquivo do collector (`novo_collector.rs`)
- [ ] Implementar `fetch_recent_papers()`
- [ ] Implementar `download_pdf()`
- [ ] Registrar em `mod.rs`
- [ ] Criar função de coleta em `main.rs`
- [ ] Testar localmente
- [ ] Adicionar tratamento de erros
- [ ] Adicionar rate limiting (se necessário)
- [ ] Documentar API keys/autenticação necessária
- [ ] Commit e push para repositório
- [ ] Deploy no servidor usando `deploy.sh`

## 🔒 Segurança e Boas Práticas

1. **API Keys**: Nunca commitar API keys no código
   - Use variáveis de ambiente (`.env`)
   - Exemplo: `std::env::var("SEMANTIC_SCHOLAR_API_KEY")`

2. **Rate Limiting**: Respeite limites da API
   ```rust
   tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
   ```

3. **User-Agent**: Sempre configure user-agent apropriado
   ```rust
   .user_agent("News-System-Collector/1.0 (YourCollector)")
   ```

4. **Timeout**: Configure timeout para requisições
   ```rust
   .timeout(std::time::Duration::from_secs(60))
   ```

5. **Tratamento de Erros**: Use `anyhow::Result` e `Context`
   ```rust
   .context("Failed to fetch papers")?
   ```

## 🐛 Debugging

Para debugar um collector:

1. **Testar isoladamente:**
   ```bash
   cd news-backend
   cargo run --bin news-backend collect-novo
   ```

2. **Ver logs:**
   ```bash
   RUST_LOG=debug cargo run --bin news-backend collect-novo
   ```

3. **Testar download manual:**
   ```rust
   // No código, adicionar logs:
   println!("DEBUG: Fetching from URL: {}", url);
   println!("DEBUG: Response status: {:?}", response.status());
   ```

## 📝 Exemplos de Uso

Ver arquivos:
- `arxiv_collector.rs` - Exemplo completo funcional
- `template_collector.rs` - Template com comentários explicativos














