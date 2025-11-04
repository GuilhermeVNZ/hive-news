# Verificação do Fluxo Featured/Hidden

## Fluxo Completo

### 1. Dashboard (Frontend)
**Arquivo**: `news-dashboard/src/pages/Logs.tsx`

**Featured**:
```typescript
// Linha 218
await axios.put(`/api/logs/articles/${item.id}/featured`, 
  { featured: newValue }
);
```

**Hidden**:
```typescript
// Linha 250
await axios.put(`/api/logs/articles/${item.id}/hidden`, 
  { hidden: newValue }
);
```

### 2. Backend API (Rust)
**Arquivo**: `news-backend/src/routes/logs.rs`

**Featured**:
```rust
// Linha 593
pub async fn set_featured(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
    Extension(manager): Extension<Arc<RegistryManager>>,
) -> Json<Value> {
    // ... encontra artigo ...
    manager.set_featured(&article_id_to_update, body.featured)?;
    // ...
}
```

**Hidden**:
```rust
// Linha 386
pub async fn set_hidden(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
    Extension(manager): Extension<Arc<RegistryManager>>,
) -> Json<Value> {
    // ... encontra artigo ...
    manager.set_hidden(&article_id_to_update, body.hidden)?;
    // ...
}
```

### 3. RegistryManager (Rust)
**Arquivo**: `news-backend/src/utils/article_registry.rs`

**Featured**:
```rust
// Linha 562
pub fn set_featured(&self, article_id: &str, featured: bool) -> Result<()> {
    let mut registry = self.registry.lock().unwrap();
    if let Some(meta) = registry.articles.get_mut(article_id) {
        meta.featured = Some(featured);  // ✅ Atualiza o campo
        drop(registry);
        self.save()?;  // ✅ Salva no arquivo JSON
        Ok(())
    }
}
```

**Hidden**:
```rust
// Linha 589
pub fn set_hidden(&self, article_id: &str, hidden: bool) -> Result<()> {
    let mut registry = self.registry.lock().unwrap();
    if let Some(meta) = registry.articles.get_mut(article_id) {
        meta.hidden = Some(hidden);  // ✅ Atualiza o campo
        drop(registry);
        self.save()?;  // ✅ Salva no arquivo JSON
        Ok(())
    }
}
```

### 4. Save (Escrita Atômica)
**Arquivo**: `news-backend/src/utils/article_registry.rs`

```rust
// Linha 417
pub fn save(&self) -> Result<()> {
    let registry = self.registry.lock().unwrap();
    registry.save(&self.registry_path)  // ✅ Salva atomicamente usando tempfile
}
```

```rust
// Linha 230
pub fn save(&self, registry_path: &Path) -> Result<()> {
    // 1. Cria arquivo temporário
    // 2. Escreve conteúdo JSON
    // 3. sync_all() - força sincronização física
    // 4. persist() - rename atômico (temp -> final)
    // ✅ Escrita atômica garante que não há corrupção
}
```

## Verificação

### ✅ Código está correto:
1. Dashboard faz PUT para `/api/logs/articles/{id}/featured` e `/api/logs/articles/{id}/hidden`
2. Backend processa as chamadas e chama `manager.set_featured()` e `manager.set_hidden()`
3. RegistryManager atualiza `meta.featured` e `meta.hidden` e chama `save()`
4. `save()` salva atomicamente no arquivo JSON usando `tempfile`

### ⚠️ Possíveis problemas:
1. **Backend não está rodando** - As chamadas PUT falham
2. **Permissões de arquivo** - Backend não consegue escrever no registry
3. **Artigo não encontrado** - ID do artigo não corresponde ao registry
4. **Erro na serialização JSON** - Falha ao salvar o registry

## Como Testar

1. Execute o script de teste:
   ```powershell
   cd G:\Hive-Hub\News-main
   powershell -ExecutionPolicy Bypass -File scripts\test_featured_hidden_update.ps1
   ```

2. O script mostrará:
   - Estado atual do artigo (featured/hidden)
   - Instruções para testar no dashboard
   - Estado após as mudanças
   - Resultado do teste

3. Se as mudanças NÃO forem refletidas:
   - Verifique se o backend está rodando
   - Verifique os logs do backend para erros
   - Verifique permissões do arquivo `articles_registry.json`
   - Verifique o Network tab no navegador para ver se a chamada API foi bem-sucedida

