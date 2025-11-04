# 🔍 Análise de Valores Hardcoded no Pipeline de Artigos

## 📋 Resumo
Este documento lista todos os valores hardcoded no pipeline de artigos que **deveriam** ser lidos do `system_config.json` ou variáveis de ambiente.

---

## ❌ Valores Hardcoded Identificados

### 1. **Caminhos de Diretórios** (CRÍTICO)

#### 📁 Caminho do `system_config.json`
- **Localização**: `main.rs:264`, `main.rs:934`, `main.rs:1206`
- **Hardcoded**: `"G:/Hive-Hub/News-main/news-backend/system_config.json"`
- **Deveria**: Usar caminho relativo ou variável de ambiente

#### 📁 Caminho do Registry
- **Localização**: `main.rs:306`, `main.rs:2566`, `main.rs:2703`
- **Hardcoded**: `"G:/Hive-Hub/News-main/articles_registry.json"`
- **Deveria**: Configurável via `system_config.json`

#### 📁 Diretório de Downloads
- **Localização**: `main.rs:322`, `main.rs:806`, `main.rs:869`
- **Hardcoded**: `"G:/Hive-Hub/News-main/downloads"`
- **Deveria**: Configurável via `system_config.json`

#### 📁 Diretório de Output
- **Localização**: `main.rs:2590`, `main.rs:2719`, `main.rs:3122`
- **Hardcoded**: `"G:/Hive-Hub/News-main/output/AIResearch"`
- **Deveria**: Detectar do site configurado em `system_config.json`

---

### 2. **Configurações do Writer** (CRÍTICO)

#### 🔑 API Key do DeepSeek
- **Localização**: `start.rs:322`, `start.rs:335`
- **Hardcoded**: `"sk-3cdb0bc989414f2c8d761ac9ee5c20ce"`
- **Deveria**: Ler de `system_config.json` → `sites.<site_id>.writer.api_key`
- **⚠️ SEGURANÇA**: API key exposta no código!

#### 🎯 Site Padrão
- **Localização**: `start.rs:323`, `start.rs:336`, `main.rs:2584`
- **Hardcoded**: `"AIResearch"` ou `"airesearch"`
- **Deveria**: Detectar primeiro site habilitado de `system_config.json` → `sites.<site_id>.writer.enabled`

---

### 3. **Configurações do Collector** (Parcialmente OK)

#### 📊 Categoria arXiv
- **Localização**: `main.rs:270`
- **Hardcoded**: `"cs.AI"` (apenas como default)
- **Status**: ✅ Já lê do config, mas tem default hardcoded
- **Recomendação**: Remover default ou torná-lo configurável

#### 📈 Max Results
- **Localização**: `main.rs:271`
- **Hardcoded**: `10` (apenas como default)
- **Status**: ✅ Já lê do config, mas tem default hardcoded
- **Recomendação**: Remover default ou torná-lo configurável

---

### 4. **Caminhos no start.rs** (CRÍTICO)

#### 🔧 Caminho do Binário
- **Localização**: `start.rs:512`, `start.rs:315`
- **Hardcoded**: `"G:\\Hive-Hub\\News-main\\news-backend\\target\\debug\\news-backend.exe"`
- **Deveria**: Detectar automaticamente ou usar variável de ambiente

#### 📂 Working Directory
- **Localização**: `start.rs:517`, `start.rs:320`, `start.rs:528`, `start.rs:333`
- **Hardcoded**: `"G:\\Hive-Hub\\News-main\\news-backend"`
- **Deveria**: Detectar automaticamente baseado na localização do `start.rs`

---

## ✅ O que JÁ está Configurável

1. ✅ **Categoria arXiv** - Lê de `system_config.json` → `sites.<site_id>.collectors[arxiv].config.category`
2. ✅ **Max Results** - Lê de `system_config.json` → `sites.<site_id>.collectors[arxiv].config.max_results`
3. ✅ **Site Selection** - Tenta ler de `system_config.json` em `run_writer_pipeline()` (mas tem fallback hardcoded)

---

## 🔧 Correções Necessárias

### Prioridade ALTA 🔴

1. **API Key do DeepSeek**
   - ❌ Atualmente: Hardcoded em `start.rs`
   - ✅ Deveria: Ler de `system_config.json` → `sites.<site_id>.writer.api_key`
   - ⚠️ **Risco de segurança**: API key exposta

2. **Site Padrão**
   - ❌ Atualmente: `"AIResearch"` hardcoded
   - ✅ Deveria: Detectar do config ou variável de ambiente

3. **Caminhos Absolutos**
   - ❌ Atualmente: Todos hardcoded com `G:\Hive-Hub\...`
   - ✅ Deveria: Caminhos relativos ou variáveis de ambiente

### Prioridade MÉDIA 🟡

4. **Diretórios de Output**
   - ❌ Atualmente: `output/AIResearch` hardcoded
   - ✅ Deveria: Baseado no site configurado

5. **Caminho do Registry**
   - ❌ Atualmente: Hardcoded
   - ✅ Deveria: Configurável no `system_config.json`

### Prioridade BAIXA 🟢

6. **Defaults de Categoria e Max Results**
   - ⚠️ Atualmente: Defaults hardcoded (mas já lê do config)
   - ✅ Melhorar: Tornar defaults configuráveis ou remover

---

## 📝 Recomendações

1. **Criar seção `paths` no `system_config.json`**:
```json
{
  "paths": {
    "base_dir": "G:/Hive-Hub/News-main",
    "downloads_dir": "downloads",
    "output_dir": "output",
    "registry_file": "articles_registry.json"
  }
}
```

2. **Criar seção `system` para configurações globais**:
```json
{
  "system": {
    "default_site": "airesearch",
    "default_category": "cs.AI",
    "default_max_results": 10
  }
}
```

3. **Usar caminhos relativos** baseados na localização do executável

4. **Remover API key hardcoded** - sempre ler de config ou variável de ambiente

---

## 🎯 Plano de Ação

1. ✅ **Identificar valores hardcoded** (este documento)
2. ⏳ **Criar estrutura no `system_config.json`** para paths
3. ⏳ **Refatorar `main.rs`** para ler paths do config
4. ⏳ **Refatorar `start.rs`** para ler API keys e sites do config
5. ⏳ **Remover defaults hardcoded** ou torná-los configuráveis
6. ⏳ **Testar pipeline completo** após refatoração

---

**Data de Análise**: 2025-01-03
**Arquivos Analisados**: 
- `start.rs`
- `main.rs` (pipeline de artigos)
- `system_config.json`









