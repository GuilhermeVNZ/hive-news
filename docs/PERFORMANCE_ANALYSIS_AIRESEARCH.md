# 🚀 Análise de Performance e Escalabilidade - AIResearch

**Data:** 2025-11-22  
**Status:** Análise Crítica - Lentidão Identificada  
**Artigos Atuais:** ~800 artigos publicados  

---

## 📊 **Diagnóstico Atual**

### 🔍 **Estatísticas do Sistema:**
- **Artigos na pasta:** 800 diretórios
- **Artigos no registry:** 314 válidos para AIResearch  
- **Artigos não processados:** 486 pastas sem registry
- **Status incorreto:** 174 artigos
- **Tamanho médio por artigo:** ~3.3KB (texto completo)

### ⏱️ **Tempos de Resposta Medidos:**
- **Página inicial (SSR):** 158ms
- **API (6 artigos):** 85ms  
- **API (50 artigos):** 180ms

### 🚨 **Gargalos Identificados:**

#### 1. **Carregamento Completo de Artigos**
```rust
// PROBLEMA: Carrega texto completo de TODOS os artigos na memória
let haystack = normalize_for_search(&format!(
    "{} {} {} {} {} {}",
    article.title,
    article.id,
    &article.excerpt,
    article.category,
    topics,
    &article.article  // ← TEXTO COMPLETO (~3KB por artigo)
));
```

**Impacto:** 800 artigos × 3KB = ~2.4MB de texto carregado na RAM por requisição

#### 2. **Busca Linear em Memória**
```rust
// PROBLEMA: Busca sequencial em todos os artigos
.filter(|article| {
    article_matches_search(article, &search_words)
})
```

**Impacto:** O(n) para cada busca, sem índices

#### 3. **Cache Ineficiente**
- Cache baseado apenas em signature do registry
- Não há cache de resultados de busca
- Reprocessamento completo a cada query diferente

#### 4. **Sem Paginação Eficiente**
- Frontend carrega artigos em batches pequenos (6 iniciais)
- Mas backend processa TODOS os artigos para filtrar

---

## 🎯 **Soluções de Escalabilidade**

### 🏗️ **Arquitetura Recomendada para Grandes Portais**

#### **Nível 1: Otimizações Imediatas (0-1000 artigos)**

1. **Índice de Busca em Memória**
```rust
struct SearchIndex {
    title_index: HashMap<String, Vec<String>>, // palavra -> [article_ids]
    content_index: HashMap<String, Vec<String>>,
    category_index: HashMap<String, Vec<String>>,
}
```

2. **Cache de Resultados**
```rust
struct QueryCache {
    results: LruCache<String, Vec<Article>>, // query_hash -> articles
    ttl: Duration,
}
```

3. **Lazy Loading de Conteúdo**
```rust
struct ArticleSummary {
    id: String,
    title: String,
    excerpt: String,
    // Sem campo 'article' (conteúdo completo)
}
```

#### **Nível 2: Banco de Dados (1000-10000 artigos)**

1. **PostgreSQL com Full-Text Search**
```sql
CREATE TABLE articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    excerpt TEXT,
    content TEXT,
    published_at TIMESTAMP,
    category TEXT,
    search_vector tsvector GENERATED ALWAYS AS (
        to_tsvector('english', title || ' ' || excerpt || ' ' || content)
    ) STORED
);

CREATE INDEX idx_articles_search ON articles USING GIN(search_vector);
CREATE INDEX idx_articles_category ON articles(category);
CREATE INDEX idx_articles_published ON articles(published_at DESC);
```

2. **Paginação Eficiente**
```sql
SELECT id, title, excerpt, published_at, category
FROM articles 
WHERE search_vector @@ plainto_tsquery('artificial intelligence')
ORDER BY published_at DESC 
LIMIT 20 OFFSET 0;
```

#### **Nível 3: Elasticsearch (10000+ artigos)**

1. **Índice Otimizado**
```json
{
  "mappings": {
    "properties": {
      "title": { "type": "text", "boost": 3.0 },
      "excerpt": { "type": "text", "boost": 2.0 },
      "content": { "type": "text" },
      "category": { "type": "keyword" },
      "published_at": { "type": "date" },
      "tags": { "type": "keyword" }
    }
  }
}
```

2. **Busca Avançada**
```json
{
  "query": {
    "bool": {
      "must": [
        {
          "multi_match": {
            "query": "artificial intelligence",
            "fields": ["title^3", "excerpt^2", "content"]
          }
        }
      ],
      "filter": [
        { "term": { "category": "ai" } },
        { "range": { "published_at": { "gte": "2024-01-01" } } }
      ]
    }
  },
  "sort": [{ "published_at": { "order": "desc" } }],
  "from": 0,
  "size": 20
}
```

---

## 🛠️ **Implementação Prioritária**

### **Fase 1: Otimizações Críticas (Implementar AGORA)**

#### 1. **Separar Listagem de Conteúdo**
```rust
#[derive(Serialize)]
struct ArticleListItem {
    id: String,
    slug: String,
    title: String,
    excerpt: String,
    published_at: String,
    category: String,
    read_time: u32,
    image_path: Option<String>,
    featured: bool,
    // SEM campo 'article' (conteúdo completo)
}

#[derive(Serialize)]
struct ArticleDetail {
    // Todos os campos + conteúdo completo
    article: String,
}
```

#### 2. **Cache de Busca**
```rust
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

static SEARCH_CACHE: Lazy<RwLock<LruCache<u64, Arc<Vec<ArticleListItem>>>>> = 
    Lazy::new(|| RwLock::new(LruCache::new(100)));

fn get_query_hash(category: &Option<String>, query: &Option<String>) -> u64 {
    let mut hasher = DefaultHasher::new();
    category.hash(&mut hasher);
    query.hash(&mut hasher);
    hasher.finish()
}
```

#### 3. **Índice de Busca Simples**
```rust
struct SimpleSearchIndex {
    word_to_articles: HashMap<String, HashSet<String>>,
}

impl SimpleSearchIndex {
    fn build(articles: &[Article]) -> Self {
        let mut index = HashMap::new();
        
        for article in articles {
            let words = normalize_for_search(&format!(
                "{} {} {}", 
                article.title, 
                article.excerpt,
                article.category
            )).split_whitespace();
            
            for word in words {
                index.entry(word.to_string())
                     .or_insert_with(HashSet::new)
                     .insert(article.id.clone());
            }
        }
        
        Self { word_to_articles: index }
    }
    
    fn search(&self, query: &str) -> HashSet<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        if words.is_empty() { return HashSet::new(); }
        
        let mut result = self.word_to_articles
            .get(words[0])
            .cloned()
            .unwrap_or_default();
            
        for word in &words[1..] {
            if let Some(word_articles) = self.word_to_articles.get(*word) {
                result = result.intersection(word_articles).cloned().collect();
            } else {
                return HashSet::new();
            }
        }
        
        result
    }
}
```

### **Fase 2: Banco de Dados (Próximas semanas)**

1. **Migração para PostgreSQL**
2. **Índices de busca full-text**
3. **Cache Redis para resultados**

### **Fase 3: CDN e Cache (Médio prazo)**

1. **Cloudflare/AWS CloudFront**
2. **Cache de imagens**
3. **Compressão gzip/brotli**

---

## 📈 **Benchmarks Esperados**

### **Após Fase 1:**
- **Tempo de resposta:** 85ms → 15ms
- **Memória por request:** 2.4MB → 50KB
- **Capacidade:** 800 → 5000 artigos

### **Após Fase 2:**
- **Tempo de resposta:** 15ms → 5ms
- **Busca complexa:** Suporte a operadores AND/OR
- **Capacidade:** 5000 → 50000 artigos

### **Após Fase 3:**
- **TTFB:** 5ms → 1ms (cache edge)
- **Imagens:** Lazy loading + WebP
- **Capacidade:** 50000+ artigos

---

## 🎯 **Próximos Passos**

1. **[CRÍTICO]** Implementar ArticleListItem sem conteúdo completo
2. **[CRÍTICO]** Adicionar cache de busca LRU
3. **[IMPORTANTE]** Criar índice de busca em memória
4. **[MÉDIO]** Planejar migração para PostgreSQL
5. **[LONGO]** Avaliar Elasticsearch para 10k+ artigos

---

## 📚 **Referências - Grandes Portais**

### **The Verge / Vox Media**
- **Elasticsearch** para busca
- **Redis** para cache de sessão
- **CDN** para assets estáticos
- **Lazy loading** de imagens

### **TechCrunch**
- **WordPress VIP** (MySQL otimizado)
- **Memcached** para queries
- **Fastly CDN**
- **AMP** para mobile

### **Ars Technica**
- **Custom CMS** com PostgreSQL
- **Solr** para busca avançada
- **Varnish** cache reverso
- **Progressive Web App**

### **Padrões Comuns:**
1. **Separação:** Lista ≠ Conteúdo completo
2. **Cache em camadas:** Memory → Redis → DB
3. **Índices especializados:** Full-text search
4. **CDN global:** Assets + cache de borda
5. **Lazy loading:** Imagens + conteúdo

---

**🚨 AÇÃO IMEDIATA NECESSÁRIA:** A lentidão atual é causada pelo carregamento de texto completo de todos os artigos. Implementar Fase 1 resolverá 80% do problema de performance.
