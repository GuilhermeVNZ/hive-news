# Correções para Problemas de Coleta RSS/HTML

## Problema 1: Alibaba DAMO - Erro de Parsing de Feed RSS

### Diagnóstico
- URL configurada: `https://damo.alibaba.com/news/rss`
- Status: HTTP 200
- Problema: Retorna HTML, não RSS/Atom feed
- Testadas outras URLs (`/news`, `/news/feed`, `/feed`): Todas retornam HTML

### Possibilidades de Correção

#### Opção 1: Converter para HTML Collector (Recomendado)
Como o site não possui feed RSS público, usar o HTML collector:

```json
{
  "id": "html_alibaba_damo",
  "name": "Alibaba DAMO News",
  "enabled": true,
  "collector_type": "html",
  "base_url": "https://damo.alibaba.com/news",
  "selectors": {
    "article": "article, .news-item, .post",
    "title": "h1, h2, .title",
    "content": ".content, article, main",
    "link": "a[href*='/news/']"
  }
}
```

#### Opção 2: Melhorar Detecção de Feed RSS
Adicionar lógica para detectar se a resposta é HTML ao invés de RSS/Atom:
- Verificar se resposta começa com `<?xml` ou `<feed` ou `<rss`
- Se não for feed válido, retornar erro mais claro
- Logar warning quando URL de feed retorna HTML

**Implementação:**
```rust
// Em rss_collector.rs, antes de tentar parsear:
let feed_content = response.text().await?;

// Verificar se é realmente um feed
if !feed_content.trim_start().starts_with("<?xml") 
    && !feed_content.trim_start().starts_with("<feed")
    && !feed_content.trim_start().starts_with("<rss") {
    return Err(anyhow::anyhow!(
        "Feed URL returned HTML instead of RSS/Atom feed. URL may not be a valid feed."
    ));
}
```

#### Opção 3: Procurar Feed RSS Alternativo
- Verificar se há feed RSS embutido no HTML (link rel="alternate")
- Verificar URLs alternativas como `/rss.xml`, `/atom.xml`, `/feed.xml`

#### Opção 4: Usar Web Scraping para Extrair Feed
- Se o site carrega feed via JavaScript, usar headless browser
- Não recomendado por ser complexo e custoso

---

## Problema 2: Meta AI - HTTP 400 Bad Request

### Diagnóstico
- URL: `https://ai.meta.com/blog/`
- Status: HTTP 400 Bad Request
- Tentativas: User-Agent padrão já está configurado no código

### Possibilidades de Correção

#### Opção 1: Adicionar Headers Adicionais (Recomendado)
Adicionar mais headers HTTP para simular navegador real:

```rust
// Em html_collector.rs
headers.insert(
    reqwest::header::ACCEPT_ENCODING,
    reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
);
headers.insert(
    reqwest::header::SEC_FETCH_SITE,
    reqwest::header::HeaderValue::from_static("none"),
);
headers.insert(
    reqwest::header::SEC_FETCH_MODE,
    reqwest::header::HeaderValue::from_static("navigate"),
);
headers.insert(
    reqwest::header::SEC_FETCH_USER,
    reqwest::header::HeaderValue::from_static("?1"),
);
headers.insert(
    reqwest::header::SEC_FETCH_DEST,
    reqwest::header::HeaderValue::from_static("document"),
);
```

#### Opção 2: Adicionar Cookies/Session
- Alguns sites requerem cookies de sessão
- Implementar gerenciamento de cookies com `reqwest::Client::cookie_store(true)` (já está habilitado)
- Adicionar cookie inicial ou fazer requisição preliminar

#### Opção 3: Adicionar Referer Header
```rust
headers.insert(
    reqwest::header::REFERER,
    reqwest::header::HeaderValue::from_static("https://www.google.com/"),
);
```

#### Opção 4: Tentar URL Alternativa
- Verificar se há URL alternativa como `https://about.fb.com/news/tag/meta-ai/`
- Verificar se há feed RSS em vez de HTML scraping

#### Opção 5: Usar Headless Browser (Último Recurso)
- Para sites que requerem JavaScript completo
- Usar biblioteca como `puppeteer` ou `playwright`
- Muito mais lento e complexo

#### Opção 6: Verificar Rate Limiting
- Adicionar delay maior entre requisições
- Usar IP rotativo ou proxy (complexo)

---

## Problema 3: DeepSeek - HTTP 403 Forbidden

### Diagnóstico
- URL: `https://deepseek.com/news`
- Status: HTTP 403 Forbidden
- Problema: Bot protection ativo (Cloudflare ou similar)

### Possibilidades de Correção

#### Opção 1: Adicionar Headers Mais Completos (Recomendado)
Mesma abordagem do Meta AI, adicionar todos os headers que um navegador real envia:

```rust
// Headers completos para contornar bot protection
headers.insert(
    reqwest::header::ACCEPT_ENCODING,
    reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
);
headers.insert(
    reqwest::header::SEC_FETCH_SITE,
    reqwest::header::HeaderValue::from_static("none"),
);
headers.insert(
    reqwest::header::SEC_FETCH_MODE,
    reqwest::header::HeaderValue::from_static("navigate"),
);
headers.insert(
    reqwest::header::SEC_FETCH_USER,
    reqwest::header::HeaderValue::from_static("?1"),
);
headers.insert(
    reqwest::header::SEC_FETCH_DEST,
    reqwest::header::HeaderValue::from_static("document"),
);
headers.insert(
    reqwest::header::REFERER,
    reqwest::header::HeaderValue::from_static("https://www.google.com/"),
);
```

#### Opção 2: Adicionar Delay Aleatório Entre Requisições
```rust
use rand::Rng;
let delay = rand::thread_rng().gen_range(1000..5000); // 1-5 segundos
tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
```

#### Opção 3: Verificar se Há Feed RSS Público
- Verificar `https://deepseek.com/rss`, `/feed`, `/atom.xml`
- Alguns sites bloqueiam scraping HTML mas permitem feeds RSS

#### Opção 4: Usar Proxy Rotativo
- Rotacionar IPs para evitar detecção
- Complexo e pode ser custoso
- Usar serviços como Bright Data, ScraperAPI, etc.

#### Opção 5: Usar Headless Browser com Playwright
- Mais resistente a bot protection
- Muito mais lento e pesado
- Requer instalação de browser headless

#### Opção 6: Desabilitar Temporariamente
- Se não for crítico, desabilitar o coletor
- Verificar se há alternativa (feed RSS, API, etc.)
- Documentar limitação

#### Opção 7: Contatar DeepSeek para API
- Verificar se oferecem API oficial
- Solicitar acesso a feed RSS

---

## Implementação Recomendada (Prioridade)

### Fase 1: Correções Simples (Imediato)
1. **Alibaba DAMO**: Converter para HTML collector
2. **Meta AI / DeepSeek**: Adicionar headers HTTP mais completos

### Fase 2: Melhorias Gerais (Curto Prazo)
1. Adicionar detecção de tipo de conteúdo (HTML vs RSS/Atom)
2. Adicionar retry logic com backoff exponencial
3. Melhorar mensagens de erro para debug

### Fase 3: Soluções Avançadas (Médio Prazo)
1. Implementar suporte a headless browser para sites com JS
2. Adicionar suporte a proxies rotativos
3. Implementar cache de respostas HTTP

---

## Código de Implementação Sugerida

### 1. Melhorar Headers HTTP (Meta AI / DeepSeek)

```rust
// Em html_collector.rs, método new()
pub fn new(temp_dir: PathBuf) -> Self {
    let mut headers = reqwest::header::HeaderMap::new();
    
    // User Agent
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );
    
    // Accept headers
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );
    
    // Security headers (para contornar bot protection)
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        reqwest::header::HeaderValue::from_static("none"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-mode"),
        reqwest::header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-user"),
        reqwest::header::HeaderValue::from_static("?1"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-dest"),
        reqwest::header::HeaderValue::from_static("document"),
    );
    
    // Referer
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static("https://www.google.com/"),
    );

    Self {
        client: Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .default_headers(headers)
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("Failed to create HTML client"),
        temp_dir,
    }
}
```

### 2. Detecção de Tipo de Conteúdo (RSS Collector)

```rust
// Em rss_collector.rs, método fetch_feed()
let feed_content = response.text().await?;

// Verificar se é realmente um feed RSS/Atom
let content_start = feed_content.trim_start();
if !content_start.starts_with("<?xml") 
    && !content_start.starts_with("<feed")
    && !content_start.starts_with("<rss") {
    warn!(
        url = %feed_url,
        "Feed URL returned HTML instead of RSS/Atom feed"
    );
    return Err(anyhow::anyhow!(
        "Feed URL '{}' returned HTML instead of RSS/Atom feed. This may not be a valid feed URL.",
        feed_url
    ));
}
```

### 3. Converter Alibaba DAMO para HTML

Atualizar `collectors_config.json`:
```json
{
  "id": "html_alibaba_damo",
  "name": "Alibaba DAMO News",
  "enabled": true,
  "collector_type": "html",
  "feed_url": null,
  "base_url": "https://damo.alibaba.com/news",
  "selectors": {
    "article": "article, .news-item, .post-item",
    "title": "h1, h2, .title, .post-title",
    "content": ".content, .post-content, article",
    "link": "a[href*='/news/']"
  },
  "config": {
    "max_results": 5
  }
}
```

E remover o coletor RSS antigo:
```json
{
  "id": "rss_alibaba_damo",
  "enabled": false
}
```

---

## Observações Importantes

1. **Respeitar robots.txt**: Sempre verificar e respeitar o arquivo `robots.txt` dos sites
2. **Rate Limiting**: Manter delays entre requisições para não sobrecarregar servidores
3. **User-Agent**: Usar User-Agent descritivo para identificação
4. **Legalidade**: Verificar termos de serviço antes de fazer scraping
5. **Alternativas**: Sempre verificar se há API oficial ou feed RSS antes de fazer scraping HTML

---

## Próximos Passos

1. Implementar correções da Fase 1
2. Testar cada correção isoladamente
3. Monitorar logs para identificar novos problemas
4. Documentar limitações conhecidas


























