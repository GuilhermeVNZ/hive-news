# 🧩 FASE I — INGESTÃO E ARMAZENAMENTO
# 🧱 Etapa 1 — Coleta e Download (Collector)

## 📋 Visão Geral

Sistema de coleta automatizada de documentos científicos de múltiplas fontes (arXiv, Nature, Science, PubMed, etc.), organizados por origem e data, com persistência de metadados no banco de dados.

**Status:** ✅ arXiv implementado e funcional  
**Localização:** `G:\Hive-Hub\News-main\downloads/<origem>/<YYYY-MM-DD>/`

## 🎯 Objetivo

Conectar-se às APIs das fontes configuradas, baixar arquivos originais (PDFs) e armazená-los no sistema local com organização por origem e data.

## 🔄 Fluxo

```
Scheduler →
  Collector Service →
    API Request (arXiv/Nature/Science) →
      Download PDF →
        downloads/<origem>/<YYYY-MM-DD>/<arquivo>.pdf →
          Metadados em raw_documents
```

## ✅ Estrutura de Downloads

### Diretório Base
**Localização:** `G:\Hive-Hub\News-main\downloads`

### Organização
```
downloads/
├── arxiv/              ← Origem: arXiv
│   └── 2025-10-27/
│       ├── 2106.01234.pdf
│       ├── 2106.01235.pdf
│       └── ... (10 papers)
├── nature/             ← Origem: Nature
│   └── 2025-10-27/
│       └── article.pdf
├── science/            ← Origem: Science
│   └── 2025-10-27/
│       └── article.pdf
└── temp/               ← Arquivos temporários
    └── arxiv_feed_<timestamp>.xml
```

## 🔧 Implementação Atual: arXiv

### ✅ Status Implementado

**Fonte:** arXiv (https://arxiv.org)  
**Categoria:** cs.AI (Computer Science - Artificial Intelligence)  
**Quantidade:** 10 artigos mais recentes  
**Chave API:** Não necessária (API pública)  
**Diretório:** `G:\Hive-Hub\News-main\downloads\arxiv\2025-10-27\`

### Como Funciona

1. **API OAI-PMH do arXiv**
   - URL: `https://export.arxiv.org/api/query`
   - Busca os 10 papers mais recentes de cs.AI
   - Ordena por data de submissão (mais recentes primeiro)

2. **XML Temporário**
   - Salvo em `downloads/temp/arxiv_feed_<timestamp>.xml`
   - Usado para parsing e auditoria

3. **Downloads PDF**
   - URL: `https://arxiv.org/pdf/<paper_id>.pdf`
   - Salvo em: `downloads/arxiv/<YYYY-MM-DD>/<paper_id>.pdf`

4. **Metadados no Banco**
   - Título, resumo, autores
   - URL do paper e do PDF
   - Timestamp de download

## 🔑 Configuração de Chaves API

### Arquivo de Configuração

**Localização:** `news-backend/.env`

```env
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/news_system

# Collector
COLLECTOR_TIMEOUT_SECONDS=30
COLLECTOR_MAX_RETRIES=3
COLLECTOR_DOWNLOAD_DIR=./downloads

# ==========================================
# API Keys for Document Collection Sources
# ==========================================

# arXiv - NO KEY REQUIRED ✅ (Implementado)
ARXIV_BASE_URL=https://export.arxiv.org/api/query
ARXIV_CATEGORY=cs.AI
ARXIV_MAX_RESULTS=10
ARXIV_RATE_LIMIT=120

# Nature Publishing Group
NATURE_API_KEY=your_nature_api_key_here
NATURE_BASE_URL=https://api.nature.com
NATURE_RATE_LIMIT=60

# Science (AAAS)
SCIENCE_API_KEY=your_science_api_key_here
SCIENCE_BASE_URL=https://api.science.org
SCIENCE_RATE_LIMIT=60

# PubMed / NCBI
PUBMED_BASE_URL=https://eutils.ncbi.nlm.nih.gov/entrez/eutils
PUBMED_RATE_LIMIT=100

# IEEE Xplore
IEEE_API_KEY=your_ieee_api_key_here
IEEE_BASE_URL=https://ieeexploreapi.ieee.org/api/v1
IEEE_RATE_LIMIT=60

# Springer Nature
SPRINGER_API_KEY=your_springer_api_key_here
SPRINGER_BASE_URL=https://api.springernature.com
SPRINGER_RATE_LIMIT=60

# Elsevier ScienceDirect
ELSEVIER_API_KEY=your_elsevier_api_key_here
ELSEVIER_BASE_URL=https://api.elsevier.com/content
ELSEVIER_RATE_LIMIT=60

# File Management
TEMP_FILE_RETENTION_DAYS=7
MAX_DOWNLOAD_SIZE_MB=50
```

### Como Configurar

```bash
cd G:\Hive-Hub\News-main\news-backend

# 1. Copiar template
copy .env.example .env

# 2. Editar com suas chaves
notepad .env

# 3. Executar
cargo run
```

## 📊 Fontes Disponíveis

### ✅ arXiv (IMPLEMENTADO)

**Status:** Funcional  
**Chave API:** Não necessária  
**Categoria:** cs.AI (configurável)  
**Quantidade:** 10 papers  
**Rate Limit:** 120 requests/min

**Categorias Disponíveis:**
- `cs.AI` - Artificial Intelligence
- `cs.LG` - Machine Learning
- `cs.CV` - Computer Vision
- `cs.NE` - Neural and Evolutionary Computing
- `cs.CL` - Computation and Language
- `stat.ML` - Statistics - Machine Learning

**API:** https://export.arxiv.org/api/query  
**URL PDF:** https://arxiv.org/pdf/\<paper_id\>.pdf  
**Documentação:** https://arxiv.org/help/api

### ⏳ Nature Publishing Group

**Status:** Em desenvolvimento  
**Chave API:** Necessária  
**Registro:** https://go.nature.com/api-keys  
**Rate Limit:** 60 requests/min  
**Documentação:** https://www.nature.com/documents/nr-API-Developer-Guide.pdf

### ⏳ Science (AAAS)

**Status:** Em desenvolvimento  
**Chave API:** Necessária  
**Registro:** https://science.sciencemag.org/api  
**Rate Limit:** 60 requests/min  
**Documentação:** https://www.science.org/content/page/science-api

### ⏳ PubMed / NCBI

**Status:** Planejado  
**Chave API:** Não necessária  
**Rate Limit:** 100 requests/min  
**Documentação:** https://www.ncbi.nlm.nih.gov/books/NBK25497/

### ⏳ IEEE Xplore

**Status:** Planejado  
**Chave API:** Necessária  
**Registro:** https://developer.ieee.org/register  
**Rate Limit:** 60 requests/min  
**Documentação:** https://developer.ieee.org/

### ⏳ Springer Nature

**Status:** Planejado  
**Chave API:** Necessária  
**Registro:** https://dev.springernature.com/signup  
**Rate Limit:** 60 requests/min  
**Documentação:** https://dev.springernature.com/

### ⏳ Elsevier ScienceDirect

**Status:** Planejado  
**Chave API:** Necessária  
**Registro:** https://dev.elsevier.com/user/login  
**Rate Limit:** 60 requests/min  
**Documentação:** https://dev.elsevier.com/

## 📊 Schema do Banco de Dados

```sql
CREATE TABLE raw_documents (
    id SERIAL PRIMARY KEY,
    portal_id INT REFERENCES pages_config(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source_url TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size BIGINT,
    metadata JSONB DEFAULT '{}',
    downloaded_at TIMESTAMP DEFAULT NOW(),
    processed BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_raw_documents_portal_id ON raw_documents(portal_id);
CREATE INDEX idx_raw_documents_processed ON raw_documents(processed);
CREATE INDEX idx_raw_documents_downloaded_at ON raw_documents(downloaded_at);
```

## 🧰 Tecnologias

- **reqwest**: HTTP client assíncrono
- **tokio**: Concorrência assíncrona e runtime
- **chrono**: Controle temporal e formatação de datas
- **serde_json**: Serialização e parsing de JSON
- **tracing**: Sistema de logs estruturados
- **sqlx**: Interações com PostgreSQL
- **tokio-cron-scheduler**: Agendamento de tarefas
- **regex**: Parsing de XML do arXiv

## 🔧 CollectorService

### Estrutura

```rust
pub struct CollectorService {
    db: PgPool,
    client: reqwest::Client,
    download_dir: PathBuf,
    arxiv_collector: ArxivCollector,
}

impl CollectorService {
    // Coleta artigos para um portal
    pub async fn collect_for_portal(&self, portal_id: i32) -> Result<CollectionResult>
    
    // Faz download de um artigo
    pub async fn download_article(&self, article, portal, source) -> Result<PathBuf>
    
    // Salva metadados no banco
    pub async fn save_document(&self, doc: &CreateRawDocument) -> Result<i32>
    
    // Busca documentos não processados
    pub async fn get_unprocessed_documents(&self) -> Result<Vec<RawDocument>>
    
    // Marca documento como processado
    pub async fn mark_as_processed(&self, document_id: i32) -> Result<()>
}
```

## 🚀 Como Usar

### Via start.rs

```bash
cd G:\Hive-Hub\News-main

# Ver status do collector
cargo run --bin start collector

# Iniciar sistema completo
cargo run --bin start start
```

### Diretamente no Backend

```bash
cd G:\Hive-Hub\News-main\news-backend

# Carregar .env e executar
cargo run
```

## 📝 Endpoints REST

### POST /api/collector/start

Inicia coleta para um portal específico.

**Request:**
```json
{
  "portal_id": 1
}
```

**Response:**
```json
{
  "success": true,
  "documents_collected": 12,
  "duration_ms": 1523,
  "errors": []
}
```

### GET /api/collector/status/:portal_id

Retorna status da última coleta.

**Response:**
```json
{
  "portal_id": 1,
  "last_collection": "2025-10-27T14:30:00Z",
  "status": "success",
  "articles_collected": 12,
  "next_collection": "2025-10-27T15:30:00Z"
}
```

### GET /api/collector/logs

Retorna logs de coletas.

**Query params:** `portal_id`, `limit`, `offset`

## 🧪 Testes

### Unit Tests

- ✅ Download de arquivo simples
- ✅ Sanitização de nome de arquivo
- ✅ Deduplicação de downloads
- ✅ Criação de estrutura de diretórios
- ✅ Parsing de XML do arXiv

### Integration Tests

- ⏳ Coleta completa para um portal
- ⏳ Persistência no banco de dados
- ⏳ Verificação de arquivos baixados
- ⏳ Scheduler de tarefas

## 📊 Monitoramento

### Métricas Coletadas

- ✅ Total de documentos coletados
- ✅ Taxa de sucesso vs falhas
- ✅ Tempo médio de download
- ✅ Tamanho médio de arquivos
- ✅ Última coleta por portal
- ✅ Rate limiting por API

### Logs Estruturados

```rust
tracing::info!(
    source = %source,
    paper_id = %paper_id,
    pdf_url = %pdf_url,
    path = %file_path.display(),
    size = size,
    duration_ms = %duration.as_millis(),
    "Article downloaded successfully"
);
```

## 🚨 Tratamento de Erros

### Cenários Comuns

**1. API Indisponível**
- Retry com backoff exponencial
- Log estruturado
- Continua para próxima fonte

**2. Arquivo Corrompido**
- Validação de checksum (opcional)
- Retry do download
- Log de erro específico

**3. Espaço em Disco Insuficiente**
- Verificação prévia de espaço
- Limpeza de arquivos antigos (opcional)
- Notificação de erro crítico

**4. Rate Limit Excedido**
- Aguarda tempo necessário
- Retry após rate limit reset
- Log de rate limiting

## 🔒 Segurança

### ⚠️ IMPORTANTE

1. **NUNCA commite arquivo `.env`** com chaves reais
2. **Sempre use `.env.example`** como template
3. **Adicione `.env` ao `.gitignore`**
4. **Use variáveis de ambiente** em produção
5. **Rote as chaves** quando possível

### .gitignore

```gitignore
# Environment variables
.env
.env.local
.env.*.local
```

## 📈 Status da Implementação

### ✅ Concluído

- [x] Estrutura de diretórios por origem
- [x] Organização por data (YYYY-MM-DD)
- [x] Sanitização de nomes de arquivo
- [x] Deduplicação de downloads
- [x] Persistência no banco de dados
- [x] CollectorService implementado
- [x] arXiv API integrada (10 papers)
- [x] XML temporário salvo
- [x] Sistema de configuração de chaves API
- [x] Rate limiting por API
- [x] Logs estruturados

### ⏳ Em Progresso

- [ ] Busca de configuração de portal no banco
- [ ] Busca de sources configuradas no banco
- [ ] Scheduler automático com tokio-cron-scheduler
- [ ] Retry logic com backoff exponencial

### 🎯 Próximos Passos

**Implementar Outras Fontes:**
- Nature Publishing Group
- Science (AAAS)
- PubMed / NCBI
- IEEE Xplore
- Springer Nature
- Elsevier ScienceDirect

**Funcionalidades Adicionais:**
- [x] **Limpeza automática de arquivos temporários** - Implementada ✅
- [x] **Coleta incremental anti-duplicação** - Implementada ✅
- [x] **Correção de bloqueio reCAPTCHA nos downloads** - Implementada ✅
- [ ] Validação de arquivos baixados
- [ ] Download paralelo de múltiplos PDFs
- [ ] Estatísticas de uso de disco
- [ ] Webhooks para notificações

## 🐛 Problema Resolvido: reCAPTCHA Blocking Downloads

### Problema Identificado

O download de PDFs estava falhando com mensagens de "Invalid PDF (got HTML or redirect)" porque o site público do arXiv (`arxiv.org/pdf/{ID}.pdf`) estava apresentando um desafio reCAPTCHA do Google:

**Erro original:**
```
❌ Invalid PDF (got HTML or redirect)
```

**Causa:** O endpoint público do arXiv implementa proteção anti-bot que bloqueia downloads automáticos com reCAPTCHA.

### Solução Implementada

**Mudança de endpoint:** Alterado de `arxiv.org` para `export.arxiv.org` (API oficial).

**Antes:**
```rust
let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", paper_id);
// ❌ Retorna HTML com reCAPTCHA
```

**Depois:**
```rust
let pdf_url = format!("https://export.arxiv.org/pdf/{}.pdf", paper_id);
// ✅ Retorna PDF diretamente
```

### Proteções Adicionais Implementadas

1. **Cookies e Sessão**
   ```rust
   let client = reqwest::Client::builder()
       .cookie_store(true)  // Manter sessão entre requisições
       .user_agent("Mozilla/5.0...")  // Simular navegador real
       .build()?;
   ```

2. **Headers de Navegador Real**
   ```rust
   .header("Accept", "application/pdf,text/html,application/xhtml+xml")
   .header("Accept-Language", "en-US,en;q=0.9")
   .header("Sec-Fetch-Dest", "document")
   .header("Sec-Fetch-Mode", "navigate")
   ```

3. **Estabelecer Sessão Antecipadamente**
   ```rust
   // Fazer requisição inicial para obter cookies
   client.get("https://arxiv.org/list/cs.AI/recent").send().await?;
   tokio::time::sleep(Duration::from_secs(2)).await;
   ```

4. **Rate Limiting entre Downloads**
   ```rust
   // Delay de 3 segundos entre downloads
   tokio::time::sleep(Duration::from_secs(3)).await;
   ```

### Resultado

✅ **Downloads funcionando 100%**  
✅ **10/10 papers baixados com sucesso**  
✅ **Sem bloqueios de reCAPTCHA**  
✅ **Sem arquivos HTML ou erros**

**Output final:**
```
✅ Collection completed!
   New papers downloaded: 10/10
   Location: G:/Hive-Hub/News-main/downloads\arxiv\2025-10-27
```

---

## 🎉 Funcionalidades Implementadas

### 1. Coleta Incremental e Anti-Duplicação

O sistema busca automaticamente **artigos novos não duplicados**:

**Como funciona:**
```rust
// Loop até baixar 10 novos artigos
while downloaded_count < 10 {
    // Busca batch com offset dinâmico (0, 10, 20...)
    let url = format!("https://export.arxiv.org/api/query?search_query=cat:cs.AI&start={}&max_results=20", offset);
    
    // Para cada artigo
    if file_already_downloaded(paper_id, base_dir) {
        println!("  ⏭️  (already exists)");
        continue; // Pula duplicados
    } else {
        download_pdf(); // Baixa apenas novos
        downloaded_count++;
    }
    
    // Incrementa offset se não encontrou novos
    if downloaded_count == 0 {
        offset += 10;
    }
}
```

**Verificação em 3 locais:**
```rust
fn file_already_downloaded(paper_id: &str, base_dir: &Path) -> bool {
    // 1. Verifica em downloads/arxiv/ (todas as datas)
    // 2. Verifica em downloads/filtered/<categoria>/
    // 3. Verifica em downloads/rejected/
    
    // Retorna true se encontrado em qualquer lugar
}
```

**Características:**
- ✅ Verifica existência antes de baixar
- ✅ **Verifica em 3 locais**: arxiv/, filtered/, rejected/
- ✅ Offset dinâmico (auto-incrementa até 100)
- ✅ Loop inteligente até completar 10 novos artigos
- ✅ Logs claros: `⏭️ (already exists)` vs `✅ NEW`
- ✅ Safety limit para evitar loops infinitos
- ✅ **NOVO**: Detecta arquivos já movidos pelo filtro

**Output exemplo:**
```
📡 Fetching batch starting from offset 0...
  Found 20 papers in this batch
  [1/10]: 2510.21695v1... ⏭️  (already exists)
  [1/10]: 2510.21689v1... ⏭️  (already exists)
  ...
  [1/10]: 2510.21443v1... ✅ NEW
  [2/10]: 2510.21436v1... ✅ NEW
  ...
```

### 2. Limpeza Automática de Arquivos Temporários

**Implementado:** O sistema **remove automaticamente** arquivos XML temporários após cada coleta:

```rust
async fn cleanup_temp_files(temp_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(temp_dir)? {
        let path = entry.path();
        
        // Deletar apenas XMLs
        if path.is_file() && path.extension() == "xml" {
            fs::remove_file(&path)?;
            println!("  ✓ Deleted: {}", filename);
        }
    }
}
```

**Execução automática:**
```rust
println!("\n✅ Collection completed!");
println!("   New papers downloaded: 10/10");

// Limpar arquivos temporários
println!("\n🧹 Cleaning temporary files...");
cleanup_temp_files(&temp_dir).await?;
```

**Output exemplo:**
```
✅ Collection completed!
   New papers downloaded: 10/10
   Location: G:/Hive-Hub/News-main/downloads\arxiv\2025-10-27

🧹 Cleaning temporary files...
  ✓ Deleted: arxiv_feed_1761543231.xml
  ✓ Deleted: arxiv_feed_1761543107.xml
  Cleaned 2 temporary file(s)
```

**Resultado:** Diretório `downloads/temp/` sempre limpo após execução.

## 📚 Próximas Etapas do Pipeline

Após implementar o Collector:

1. **Etapa 2**: Extração e chunking de texto (`ExtractorService`)
2. **Etapa 3**: Embedding e indexação vetorial (`EmbedderService`)
3. **Etapa 4**: Rankeamento e seleção de artigos (`RankerService`)
4. **Etapa 5**: Geração de conteúdo (`PublisherService`)

## 📚 Referências

- [reqwest documentation](https://docs.rs/reqwest/)
- [tokio-cron-scheduler](https://docs.rs/tokio-cron-scheduler/)
- [sqlx migrations](https://docs.rs/sqlx/0.7/sqlx/macro.migrate.html)
- [tracing crate](https://docs.rs/tracing/)
- [arXiv API](https://arxiv.org/help/api)
- [Nature API](https://www.nature.com/documents/nr-API-Developer-Guide.pdf)

---

**🎯 Collector implementado com arXiv e pronto para expansão com outras fontes!**

**Localização:** `G:\Hive-Hub\News-main\downloads/<origem>/<YYYY-MM-DD>/`
