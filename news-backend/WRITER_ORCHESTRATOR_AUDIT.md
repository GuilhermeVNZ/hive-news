# Writer Orchestrator - Auditoria de Funcionamento

## Data: 2025-10-30

### 1. Determinação do Site ✅

**Localização:** `src/main.rs` (linha 946-960)

**Lógica:**
1. Primeiro verifica variável de ambiente `WRITER_DEFAULT_SITE`
2. Se não encontrar, procura primeiro site `enabled` com `writer.enabled = true` no `system_config.json`
3. Fallback para `"airesearch"` se nada for encontrado

**Código:**
```rust
let site_id = std::env::var("WRITER_DEFAULT_SITE")
    .ok()
    .map(|s| s.to_lowercase())
    .or_else(|| {
        let config_manager = SiteConfigManager::new(Path::new("system_config.json"));
        if let Ok(sites) = config_manager.get_all_sites() {
            sites.iter()
                .find(|s| s.enabled && s.writer.enabled)
                .map(|s| s.id.clone())
        } else {
            None
        }
    })
    .unwrap_or_else(|| "airesearch".to_string());
```

**Status:** ✅ Funcionando corretamente

---

### 2. Carregamento de API Key ✅

**Localização:** `src/writer/content_generator.rs` (linha 52-54)

**Lógica:**
1. Primeiro tenta usar `writer_config.api_key` do `system_config.json` do site selecionado
2. Se não encontrar, usa variável de ambiente `DEEPSEEK_API_KEY`
3. Se nenhum for encontrado, retorna erro

**Código:**
```rust
let api_key = writer_config.api_key.clone()
    .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
    .context("API key not found in config or environment")?;
```

**Prioridade:**
1. ✅ `system_config.json` → `sites.{site_id}.writer.api_key`
2. ✅ `.env` → `DEEPSEEK_API_KEY` (via env var)
3. ❌ Erro se nenhum encontrado

**Status:** ✅ Funcionando corretamente

**Nota:** A sincronização automática do `.env` garante que quando você salva API key no frontend, ela vai para ambos `system_config.json` E `.env`

---

### 3. Carregamento de Prompts Customizados ✅

**Localização:** `src/writer/content_generator.rs` (linha 63-77)

**Lógica:**
1. Para cada tipo de prompt (article, social, blog):
   - Verifica se `prompt_{type}_enabled = true` no config
   - Se sim, carrega `prompt_{type}` customizado
   - Se não, retorna `None` (usa prompt padrão)

**Código:**
```rust
let prompt_article = if site_config.prompt_article_enabled.unwrap_or(false) {
    site_config.prompt_article.clone()
} else {
    None
};
```

**Status:** ✅ Funcionando corretamente

---

### 4. Uso dos Prompts na Geração ✅

**Localização:** `src/writer/content_generator.rs`

#### Prompt Article (linha 233-243)
**ANTES:** Sempre usava `build_article_prompt()` padrão
**AGORA:** ✅ Verifica se há prompt customizado antes de usar padrão

**Código:**
```rust
let article_prompt = if let Some(ref custom_prompt) = self.prompt_article {
    println!("  📝 Using custom article prompt from config");
    // Replace {{paper_text}} placeholder if present
    if custom_prompt.contains("{{paper_text}}") {
        custom_prompt.replace("{{paper_text}}", &parsed.text)
    } else {
        format!("{}\n\n## PAPER TEXT (YOUR ONLY SOURCE):\n{}", custom_prompt, &parsed.text)
    }
} else {
    build_article_prompt(&parsed.text, &[], &self.site)
};
```

**Status:** ✅ **CORRIGIDO** - Agora usa prompt customizado quando habilitado

#### Prompt Social (linha 259-276)
**Status:** ✅ Já estava funcionando corretamente
- Usa prompt customizado se `prompt_social_enabled = true`
- Usa prompt padrão caso contrário
- Suporta placeholders `{{article_text}}` e `{{paper_title}}`

---

### 5. Fluxo Completo de Execução ✅

```
run_writer_pipeline()
  ↓
1. Determina site_id (env var OU primeiro enabled OU "airesearch")
  ↓
2. WriterService::new_with_site(Some(&site_id))
  ↓
3. Carrega config do site do system_config.json:
   - API key (prioridade: config → env var)
   - Base URL
   - Model
   - Prompts customizados (se enabled)
  ↓
4. Para cada PDF filtrado:
   - process_pdf()
     - Usa prompt customizado SE enabled
     - Usa prompt padrão SE não enabled
     - Usa API key do site configurado
```

**Status:** ✅ Funcionando corretamente após correção do prompt article

---

## Resumo de Status

| Componente | Status | Observações |
|------------|--------|-------------|
| Determinação do Site | ✅ | Usa env var, config ou default |
| API Key Loading | ✅ | Prioridade: config.json → .env |
| Prompt Loading | ✅ | Carrega customizado se enabled |
| Uso Prompt Article | ✅ | **CORRIGIDO** - Agora usa customizado |
| Uso Prompt Social | ✅ | Já estava funcionando |
| Sincronização .env | ✅ | Automática após salvar config |

---

## Correções Aplicadas

1. ✅ **Prompt Article Customizado**: Agora verifica `self.prompt_article` antes de usar prompt padrão
2. ✅ **Placeholder Support**: Suporta `{{paper_text}}` no prompt article customizado

---

## Recomendações

1. ✅ Tudo funcionando corretamente após correções
2. ⚠️  Lembre-se de habilitar o prompt (`prompt_article_enabled = true`) no frontend para usar customizado
3. ✅ API key é sincronizada automaticamente do config para `.env`












































