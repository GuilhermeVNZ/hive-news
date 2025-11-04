# Integração Dashboard ↔ system_config.json

Este documento descreve como todas as opções selecionadas no dashboard se comunicam com o arquivo `system_config.json`.

## Estrutura do system_config.json

```json
{
  "sites": {
    "airesearch": {
      "id": "airesearch",
      "name": "AI Research",
      "domain": "airesearch.news",
      "enabled": true,
      "collectors": [...],
      "writer": {...},
      "education_sources": [...],
      "social_media": [...],
      "collection_frequency_minutes": 60,
      "writing_style": "scientific",
      "prompt_article": "...",
      "prompt_social": "...",
      "prompt_blog": "...",
      "prompt_article_enabled": true,
      "prompt_social_enabled": false,
      "prompt_blog_enabled": false
    },
    "scienceai": {...}
  },
  "updated_at": "..."
}
```

## Rotas da API e Integração

### 1. Sites (/api/sites)

#### GET `/api/sites`
- **Arquivo**: `src/routes/sites.rs` → `get_all_sites()`
- **Ação**: Lê todos os sites do `system_config.json`
- **Salva**: ❌ (apenas leitura)

#### GET `/api/sites/:site_id`
- **Arquivo**: `src/routes/sites.rs` → `get_site_config()`
- **Ação**: Lê configuração de um site específico
- **Salva**: ❌ (apenas leitura)

#### PUT `/api/sites/:site_id/writer`
- **Arquivo**: `src/routes/sites.rs` → `update_writer_config()`
- **Ação**: Atualiza configuração do Writer para um site
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `writer.provider`
  - `writer.model`
  - `writer.api_key`
  - `writer.base_url`
  - `writer.temperature`
  - `writer.max_tokens`
  - `writer.enabled`
  - `writer.use_compressor`
  - `site.writing_style`
  - `site.prompt_article`
  - `site.prompt_social`
  - `site.prompt_blog`
  - `site.prompt_article_enabled`
  - `site.prompt_social_enabled`
  - `site.prompt_blog_enabled`

#### PUT `/api/sites/:site_id/collectors/:collector_id/status`
- **Arquivo**: `src/routes/sites.rs` → `update_collector_status()`
- **Ação**: Atualiza status (enabled/disabled) de um collector para um site
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.collectors[].enabled`

#### PUT `/api/sites/:site_id/social/:social_id/status`
- **Arquivo**: `src/routes/sites.rs` → `update_social_status()`
- **Ação**: Atualiza status de social media para um site
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.social_media[].enabled`

#### PUT `/api/sites/:site_id/social/:social_id/config`
- **Arquivo**: `src/routes/sites.rs` → `update_social_config()`
- **Ação**: Atualiza configuração completa de social media
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.social_media[].enabled`
  - `site.social_media[].api_key`
  - `site.social_media[].api_secret`
  - `site.social_media[].access_token`
  - `site.social_media[].refresh_token`
  - `site.social_media[].channel_id`
  - `site.social_media[].username`
  - `site.social_media[].config`

#### PUT `/api/sites/:site_id/education/:source_id/status`
- **Arquivo**: `src/routes/sites.rs` → `update_education_status()`
- **Ação**: Atualiza status de education source para um site
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.education_sources[].enabled`

#### PUT `/api/sites/:site_id/education/:source_id/config`
- **Arquivo**: `src/routes/sites.rs` → `update_education_config()`
- **Ação**: Atualiza configuração completa de education source
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.education_sources[].enabled`
  - `site.education_sources[].api_key`
  - `site.education_sources[].config`

### 2. Collectors (/api/collectors)

#### GET `/api/collectors`
- **Arquivo**: `src/routes/collectors.rs` → `get_collectors()`
- **Ação**: Lê todos os collectors de todos os sites
- **Salva**: ❌ (apenas leitura)
- **Nota**: Consolida collectors de todos os sites do `system_config.json`

#### PUT `/api/collectors/:id/status`
- **Arquivo**: `src/routes/collectors.rs` → `update_collector_status()`
- **Ação**: Atualiza status do collector em TODOS os sites
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `sites[].collectors[].enabled` (em todos os sites onde o collector existe)

#### PUT `/api/collectors/:id/sites`
- **Arquivo**: `src/routes/collectors.rs` → `update_collector_sites()`
- **Ação**: Atualiza quais sites usam este collector (destinations)
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `sites[].collectors[].destinations` (array de site IDs)
  - `sites[].collectors[].enabled` (habilitado se site está em destinations)

### 3. Pages (/api/pages)

#### GET `/api/pages`
- **Arquivo**: `src/routes/pages.rs` → `list_pages()`
- **Ação**: Lista todas as páginas (sites)
- **Salva**: ❌ (apenas leitura)

#### POST `/api/pages`
- **Arquivo**: `src/routes/pages.rs` → `create_page()`
- **Ação**: Cria ou atualiza um site (page)
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.id`
  - `site.name`
  - `site.domain`
  - `site.enabled`
  - `site.collection_frequency_minutes`
  - `site.writing_style`

#### PUT `/api/pages/:id`
- **Arquivo**: `src/routes/pages.rs` → `update_page()`
- **Ação**: Atualiza configuração básica de um site
- **Salva**: ✅ `system_config.json`
- **Campos salvos**:
  - `site.collection_frequency_minutes`
  - `site.writing_style`
  - `site.enabled`

## Páginas do Dashboard

### 1. Sites (`/sites`)
- **Arquivo**: `news-dashboard/src/pages/Sites.tsx`
- **Rotas usadas**:
  - `GET /api/sites` - Carregar sites
  - `PUT /api/sites/:site_id/writer` - Salvar prompts e configurações de writer
- **Configurações salvas**:
  - ✅ `prompt_article` e `prompt_article_enabled`
  - ✅ `prompt_social` e `prompt_social_enabled`
  - ✅ `prompt_blog` e `prompt_blog_enabled`

### 2. Writer (`/writer`)
- **Arquivo**: `news-dashboard/src/pages/Writer.tsx`
- **Rotas usadas**:
  - `GET /api/sites` - Carregar sites
  - `PUT /api/sites/:site_id/writer` - Configurar provider e API key
- **Configurações salvas**:
  - ✅ `writer.provider`
  - ✅ `writer.api_key`
  - ✅ `writer.enabled` (implícito ao selecionar sites)

### 3. Sources (`/sources`)
- **Arquivo**: `news-dashboard/src/pages/Sources.tsx`
- **Rotas usadas**:
  - `GET /api/collectors` - Carregar collectors
  - `GET /api/sites` - Carregar sites
  - `PUT /api/collectors/:id/status` - Habilitar/desabilitar collector
  - `PUT /api/collectors/:id/sites` - Selecionar sites destino
- **Configurações salvas**:
  - ✅ `collectors[].enabled` (em todos os sites)
  - ✅ `collectors[].destinations` (quais sites usam o collector)

### 4. Educational (`/educational`)
- **Arquivo**: `news-dashboard/src/pages/Educational.tsx`
- **Rotas usadas**:
  - `GET /api/sites` - Carregar sites
  - `PUT /api/sites/:site_id/education/:source_id/status` - Habilitar/desabilitar
  - `PUT /api/sites/:site_id/education/:source_id/config` - Configurar API key
- **Configurações salvas**:
  - ✅ `education_sources[].enabled`
  - ✅ `education_sources[].api_key`
  - ✅ `education_sources[].config`

### 5. Dashboard (`/dashboard`)
- **Arquivo**: `news-dashboard/src/pages/Dashboard.tsx`
- **Rotas usadas**:
  - `GET /api/sites` - Carregar sites
  - `GET /api/logs` - Carregar logs
  - `GET /api/system/status` - Status do sistema
  - `POST /api/sites/:site_id/collect/start` - Iniciar coleta
- **Salva**: ❌ (apenas leitura)

## Função de Salvamento

Todas as rotas que salvam configuração utilizam:

```rust
// src/utils/site_config_manager.rs
impl SiteConfigManager {
    pub fn update_site_config(&self, site_id: &str, site_config: SiteConfig) -> Result<()> {
        let mut config = self.load()?;  // Carrega do system_config.json
        config.sites.insert(site_id.to_string(), site_config);
        self.save(&config)  // Salva de volta no system_config.json
    }
    
    pub fn save(&self, config: &SystemConfig) -> Result<()> {
        config.updated_at = chrono::Utc::now().to_rfc3339();
        let content = serde_json::to_string_pretty(&config)
            .context("Failed to serialize config")?;
        fs::write(&self.config_path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}
```

## Checklist de Integração

### ✅ Funcionalidades que salvam no system_config.json

- [x] **Sites**: Configuração de prompts (article, social, blog)
- [x] **Writer**: Provider, API key, modelo
- [x] **Collectors**: Status (enabled/disabled) e destinations (sites)
- [x] **Educational**: Status e API keys
- [x] **Social Media**: Status e configurações
- [x] **Pages**: Frequency, writing_style, enabled

### ❌ Funcionalidades que NÃO salvam (apenas leitura)

- [ ] Dashboard: Apenas visualização
- [ ] Logs: Apenas visualização
- [ ] System Status: Apenas visualização

## Notas Importantes

1. **Destinations**: O campo `destinations` no `CollectorConfig` indica quais sites devem receber conteúdo deste collector. Quando você seleciona sites no dashboard Sources, isso atualiza o campo `destinations` em todos os sites.

2. **Writer Sync**: Após atualizar configuração do Writer, o sistema sincroniza automaticamente o arquivo `.env` (se a função `env_sync::sync_env_from_config` estiver disponível).

3. **Collector Status Global**: Quando você habilita/desabilita um collector na página Sources, isso atualiza o status em TODOS os sites onde o collector existe.

4. **Sites Assignment**: A página Sources permite selecionar quais sites usam cada collector. Isso salva no campo `destinations` e também atualiza o `enabled` baseado se o site está na lista de destinations.

## Próximos Passos

Se você encontrar alguma configuração que não está sendo salva:

1. Verifique se a rota está usando `SiteConfigManager::update_site_config()`
2. Verifique se o campo está incluído na estrutura `SiteConfig`
3. Verifique se o endpoint está fazendo `PUT` (não apenas `GET`)
4. Verifique os logs do backend para erros de serialização





