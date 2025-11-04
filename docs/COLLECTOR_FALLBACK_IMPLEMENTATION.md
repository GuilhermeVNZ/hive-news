# Implementação de Fallback RSS → HTML → JavaScript

## Resumo

Implementamos um sistema de fallback para collectors de news que tenta múltiplas estratégias quando RSS falha:

1. **RSS Collector** (primeira tentativa)
   - Tenta buscar feed RSS/Atom normalmente
   
2. **HTML Collector com JavaScript** (fallback)
   - Se RSS falha com erro 404, 403, 308 (Redirect), tenta como HTML collector
   - Usa JavaScript rendering (Playwright) automaticamente se o collector estiver na lista de JS_COLLECTORS
   - Usa seletores genéricos para extrair links, títulos e conteúdo

## Mudanças Implementadas

### 1. Atualização de `needs_js_rendering` em `html_collector.rs`

Adicionados novos collectors que precisam de JavaScript rendering:
- `html_mistral_ai` (308 redirect)
- `html_character_ai` (308 redirect)
- `html_intel_ai` (403 Proibido)

### 2. Atualização de `fetch_full_article` em `html_collector.rs`

Adicionadas URLs que precisam de JavaScript rendering:
- `mistral.ai`
- `character.ai`
- `intel.com`

### 3. Fallback RSS → HTML em `main.rs`

Implementado fallback automático em `run_rss_collectors`:
- Quando RSS falha com erro 404, 403, 308 ou Redirect
- Tenta coletar usando HTML collector
- HTML collector automaticamente usa JavaScript rendering se necessário
- Mantém mesmo fluxo de processamento (registry, duplicates, destinations)

## Sites que se Beneficiam

### Sites com Redirect (308)
- **Perplexity AI Blog RSS**: `https://blog.perplexity.ai/feed` → Tenta HTML + JS
- **VentureBeat AI RSS**: `https://venturebeat.com/category/ai/feed/` → Tenta HTML + JS
- **Mistral AI News**: `https://mistral.ai/news/` → HTML collector já configurado para JS

### Sites com 403 (Proibido)
- **X.ai News**: HTML collector já configurado para JS
- **Intel AI Blog**: Agora adicionado à lista de JS_COLLECTORS

### Sites com 404 (Não encontrado)
- **ElevenLabs Blog RSS**: Tenta HTML + JS como fallback
- **IBM Research AI RSS**: Tenta HTML + JS como fallback
- **Wired AI RSS**: Tenta HTML + JS como fallback
- **MIT Technology Review AI RSS**: Tenta HTML + JS como fallback
- **Nature AI RSS**: Tenta HTML + JS como fallback

## Como Funciona

1. **RSS Collector tenta feed normalmente**
   ```rust
   match rss_collector.fetch_feed(feed_url, max_results, base_url).await {
       Ok(articles) => { /* processa normalmente */ }
       Err(e) => {
           // Verifica se erro é 404, 403, 308 ou Redirect
           if error_str.contains("404") || error_str.contains("403") || ... {
               // Tenta como HTML collector
               html_collector.fetch_page(feed_url, selectors, max_results, collector_id).await
           }
       }
   }
   ```

2. **HTML Collector detecta se precisa JS**
   - Verifica `needs_js_rendering(collector_id)`
   - Se sim, usa Playwright para renderizar JavaScript
   - Se não, usa requisição HTTP normal

3. **Processamento continua normalmente**
   - Mesmo fluxo de registry
   - Mesma lógica de duplicates
   - Mesmos destinations

## Benefícios

✅ **Automaticamente tenta fallback** quando RSS falha  
✅ **JavaScript rendering** para sites que precisam  
✅ **Transparente** - mesmo fluxo de processamento  
✅ **Robusto** - cobre mais casos de erro  

## Notas

- Alguns sites podem ainda falhar se tiverem proteção anti-bot muito robusta
- Sites com 500 (erro interno) não são tratados automaticamente - pode ser temporário
- Alguns sites podem precisar de URLs atualizadas manualmente



