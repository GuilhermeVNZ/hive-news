# Fluxo Completo: Featured e Hide/Show

## 📋 Resumo

Quando você marca/desmarca **Featured** ou **Hide/Show** no dashboard, o seguinte fluxo acontece:

---

## 🔄 FLUXO FEATURED

### 1. **Frontend - News Dashboard** 
**Arquivo:** `News-main/news-dashboard/src/pages/Logs.tsx`

**Linha ~161:** Quando você marca/desmarca o checkbox Featured:
```typescript
const response = await axios.put(`/api/logs/articles/${item.id}/featured`, 
  { featured: newValue },
  { timeout: 5000 }
);
```

**O que faz:** Envia requisição PUT para `/api/logs/articles/{id}/featured` com `{ featured: true/false }`

---

### 2. **Backend - Rotas API**
**Arquivo:** `News-main/news-backend/src/routes/logs.rs`

**Linha ~624:** Função `set_featured()` recebe a requisição:
```rust
pub async fn set_featured(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(body): Json<FeaturedUpdate>,
) -> Json<Value>
```

**O que faz:**
- Extrai o arXiv ID do `id` recebido
- Busca o artigo no filesystem para confirmar que existe
- Encontra o artigo correspondente no registry pelo arXiv ID
- Chama `manager.set_featured(&article_id_to_update, body.featured)`

---

### 3. **RegistryManager - Atualização**
**Arquivo:** `News-main/news-backend/src/utils/article_registry.rs`

**Linha ~507:** Função `set_featured()`:
```rust
pub fn set_featured(&self, article_id: &str, featured: bool) -> Result<()> {
    let mut registry = self.registry.lock().unwrap();
    if let Some(meta) = registry.articles.get_mut(article_id) {
        meta.featured = Some(featured);  // ← AQUI ATUALIZA O VALOR
        drop(registry);
        self.save()?;  // ← AQUI SALVA NO ARQUIVO
        Ok(())
    }
}
```

**O que faz:**
- Atualiza `meta.featured = Some(featured)` no registry em memória
- Chama `self.save()` para persistir no arquivo

---

### 4. **RegistryManager - Salvamento**
**Arquivo:** `News-main/news-backend/src/utils/article_registry.rs`

**Linha ~376:** Função `save()` do RegistryManager:
```rust
pub fn save(&self) -> Result<()> {
    let registry = self.registry.lock().unwrap();
    registry.save(&self.registry_path)  // ← Chama ArticleRegistry.save()
}
```

**Linha ~223:** Função `save()` do ArticleRegistry:
```rust
pub fn save(&self, registry_path: &Path) -> Result<()> {
    // Usa escrita atômica (tempfile + rename)
    let content = serde_json::to_string_pretty(self)?;
    let mut tmp = NamedTempFile::new_in(parent_dir)?;
    tmp.as_file_mut().write_all(content.as_bytes())?;
    tmp.as_file_mut().sync_all()?;  // ← Força flush ao disco
    tmp.persist(registry_path)?;  // ← Rename atômico
    Ok(())
}
```

**O que faz:**
- Serializa o registry completo para JSON
- Cria arquivo temporário
- Escreve conteúdo
- Faz flush ao disco (`sync_all()`)
- Faz rename atômico para `articles_registry.json`

**Arquivo atualizado:** `G:\Hive-Hub\News-main\articles_registry.json`

---

## 🔄 FLUXO HIDDEN

### 1. **Frontend - News Dashboard**
**Arquivo:** `News-main/news-dashboard/src/pages/Logs.tsx`

**Linha ~193:** Quando você clica no botão Hide/Show:
```typescript
const response = await axios.put(`/api/logs/articles/${item.id}/hidden`, 
  { hidden: newValue },
  { timeout: 5000 }
);
```

**O que faz:** Envia requisição PUT para `/api/logs/articles/{id}/hidden` com `{ hidden: true/false }`

---

### 2. **Backend - Rotas API**
**Arquivo:** `News-main/news-backend/src/routes/logs.rs`

**Linha ~417:** Função `set_hidden()` recebe a requisição:
```rust
pub async fn set_hidden(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(body): Json<HiddenUpdate>,
) -> Json<Value>
```

**O que faz:**
- Extrai o arXiv ID do `id` recebido
- Busca o artigo no filesystem para confirmar que existe
- Encontra o artigo correspondente no registry pelo arXiv ID
- Chama `manager.set_hidden(&article_id_to_update, body.hidden)`

---

### 3. **RegistryManager - Atualização**
**Arquivo:** `News-main/news-backend/src/utils/article_registry.rs`

**Linha ~512:** Função `set_hidden()`:
```rust
pub fn set_hidden(&self, article_id: &str, hidden: bool) -> Result<()> {
    let mut registry = self.registry.lock().unwrap();
    if let Some(meta) = registry.articles.get_mut(article_id) {
        meta.hidden = Some(hidden);  // ← AQUI ATUALIZA O VALOR
        drop(registry);
        self.save()?;  // ← AQUI SALVA NO ARQUIVO
        Ok(())
    }
}
```

**O que faz:**
- Atualiza `meta.hidden = Some(hidden)` no registry em memória
- Chama `self.save()` para persistir no arquivo

---

### 4. **RegistryManager - Salvamento**
**Mesmo processo do Featured** - usa `ArticleRegistry.save()` que salva em `articles_registry.json`

---

## 📁 ARQUIVO FINAL ATUALIZADO

**Arquivo:** `G:\Hive-Hub\News-main\articles_registry.json`

**Estrutura:**
```json
{
  "articles": {
    "2510.27258": {
      "id": "2510.27258",
      "title": "Higher-order Linear Attention",
      "status": "Published",
      "featured": true,   // ← AQUI (quando marca Featured)
      "hidden": false,    // ← AQUI (quando marca Hide)
      "destinations": ["airesearch"],
      ...
    }
  }
}
```

---

## 🔍 COMO O AIRESEARCH LÊ

**Arquivo:** `News-main/apps/frontend-next/airesearch/app/api/articles/route.ts`

**Linha ~156:** Lê o `articles_registry.json`:
```typescript
const registry = JSON.parse(registryContent);
if (registry.articles) {
  for (const [id, meta] of Object.entries(registry.articles)) {
    const metadata = meta as any;
    
    // Verifica se está publicado e tem destino AIResearch
    const isPublished = metadata.status === 'Published';
    const hasAIResearchDest = metadata.destinations && 
      metadata.destinations.some((d: string) => d.toLowerCase() === 'airesearch');
    
    if (isPublished && hasAIResearchDest) {
      // Verifica featured
      if (isTrue(metadata.featured)) {
        featuredMap.set(id, true);
      }
      
      // Verifica hidden
      if (isTrue(metadata.hidden)) {
        hiddenMap.set(id, true);
      }
    }
  }
}
```

**O que faz:**
- Lê `articles_registry.json`
- Para cada artigo que está `Published` e tem destino `AIResearch`:
  - Se `featured === true` → adiciona ao `featuredMap`
  - Se `hidden === true` → adiciona ao `hiddenMap`
- Depois cruza com artigos do filesystem e filtra

---

## ⚠️ POSSÍVEIS PROBLEMAS

### 1. **Arquivo não está sendo salvo**
- Verificar logs do backend: `[RegistryManager] ✅ Successfully saved registry`
- Verificar se `articles_registry.json` está sendo atualizado (timestamp)

### 2. **AIResearch não está lendo o arquivo atualizado**
- Cache do Next.js (já corrigido com `runtime='nodejs'`, `revalidate=0`, `dynamic='force-dynamic'`)
- Cache do sistema de arquivos do Node.js
- O arquivo pode estar sendo lido antes de ser salvo

### 3. **Mismatch de IDs**
- Registry usa ID: `2510.27258`
- Filesystem usa pasta: `2025-10-29_unknown_2510.27258`
- A extração do arXiv ID pode não estar funcionando corretamente

### 4. **Artigo não está no registry como Published**
- Verificar se `status === 'Published'`
- Verificar se `destinations.includes('airesearch')`

---

## ✅ CHECKLIST DE VALIDAÇÃO

1. **Verificar se o arquivo está sendo salvo:**
   ```powershell
   # Verificar timestamp do arquivo antes e depois de marcar Featured
   Get-Item "G:\Hive-Hub\News-main\articles_registry.json" | Select-Object LastWriteTime
   ```

2. **Verificar se o valor está no arquivo:**
   ```powershell
   $registry = Get-Content "G:\Hive-Hub\News-main\articles_registry.json" | ConvertFrom-Json
   $article = $registry.articles.PSObject.Properties | Where-Object { $_.Name -eq "2510.27258" } | Select-Object -First 1
   Write-Output "Featured: $($article.Value.featured)"
   Write-Output "Hidden: $($article.Value.hidden)"
   ```

3. **Verificar logs do backend:**
   - Procurar por `[RegistryManager] ✅ Successfully saved registry`
   - Procurar por `[set_featured] Updating featured status`
   - Procurar por `[set_hidden] Updating hidden status`

4. **Verificar logs do AIResearch:**
   - Procurar por `[AIResearch Articles API] Found X articles in registry for AIResearch`
   - Procurar por `[AIResearch Articles API] Sample registry IDs`
   - Procurar por `[AIResearch Articles API] Matched X articles`

---

## 📝 ARQUIVOS RESPONSÁVEIS

| Ação | Arquivo | Função | Linha |
|------|---------|--------|-------|
| **Frontend - Featured** | `news-dashboard/src/pages/Logs.tsx` | Checkbox onChange | ~161 |
| **Frontend - Hidden** | `news-dashboard/src/pages/Logs.tsx` | Button onClick | ~193 |
| **Backend - Featured** | `news-backend/src/routes/logs.rs` | `set_featured()` | ~624 |
| **Backend - Hidden** | `news-backend/src/routes/logs.rs` | `set_hidden()` | ~417 |
| **Atualização Featured** | `news-backend/src/utils/article_registry.rs` | `RegistryManager.set_featured()` | ~507 |
| **Atualização Hidden** | `news-backend/src/utils/article_registry.rs` | `RegistryManager.set_hidden()` | ~512 |
| **Salvamento** | `news-backend/src/utils/article_registry.rs` | `ArticleRegistry.save()` | ~223 |
| **Arquivo Final** | `articles_registry.json` | - | - |
| **Leitura AIResearch** | `apps/frontend-next/airesearch/app/api/articles/route.ts` | `readArticles()` | ~100 |

---

## 🎯 CONCLUSÃO

**Arquivo que atualiza para true/false:**
- `News-main/news-backend/src/utils/article_registry.rs`
  - Função `set_featured()` → atualiza `meta.featured = Some(featured)`
  - Função `set_hidden()` → atualiza `meta.hidden = Some(hidden)`
  - Função `save()` → salva em `articles_registry.json`

**Arquivo final atualizado:**
- `G:\Hive-Hub\News-main\articles_registry.json`

**Arquivo que lê para exibir:**
- `News-main/apps/frontend-next/airesearch/app/api/articles/route.ts`
  - Função `readArticles()` → lê `articles_registry.json` e cruza com filesystem


