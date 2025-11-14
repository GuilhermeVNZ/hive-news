# Relatório Completo: Problema com Botões Hide/Show e Featured

## 📋 Resumo Executivo

**Problema:** Os botões "Hide/Show" e "Featured" no News Dashboard não estão atualizando corretamente os artigos no frontend AIResearch. Quando um artigo é marcado como "Featured" ou "Hidden", as mudanças não aparecem na página principal do AIResearch.

**Data:** 2025-01-26
**Status:** Em investigação
**Severidade:** Alta - Funcionalidade crítica não está funcionando

---

## 🎯 Comportamento Esperado vs. Observado

### Comportamento Esperado

1. **Botão "Featured" (Checkbox):**
   - Usuário marca checkbox "Featured" no artigo nos logs
   - Backend atualiza `articles_registry.json` com `featured: true`
   - AIResearch frontend lê o registry e identifica artigos featured
   - Artigo aparece destacado na primeira página do AIResearch

2. **Botão "Hide/Show":**
   - Usuário clica em "Hide" no artigo nos logs
   - Backend atualiza `articles_registry.json` com `hidden: true`
   - AIResearch frontend filtra artigos com `hidden: true`
   - Artigo desaparece da página principal do AIResearch

### Comportamento Observado

1. **Botão "Featured":**
   - ✅ Checkbox muda visualmente no dashboard
   - ✅ Backend parece processar a requisição (retorna sucesso)
   - ❌ Artigo NÃO aparece destacado na página principal do AIResearch
   - ❌ Status "Featured" não persiste após refresh

2. **Botão "Hide/Show":**
   - ✅ Botão muda de "Hide" para "Show" no dashboard
   - ✅ Backend parece processar a requisição (retorna sucesso)
   - ❌ Artigo NÃO desaparece da página principal do AIResearch
   - ❌ Status "Hidden" não persiste após refresh

---

## 🏗️ Arquitetura do Sistema

### Componentes Envolvidos

```
┌─────────────────────────────────────────────────────────────┐
│                    News Dashboard (Frontend)                 │
│  - Componente: Logs.tsx                                     │
│  - Endpoints: /api/logs/articles/{id}/featured             │
│              /api/logs/articles/{id}/hidden                │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP PUT
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              News Backend (Rust/Axum)                       │
│  - Arquivo: src/routes/logs.rs                              │
│  - Funções: set_featured(), set_hidden()                     │
│  - Manager: RegistryManager                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ Atualiza
                       ▼
┌─────────────────────────────────────────────────────────────┐
│           articles_registry.json                           │
│  - Formato: { "articles": { "id": { metadata } } }        │
│  - Campos: featured (bool), hidden (bool)                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ Lê
                       ▼
┌─────────────────────────────────────────────────────────────┐
│         AIResearch Frontend (Next.js)                      │
│  - Arquivo: app/api/articles/route.ts                       │
│  - Função: readArticles()                                   │
│  - Lê registry e filtra artigos                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔍 Análise Detalhada do Código

### 1. Frontend (News Dashboard) - Logs.tsx

**Localização:** `News-main/news-dashboard/src/pages/Logs.tsx`

**Código do botão Featured:**
```typescript
onChange={async(e)=>{
  try{
    const newValue = e.target.checked;
    // Otimistic update - atualizar UI imediatamente
    setItems(prev => prev.map(it => 
      it.id === item.id ? { ...it, featured: newValue } : it
    ));
    
    const response = await axios.put(`/api/logs/articles/${item.id}/featured`, 
      { featured: newValue },
      { timeout: 5000 }
    );
    // Verify response is successful
    if (!response.data?.success) {
      throw new Error(response.data?.error || 'Update failed');
    }
  } catch(e:any){
    // Reverter otimistic update em caso de erro
    setItems(prev => prev.map(it => 
      it.id === item.id ? { ...it, featured: item.featured } : it
    ));
    setError(e.response?.data?.error || e.message || 'Failed to update featured status');
  }
}}
```

**Observações:**
- ✅ Implementa optimistic update
- ✅ Faz requisição PUT para `/api/logs/articles/{id}/featured`
- ✅ Envia `{ featured: newValue }` no body
- ✅ Tem tratamento de erro com rollback

**Código do botão Hide/Show:**
```typescript
onClick={async()=>{
  try{
    const newValue = !item.hidden;
    // Otimistic update
    setItems(prev => prev.map(it => 
      it.id === item.id ? { ...it, hidden: newValue } : it
    ));
    
    const response = await axios.put(`/api/logs/articles/${item.id}/hidden`, 
      { hidden: newValue },
      { timeout: 5000 }
    );
    // Verify response is successful
    if (!response.data?.success) {
      throw new Error(response.data?.error || 'Update failed');
    }
  } catch(e:any){
    // Reverter otimistic update
    setItems(prev => prev.map(it => 
      it.id === item.id ? { ...it, hidden: item.hidden } : it
    ));
    setError(e.response?.data?.error || e.message || 'Failed to update hidden status');
  }
}}
```

**Observações:**
- ✅ Implementa optimistic update
- ✅ Faz requisição PUT para `/api/logs/articles/{id}/hidden`
- ✅ Envia `{ hidden: newValue }` no body
- ✅ Tem tratamento de erro com rollback

---

### 2. Backend (Rust) - logs.rs

**Localização:** `News-main/news-backend/src/routes/logs.rs`

#### Função `set_featured`:

```rust
pub async fn set_featured(
    Extension(_db): Extension<std::sync::Arc<Database>>,
    Path(id): Path<String>,
    Json(body): Json<FeaturedUpdate>,
) -> Json<Value> {
    // Validar ID
    if id.is_empty() {
        return Json(serde_json::json!({"success": false, "error": "Article ID is required"}));
    }

    let registry_path = get_registry_path();
    
    // Criar manager thread-safe (usa Mutex internamente)
    let manager = match RegistryManager::new(&registry_path) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load registry: {}", e);
            return Json(serde_json::json!({"success": false, "error": format!("Failed to load registry: {}", e)}));
        },
    };
    
    // Helper function to extract arXiv ID (same as in list_logs)
    fn extract_arxiv_id(id: &str) -> Option<String> {
        if let Some(captures) = regex::Regex::new(r"(\d{4}\.\d{4,6})").ok() {
            if let Some(mat) = captures.find(id) {
                return Some(mat.as_str().to_string());
            }
        }
        if id.matches('.').count() == 1 && id.len() >= 9 && id.len() <= 12 {
            if let Some(_) = id.find('.') {
                return Some(id.to_string());
            }
        }
        None
    }
    
    // Try to find article by matching title from filesystem
    // This ensures we update the correct article even if titles differ
    let arxiv_id = extract_arxiv_id(&id);
    let mut found_id: Option<String> = None;
    
    // Search for article in filesystem to get actual title, then find matching registry entry
    let site_dirs = vec![
        FsPath::new("G:/Hive-Hub/News-main/output/AIResearch"),
        FsPath::new("G:/Hive-Hub/News-main/output/ScienceAI"),
    ];
    
    let mut actual_title_from_fs: Option<String> = None;
    
    if let Some(ref arxiv) = arxiv_id {
        for site_output_dir in site_dirs {
            if let Ok(entries) = fs::read_dir(site_output_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let folder_name = entry.file_name().to_string_lossy().to_string();
                        if folder_name.contains(arxiv) {
                            let folder_path = entry.path();
                            if folder_path.is_dir() {
                                let title_txt = folder_path.join("title.txt");
                                if title_txt.exists() {
                                    if let Ok(title_content) = fs::read_to_string(&title_txt) {
                                        actual_title_from_fs = Some(title_content.trim().to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if actual_title_from_fs.is_some() {
                break;
            }
        }
    }
    
    // If we found the title from filesystem, try to find matching article in registry
    // by comparing with the actual title from filesystem
    if let Some(ref _fs_title) = actual_title_from_fs {
        if let Some(ref arxiv) = arxiv_id {
            let all_articles = manager.get_all_articles();
            for article in all_articles {
                // Extract arXiv ID from registry article ID
                let reg_arxiv_id = extract_arxiv_id(&article.id);
                if reg_arxiv_id == arxiv_id {
                    // Found matching arXiv ID, verify by checking title in filesystem
                    // We already have the title from filesystem, so use the registry ID
                    found_id = Some(article.id.clone());
                    eprintln!("[set_featured] Found article by arXiv ID: {} -> registry ID: {}", arxiv, article.id);
                    break;
                }
            }
        }
    }
    
    // Use found_id if available, otherwise fall back to provided id
    let article_id_to_update = found_id.unwrap_or(id);
    
    eprintln!("[set_featured] Updating featured status: article_id={}, featured={}, fs_title={:?}", 
        article_id_to_update, body.featured, actual_title_from_fs);
    
    // Usar método thread-safe do RegistryManager
    match manager.set_featured(&article_id_to_update, body.featured) {
        Ok(_) => {
            tracing::info!("Successfully updated featured status for article: {} (fs_title: {:?})", 
                article_id_to_update, actual_title_from_fs);
            Json(serde_json::json!({"success": true}))
        },
        Err(e) => {
            tracing::error!("Failed to update featured status for article {}: {}", article_id_to_update, e);
            Json(serde_json::json!({"success": false, "error": format!("{}", e)}))
        }
    }
}
```

**Observações:**
- ✅ Usa `get_registry_path()` para encontrar o registry
- ✅ Extrai arXiv ID do ID recebido
- ✅ Busca no filesystem para encontrar o artigo correto
- ✅ Tenta encontrar o artigo no registry pelo arXiv ID
- ✅ Chama `manager.set_featured()` que deve salvar automaticamente

**Função `set_hidden`:** Similar à `set_featured`, mas chama `manager.set_hidden()`.

---

### 3. RegistryManager - article_registry.rs

**Localização:** `News-main/news-backend/src/utils/article_registry.rs`

#### Função `set_featured`:

```rust
pub fn set_featured(&self, article_id: &str, featured: bool) -> Result<()> {
    eprintln!("[RegistryManager] set_featured called: article_id={}, featured={}", article_id, featured);
    eprintln!("[RegistryManager] Registry path: {:?}", self.registry_path);
    let mut registry = self.registry.lock().unwrap();
    if let Some(meta) = registry.articles.get_mut(article_id) {
        eprintln!("[RegistryManager] Found article, old featured value: {:?}", meta.featured);
        meta.featured = Some(featured);
        eprintln!("[RegistryManager] Updated featured to: {:?}", meta.featured);
        drop(registry); // Liberar lock antes de salvar
        match self.save() {
            Ok(_) => {
                eprintln!("[RegistryManager] ✅ Successfully saved registry with featured={} for article {}", featured, article_id);
                Ok(())
            },
            Err(e) => {
                eprintln!("[RegistryManager] ❌ Failed to save registry: {}", e);
                Err(e)
            }
        }
    } else {
        drop(registry);
        eprintln!("[RegistryManager] ❌ Article '{}' not found in registry", article_id);
        Err(anyhow::anyhow!("Article with ID '{}' not found", article_id))
    }
}
```

**Observações:**
- ✅ Usa Mutex para thread-safety
- ✅ Busca artigo pelo ID no registry
- ✅ Atualiza `meta.featured = Some(featured)`
- ✅ Chama `self.save()` para persistir no arquivo
- ✅ Tem logs de debug extensivos

**Função `save()`:**
```rust
pub fn save(&self) -> Result<()> {
    let registry = self.registry.lock().unwrap();
    let registry_path = &self.registry_path;
    
    // Serializar para JSON
    let mut map = HashMap::new();
    for (id, meta) in registry.articles.iter() {
        map.insert(id.clone(), meta.clone());
    }
    
    let registry_struct = ArticleRegistry { articles: map };
    
    // Salvar no arquivo
    let content = serde_json::to_string_pretty(&registry_struct)
        .context("Failed to serialize registry")?;
    
    std::fs::write(registry_path, content)
        .context(format!("Failed to write registry to {:?}", registry_path))?;
    
    Ok(())
}
```

**Observações:**
- ✅ Serializa o registry para JSON
- ✅ Escreve no arquivo usando `std::fs::write`
- ✅ Deveria sobrescrever o arquivo completamente

---

### 4. AIResearch Frontend - route.ts

**Localização:** `News-main/apps/frontend-next/airesearch/app/api/articles/route.ts`

#### Lendo o registry:

```typescript
// Ler registry para verificar featured status
const featuredMap = new Map<string, boolean>();
try {
  const possiblePaths = [
    path.join(process.cwd(), '../../../../articles_registry.json'),
    path.join(process.cwd(), '../../../articles_registry.json'),
    path.join(process.cwd(), '../articles_registry.json'),
    path.resolve('G:/Hive-Hub/News-main/articles_registry.json'),
  ];
  
  let registryPath: string | null = null;
  let registryContent: string = '';
  
  // Tentar encontrar o registry
  for (const testPath of possiblePaths) {
    try {
      await fs.access(testPath);
      registryPath = testPath;
      registryContent = await fs.readFile(testPath, 'utf-8');
      console.log(`[AIResearch Articles API] Reading registry from: ${testPath}`);
      break;
    } catch (err) {
      continue;
    }
  }
  
  if (!registryPath || !registryContent) {
    console.warn('[AIResearch Articles API] ⚠️  Registry not found in any of the expected paths.');
  } else {
    const registry = JSON.parse(registryContent);
    if (registry.articles) {
      let featuredFound = 0;
      for (const [id, meta] of Object.entries(registry.articles)) {
        const metadata = meta as any;
        if (metadata.featured === true) {
          featuredMap.set(id, true);
          featuredFound++;
          console.log(`[AIResearch Articles API] Found featured article in registry: ${id}`);
        }
      }
      console.log(`[AIResearch Articles API] Total featured articles in registry: ${featuredFound}`);
    }
  }
} catch (err: any) {
  console.error('[AIResearch Articles API] ⚠️  Error reading registry:', err?.message || err);
}
```

**Observações:**
- ✅ Tenta múltiplos caminhos para encontrar o registry
- ✅ Lê o arquivo `articles_registry.json`
- ✅ Itera sobre `registry.articles` e procura `metadata.featured === true`
- ✅ Armazena no `featuredMap` usando o ID do registry como chave

#### Aplicando featured aos artigos:

```typescript
// Função para extrair arXiv ID do nome da pasta
function extractArxivId(folderName: string): string {
  const arxivIdMatch = folderName.match(/\d{4}\.\d{4,6}/);
  if (arxivIdMatch) {
    return arxivIdMatch[0];
  }
  // ... mais lógica de extração
  return folderName;
}

// Adicionar campo featured aos artigos
let featuredCount = 0;
for (const article of allArticles) {
  // Extrair arXiv ID do nome da pasta (article.id)
  const arxivId = extractArxivId(article.id);
  
  // Tentar buscar no registry usando o arXiv ID extraído
  // Primeiro tenta com o ID completo, depois com o arXiv ID extraído
  let featured = featuredMap.get(article.id) === true;
  if (!featured) {
    featured = featuredMap.get(arxivId) === true;
  }
  
  (article as any).featured = featured;
  if (featured) {
    featuredCount++;
    console.log(`[AIResearch Articles API] ✓ Article ${article.id} (arXiv ID: ${arxivId}) "${article.title.substring(0, 50)}" is FEATURED`);
  }
}
```

**Observações:**
- ✅ Extrai arXiv ID do nome da pasta (ex: `2510.27258` de `2025-10-29_unknown_2510.27258`)
- ✅ Tenta buscar no `featuredMap` usando:
  1. `article.id` (nome completo da pasta)
  2. `arxivId` (extraído)
- ⚠️ **PROBLEMA POTENCIAL:** O `featuredMap` usa o ID do registry como chave, mas o `article.id` pode ser diferente!

**IMPORTANTE:** O `featuredMap` é populado com os IDs do registry (ex: `2510.27258`), mas o `article.id` pode ser o nome da pasta (ex: `2025-10-29_unknown_2510.27258`). A função `extractArxivId` tenta resolver isso, mas pode haver inconsistências.

---

## 🔧 Tentativas de Resolução

### Tentativa 1: Corrigir caminho do registry
**Problema:** `set_hidden` estava usando caminho fixo `"../articles_registry.json"`  
**Solução:** Implementado `get_registry_path()` para encontrar o registry corretamente  
**Resultado:** ❌ Não resolveu o problema

### Tentativa 2: Remover updates duplicados
**Problema:** Frontend estava fazendo update duplicado após resposta  
**Solução:** Removido update duplicado, mantido apenas optimistic update  
**Resultado:** ❌ Não resolveu o problema

### Tentativa 3: Buscar artigo pelo arXiv ID
**Problema:** Backend pode não estar encontrando o artigo correto no registry  
**Solução:** Implementada lógica para extrair arXiv ID e buscar artigo correspondente no registry  
**Resultado:** ❌ Não resolveu o problema

### Tentativa 4: Verificar se registry está sendo salvo
**Problema:** Registry pode não estar sendo salvo corretamente  
**Verificação:** Logs mostram que `save()` é chamado, mas não confirmamos se o arquivo é realmente atualizado  
**Status:** ⏳ Pendente de verificação

---

## 🔴 Problemas Identificados

### 1. **Mismatch de IDs entre Registry e Filesystem**

**Problema:**
- Registry usa IDs como `2510.27258` (arXiv ID direto)
- Filesystem usa pastas como `2025-10-29_unknown_2510.27258` (formato com data)
- AIResearch lê pastas do filesystem, então `article.id` = nome da pasta
- `featuredMap` usa IDs do registry como chave

**Impacto:**
- Quando o backend atualiza `articles_registry.json` com ID `2510.27258`
- AIResearch tenta buscar usando `article.id` (nome da pasta) ou `arxivId` extraído
- Se a extração do arXiv ID falhar ou houver inconsistência, o artigo não será encontrado

**Evidência:**
```typescript
// Backend salva no registry com ID: "2510.27258"
featuredMap.set("2510.27258", true);

// AIResearch tenta buscar:
let featured = featuredMap.get("2025-10-29_unknown_2510.27258") === true; // ❌ Não encontra
if (!featured) {
  featured = featuredMap.get("2510.27258") === true; // ✅ Deveria encontrar, mas pode falhar
}
```

### 2. **Registry pode não estar sendo salvo corretamente**

**Problema:**
- `RegistryManager.save()` usa `std::fs::write()` que deveria sobrescrever o arquivo
- Mas não há verificação se o arquivo foi realmente escrito
- Pode haver problemas de permissão ou lock do arquivo

**Verificação necessária:**
- Confirmar se o arquivo `articles_registry.json` está sendo atualizado após `set_featured()` ou `set_hidden()`
- Verificar se há erros silenciosos no `save()`

### 3. **Cache no Next.js**

**Problema:**
- Next.js pode estar fazendo cache do registry
- API route pode estar retornando dados em cache

**Solução tentada:**
- Headers `Cache-Control: no-store, no-cache` já estão implementados
- Mas pode haver cache no nível do sistema de arquivos do Node.js

### 4. **Falta de verificação de Hidden**

**Problema:**
- AIResearch não está filtrando artigos com `hidden: true`
- O código lê o registry para `featured`, mas não verifica `hidden`

**Código atual:**
```typescript
// Só verifica featured, não verifica hidden!
for (const [id, meta] of Object.entries(registry.articles)) {
  const metadata = meta as any;
  if (metadata.featured === true) {
    featuredMap.set(id, true);
  }
  // ❌ Não verifica metadata.hidden!
}
```

---

## 🧪 Testes Realizados

### Teste 1: Verificar se registry está sendo atualizado
**Comando:**
```powershell
cd G:\Hive-Hub\News-main
$registry = Get-Content "articles_registry.json" | ConvertFrom-Json
$sample = $registry.articles.PSObject.Properties | Where-Object { $_.Value.title -like "*Attention*Gets*" }
```

**Resultado:**
```
ID: 2510.27258
Title: Higher-order Linear Attention
Featured: False
Hidden: False
```

**Conclusão:** ❌ Registry NÃO está sendo atualizado após clicar nos botões

### Teste 2: Verificar logs do backend
**Status:** ⏳ Não foi verificado se há erros nos logs do servidor

---

## 💡 Hipóteses

### Hipótese 1: Registry não está sendo salvo
**Probabilidade:** Alta  
**Razão:** Teste mostrou que `Featured: False` e `Hidden: False` mesmo após clicar nos botões  
**Próximo passo:** Verificar logs do backend e confirmar se `save()` está sendo chamado com sucesso

### Hipótese 2: ID mismatch entre backend e frontend
**Probabilidade:** Média  
**Razão:** Backend pode estar atualizando um ID diferente do que o AIResearch está procurando  
**Próximo passo:** Verificar qual ID está sendo usado no backend vs. qual ID o AIResearch está procurando

### Hipótese 3: Problema de permissão de arquivo
**Probabilidade:** Baixa  
**Razão:** Arquivo pode estar bloqueado ou sem permissão de escrita  
**Próximo passo:** Verificar permissões do arquivo `articles_registry.json`

### Hipótese 4: Cache do Next.js
**Probabilidade:** Baixa  
**Razão:** Headers de cache já estão implementados  
**Próximo passo:** Reiniciar servidor Next.js e limpar cache

---

## 📝 Próximos Passos Recomendados

1. **Verificar logs do backend:**
   - Verificar se `set_featured()` e `set_hidden()` estão sendo chamados
   - Verificar se `RegistryManager.save()` está sendo executado com sucesso
   - Verificar se há erros silenciosos

2. **Verificar se o registry está sendo atualizado:**
   - Ler `articles_registry.json` antes e depois de clicar nos botões
   - Confirmar se o arquivo está sendo modificado

3. **Adicionar logging mais detalhado:**
   - Logar o ID exato que está sendo usado para atualizar
   - Logar o ID que o AIResearch está procurando
   - Comparar os dois para identificar mismatch

4. **Implementar verificação de Hidden no AIResearch:**
   - Adicionar lógica para filtrar artigos com `hidden: true`
   - Similar ao que foi feito para `featured`

5. **Testar com um artigo específico:**
   - Escolher um artigo conhecido
   - Verificar qual é o ID no registry
   - Verificar qual é o nome da pasta no filesystem
   - Verificar se o backend está usando o ID correto
   - Verificar se o AIResearch está procurando pelo ID correto

---

## 🔗 Arquivos Relacionados

- `News-main/news-dashboard/src/pages/Logs.tsx` - Frontend do dashboard
- `News-main/news-backend/src/routes/logs.rs` - Endpoints do backend
- `News-main/news-backend/src/utils/article_registry.rs` - RegistryManager
- `News-main/apps/frontend-next/airesearch/app/api/articles/route.ts` - API do AIResearch
- `News-main/articles_registry.json` - Arquivo de registry

---

## 📊 Estatísticas

- **Tentativas de resolução:** 4
- **Arquivos modificados:** 3
- **Linhas de código adicionadas:** ~200
- **Tempo de investigação:** ~2 horas
- **Status atual:** Em investigação

---

## 🎯 Conclusão

O problema parece estar relacionado a:
1. **Registry não está sendo salvo corretamente** (mais provável)
2. **Mismatch de IDs entre backend e frontend** (provável)
3. **Falta de filtro de Hidden no AIResearch** (confirmado)

A próxima ação deve ser verificar os logs do backend e confirmar se o registry está sendo atualizado no arquivo.

