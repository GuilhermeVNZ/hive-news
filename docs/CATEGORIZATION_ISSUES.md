# Problemas de Categorização - Relatório e Correções

## Data: 2025-10-26

## Problemas Identificados

### 1. Claude sendo categorizado como OpenAI ❌

**Problema:**
- Artigos sobre Claude (modelo da Anthropic) estavam sendo categorizados como OpenAI
- Exemplo: `www_claude-sonnet-4-5_66075d6f237d20a0` estava como "unknown"

**Causa:**
- A lógica de detecção de categoria não verificava "claude" explicitamente
- Artigos com "claude" no nome ou source.txt não eram mapeados para "anthropic"

**Onde ocorreu:**
1. ✅ **Backend (news_writer.rs)**: Função `detect_source_category()` não verificava "claude" ou "anthropic.com"
2. ✅ **Frontend (vite-plugin-articles-api.ts)**: Lógica de categorização não verificava "claude" no source.txt ou articleId

**Correções aplicadas:**
- ✅ Adicionada verificação `claude` → `anthropic` no backend
- ✅ Adicionada verificação `anthropic.com` no backend  
- ✅ Adicionada verificação `claude` no frontend (source.txt e articleId)
- ✅ Priorizada verificação de Anthropic ANTES de OpenAI na lógica

### 2. DeepSeek escrito de forma inconsistente ❌

**Problema:**
- Nome aparecia como "Deep Seek", "Deepseek", "deepseek" em diferentes lugares
- Nome oficial correto: **"DeepSeek"** (S maiúsculo)

**Onde ocorreu:**
- Frontend: Labels e categorias já estavam corretos ("DeepSeek")
- Backend: Configurações e comentários usavam "DeepSeek" corretamente
- ✅ **Status**: Já padronizado como "DeepSeek" em todos os lugares

## Fluxo de Categorização

### 1. Coleta (Backend - `news_writer.rs`)

**Função**: `detect_source_category(url, title)`

**Prioridade:**
1. Verifica URL primeiro (mais confiável)
2. Verifica title como fallback
3. Retorna "unknown" se não encontrar

**Mapeamentos:**
- `anthropic.com`, `anthropic`, `claude` → `anthropic` ✅ CORRIGIDO
- `openai.com`, `openai` → `openai`
- `nvidia.com`, `nvidia` → `nvidia`
- `google.com`, `blog.research.google` → `google`
- `meta.com`, `facebook.com`, `about.fb.com` → `meta`
- `deepseek.ai`, `deepseek` → `deepseek`

**Salvo em**: `source.txt` no diretório do artigo

### 2. Leitura no Frontend (vite-plugin-articles-api.ts)

**Prioridade:**
1. Lê `source.txt` (prioritário)
2. Verifica `image_categories.txt` (fallback)
3. Verifica `articleId` (fallback final)

**Mapeamentos:**
- `claude` ou `anthropic` em source.txt → `anthropic` ✅ CORRIGIDO
- `claude` no articleId → `anthropic` ✅ CORRIGIDO
- `deepseek` → `deepseek` ✅ CORRETO

## Como Prevenir Erros Futuros

### Checklist para Nova Categoria:

1. **Backend (`news_writer.rs`)**:
   - ✅ Adicionar URL da empresa em `detect_source_category()`
   - ✅ Adicionar nome da empresa no title check
   - ✅ Adicionar variantes comuns (ex: "claude" para Anthropic)

2. **Frontend (`vite-plugin-articles-api.ts`)**:
   - ✅ Adicionar mapeamento no processamento de `source.txt`
   - ✅ Adicionar no fallback de `image_categories.txt`
   - ✅ Adicionar verificação no `articleId`

3. **Configurações**:
   - ✅ Adicionar em `mockData.ts` (categorias do frontend)
   - ✅ Adicionar em todos os componentes que usam categoryLabels

### Padrão de Nomenclatura:

- **Categoria slug**: sempre lowercase (`anthropic`, `openai`, `deepseek`)
- **Display name**: sempre capitalizado corretamente (`Anthropic`, `OpenAI`, `DeepSeek`)
- **Source.txt**: usar slug lowercase

## Testes Recomendados

1. Verificar artigos com "claude" no nome são categorizados como "anthropic"
2. Verificar artigos com URL "anthropic.com" são categorizados como "anthropic"
3. Verificar nome "DeepSeek" está padronizado em todos os lugares
4. Testar fallback quando source.txt está vazio





















