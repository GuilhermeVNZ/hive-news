# 🎯 Orquestrador start.rs - Documentação Completa

> **Documento Consolidado** - Contém toda a informação sobre o orquestrador em um único lugar.

## 📋 Visão Geral

O arquivo `start.rs` é o **ORQUESTRADOR CENTRAL** do News System. Ele é responsável por:

1. ✅ **Iniciar todos os serviços necessários** em ordem
2. ✅ **Coletar configurações do dashboard** via API
3. ✅ **Orquestrar cada módulo** até conclusão das tarefas
4. ✅ **Monitorar saúde do sistema** continuamente
5. ✅ **Gerenciar ciclo de vida** de todo o sistema
6. ✅ **Executar Collector Service** para downloads automáticos

## 🎯 Função Principal

O orquestrador atua como o **cérebro central** do sistema, garantindo que todos os componentes trabalhem juntos de forma coordenada e eficiente.

```
start.rs (Orquestrador)
    │
    ├─→ Gerencia ciclo de vida
    ├─→ Coordena módulos
    ├─→ Monitora saúde
    ├─→ Executa tarefas agendadas
    └─→ Controla Collector Service
```

## 📚 Índice

1. [Resumo Executivo](#resumo-executivo)
2. [Arquitetura](#arquitetura)
3. [Fluxo de Inicialização](#fluxo-de-inicialização)
4. [Interação com Collector](#interação-com-collector)
5. [Orquestração de Módulos](#orquestração-de-módulos)
6. [Comandos Disponíveis](#comandos-disponíveis)
7. [Testes e Debugging](#testes-e-debugging)

---

## 📖 Resumo Executivo

### O Que É o `start.rs`?

O `start.rs` é o **ORQUESTRADOR CENTRAL** do News System - o cérebro que coordena todos os módulos do sistema, incluindo o **Collector Service**.

### Função Principal

```rust
start.rs = Gerente + Coordenador + Monitor + Collector Orchestrator
```

- **Gerencia**: Inicia e para serviços
- **Coordena**: Fluxo de dados entre módulos
- **Monitora**: Saúde e performance do sistema
- **Controla Collector**: Executa downloads automáticos de documentos

### O Que Ele Faz?

#### 1. Inicia o Sistema (start start)

```
🔧 Backend (3001)      ← Primeiro  
🎨 Dashboard (1420)    ← Segundo
🌐 Portais (3003/8080) ← Terceiro
```

#### 2. Coleta Informações do Dashboard

```rust
// Lê configurações ativas
GET /api/pages    → Quais portais estão ativos
GET /api/sources  → Quais fontes configuradas
```

#### 3. Orquestra Módulos (incluindo Collector)

```
Dashboard Config
      ↓
Scheduler detecta tempo
      ↓
Collector busca artigos  ← NOVO! Downloads automáticos
      ↓
Downloads salvos em downloads/<origem>/<YYYY-MM-DD>/
      ↓
Metadados em raw_documents
      ↓
Extractor processa texto
      ↓
Embedder cria vetores
      ↓
Ranker seleciona top-K
```

#### 4. Monitora Continuamente

```rust
Loop infinito (30s):
  ├─ Health check de todos serviços
  ├─ Verifica mudanças no dashboard
  ├─ Executa tarefas agendadas (incluindo Collector)
  └─ Coleta métricas e logs
```

### Resumo em 1 Frase

**O `start.rs` é o maestro que coordena todos os músicos (serviços), incluindo o Collector para downloads automáticos de artigos!** 🎼

---

## 🏗️ Arquitetura

### Componentes Gerenciados

1. **Backend API** (`http://localhost:3001`)
   - API RESTful
   - Lógica de negócios
   - Banco de dados PostgreSQL
   - **Collector Service** ← NOVO!

2. **Dashboard Frontend** (`http://localhost:1420`)
   - Interface administrativa
   - Configuração de portais
   - Monitoramento em tempo real

3. **Scheduler Service**
   - Agendamento de coletas
   - Execução de tarefas periódicas
   - Trigger de processos
   - **Controla Collector Service** ← NOVO!

4. **Collector Service** ← NOVO!
   - Downloads organizados por origem
   - Estrutura: `downloads/<origem>/<YYYY-MM-DD>/`
   - Persistência em `raw_documents`

### Diagrama de Arquitetura com Collector

```
┌─────────────────────────────────────┐
│      ORQUESTRADOR (start.rs)       │
│                                     │
│  1. Inicia serviços em ordem       │
│  2. Aguarda readiness de cada um   │
│  3. Coleta config do dashboard      │
│  4. Configura scheduler             │
│  5. Monitora continuamente          │
│  6. Executa tarefas agendadas      │
│  7. Controla Collector Service     │ ← NOVO!
└─────────────────────────────────────┘
         │           │           │
         ▼           ▼           ▼
    ┌────────┐  ┌────────────┐  ┌──────────────┐
    │ Backend │  │ Frontends   │  │ Collector     │
    │  :3001  │  │ :1420/8080 │  │ Service       │
    └────────┘  └────────────┘  └──────┬───────┘
         │             │               │
         └─────────────┴───────────────┘
                       │
                       ▼
                ┌─────────────┐
                │   Database  │
                │  PostgreSQL │
                └─────────────┘
                               │
                               ▼
                       ┌──────────────────┐
                       │   downloads/     │
                       │   └──<origem>/   │
                       │      └──<data>/  │
                       └──────────────────┘
```

---

## 🔄 Interação com Collector

### Como o start.rs Controla o Collector

#### 1. Inicialização

```rust
// No start_full_system()
fn start_full_system() {
    // ... outros serviços ...
    
    // Backend inicia com Collector Service
    start_backend_background();  // ← Inclui Collector
}
```

#### 2. Comando Dedicated

```rust
fn test_collector() {
    println!("🔍 Testing Collector Service...");
    println!("\n✅ Collector Implementation Status:");
    println!("\n📂 Downloads Location: G:\\Hive-Hub\\News-main\\downloads");
    
    // Mostra estrutura de downloads
}
```

#### 3. Scheduler Integration

```rust
fn run_scheduler() {
    // Configura collector baseado no dashboard
    configure_scheduler_from_dashboard();
    
    // Executa collector para cada portal
    // Downloads salvos em downloads/<origem>/<data>/
}
```

### Estrutura de Downloads do Collector

```
downloads/
├── arxiv/              ← Origem: arXiv
│   └── 2025-10-27/     ← Data: YYYY-MM-DD
│       └── article.pdf ← Arquivo baixado
├── nature/             ← Origem: Nature
│   └── 2025-10-27/
│       └── article.pdf
└── science/            ← Origem: Science
    └── 2025-10-27/
        └── article.pdf
```

### Fluxo Completo com Collector

```
start.rs
    ↓
Start Backend (inclui Collector Service)
    ↓
Dashboard Configura fontes
    ↓
Scheduler detecta tempo configurado
    ↓
Collector Service executa
    ├─ Busca novos artigos das APIs
    ├─ Download de arquivos PDF
    ├─ Salva em downloads/<origem>/<data>/
    └─ Registra metadados em raw_documents
    ↓
Arquivos prontos para processamento
```

---

## 🔄 Fluxo de Inicialização com Collector

### Etapa 1: Verificação de Dependências

```rust
check_dependencies()
```

Verifica se todas as dependências estão instaladas:

- ✅ PostgreSQL (banco de dados)
- ✅ Rust (compilador)
- ✅ Node.js (runtime frontend)
- ✅ npm (gerenciador de pacotes)
- ✅ Diretório `downloads/` ← NOVO!

### Etapa 2: Iniciar Backend (COM COLLECTOR)

```rust
start_backend_background()
```

- Compila e inicia backend Rust
- API disponível em `http://localhost:3001`
- **Collector Service** iniciado
- Diretório `downloads/` configurado
- Aguarda 3 segundos para estabilizar

**Serviços iniciados:**
- REST API endpoints
- Database connection pool
- Auth middleware
- **Collector Service** ← NOVO!
- Scheduler service

### Etapa 3: Coletar Configurações

```rust
collect_dashboard_config()
```

Coleta configurações ativas do backend via API:

```json
GET /api/pages
GET /api/sources
```

**Informações coletadas:**
- Portais ativos (AIResearch, ScienceAI)
- Fontes configuradas (Nature, Science, arXiv) ← Para Collector!
- Frequências de coleta (60min, 120min)
- Estilos de escrita (scientific, technical)

### Etapa 4: Configurar Scheduler (COM COLLECTOR)

```rust
configure_scheduler_from_dashboard()
```

Configura tarefas agendadas incluindo executar Collector:

```rust
{
    "AIResearch": {
        "frequency": "60 minutes",
        "sources": ["Nature", "Science"],
        "collector": {
            "download_dir": "G:/Hive-Hub/News-main/downloads",
            "organize_by": "source"
        }
    }
}
```

### Etapa 5: Iniciar Dashboard

```rust
start_dashboard_background()
```

- Inicia servidor Vite (dev mode)
- Interface React disponível
- WebSocket para updates em tempo real
- **Pode configurar colectas via UI** ← NOVO!

### Etapa 6: Loop de Orquestração (COM COLLECTOR)

```rust
run_orchestration_loop()
```

Loop contínuo que executa a cada 30 segundos:

1. **Health Check** - Verifica status de todos componentes, **incluindo Collector**
2. **Coletar Updates** - Busca mudanças no dashboard
3. **Executar Collector** - Executa downloads agendados ← NOVO!
4. **Monitorar** - Coleta métricas e logs do Collector ← NOVO!

---

## 🔄 Orquestração de Módulos com Collector

### Fluxo Completo de Dados

```
Dashboard (Usuário)
    ↓ define configurações
Orquestrador
    ↓ lê configurações
Scheduler
    ↓ agenda tarefas
Collector ← INCLUÍDO!
    ↓ baixa documentos para downloads/<origem>/<data>/
    ↓ salva metadados em raw_documents
Extractor
    ↓ extrai texto
Embedder
    ↓ gera embeddings
Ranker
    ↓ seleciona top-K
Publisher
    ↓ publica conteúdo
Portals
    ↓ mostra ao usuário
```

### Orquestração Detalhada do Collector

#### A. Collector → Download (IMPLEMENTADO!)

```
Dashboard Config
    ↓
Scheduler detecta tempo
    ↓
Collector Service (news-backend/src/services/collector_service.rs)
    ├─ Busca novos artigos das APIs configuradas
    ├─ Download de PDFs usando reqwest
    ├─ Organiza em downloads/<origem>/<YYYY-MM-DD>/
    ├─ Sanitiza nomes de arquivos
    ├─ Deduplica downloads
    └─ Salva metadados em raw_documents
```

### Como o start.rs Interage com Collector

#### 1. Comando `collector`

```bash
cargo run --bin start collector
```

**O que faz:**
- Mostra status da implementação do Collector
- Exibe localização dos downloads: `G:\Hive-Hub\News-main\downloads`
- Mostra features implementadas
- Exibe estrutura de organização dos downloads

**Output:**
```
🔍 Testing Collector Service...

✅ Collector Implementation Status:

📂 Downloads Location: G:\Hive-Hub\News-main\downloads

📋 Features Implemented:
   ✅ Download directory structure (per source)
   ✅ Organization by date (YYYY-MM-DD)
   ✅ Filename sanitization
   ✅ Deduplication check
   ✅ Database persistence
   ✅ Metadata storage

📊 Example Structure:
   downloads/
   ├── arxiv/
   │   └── 2025-10-27/
   │       └── article.pdf
   ├── nature/
   │   └── 2025-10-27/
   │       └── article.pdf
   └── science/
       └── 2025-10-27/
           └── article.pdf
```

#### 2. Comando `schedule`

```bash
cargo run --bin start schedule
```

**O que faz:**
- Configura e executa tarefas agendadas
- Inclui execução do Collector Service
- Busca portais ativos do dashboard
- Executa collector para cada portal
- Downloads salvos em `downloads/<source>/<date>/`

#### 3. No Loop de Orquestração

```rust
fn run_orchestration_loop() {
    loop {
        // ... health checks ...
        
        // 3. Executar tarefas agendadas (incluindo Collector)
        println!("   ⏰ Checking scheduled tasks...");
        
        // Para cada tarefa agendada:
        // - Executa Collector Service
        // - Download de documentos
        // - Organiza em downloads/<origem>/<data>/
        // - Salva metadados
    }
}
```

---

## 🎛️ Comandos Disponíveis

### `cargo run --bin start start`
Inicia sistema completo (incluindo Collector Service)

### `cargo run --bin start backend`
Inicia apenas backend (inclui Collector)

### `cargo run --bin start frontend`
Inicia apenas dashboard

### `cargo run --bin start collector` ← NOVO!
Testa e mostra status do Collector Service

### `cargo run --bin start schedule` ← NOVO!
Executa tarefas agendadas (incluindo Collector)

### `cargo run --bin start status`
Verifica status de todos componentes (incluindo Collector)

### `cargo run --bin start monitor`
Monitora métricas em tempo real (incluindo Collector)

---

## 🧪 Testes e Debugging com Collector

### Testar Collector

```bash
cargo run --bin start collector
```

### Verificar Downloads

```powershell
Get-ChildItem -Path "G:\Hive-Hub\News-main\downloads" -Recurse
```

### Verificar Banco de Dados

```sql
SELECT * FROM raw_documents 
ORDER BY downloaded_at DESC 
LIMIT 10;
```

### Modo Debug

```bash
RUST_LOG=debug cargo run --bin start start
```

---

## 📊 Output Esperado com Collector

```
🚀 News System - Full Orchestrator
=====================================

📋 Step 1: Checking system dependencies...
✅ PostgreSQL - OK
✅ Rust - OK
✅ Node.js - OK
✅ npm - OK
✅ Downloads directory ready

🔧 Step 3: Starting Backend Server...
   ✅ Collector Service initialized
   ✅ Download directory: G:\Hive-Hub\News-main\downloads

⏰ Step 6: Configuring scheduler from dashboard...
   📅 Active portals: 2
   📊 Sources: Nature, Science, arXiv
   ⏰ Collection frequency: 60 minutes
   🔄 Collector ready for downloads
   🔄 Scheduler configured and ready

✅ News System is FULLY OPERATIONAL!
=====================================
   🔧 Backend API:    http://localhost:3001
   🎨 Dashboard:      http://localhost:1420
   🎯 Orchestrator:   ACTIVE
   ⏰ Scheduler:      CONFIGURED
   📊 Monitor:        RUNNING
   📥 Collector:       READY ← NOVO!

🔄 Orchestration Loop #1
   💚 Health check...
   ✅ Backend: Healthy
   ✅ Dashboard: Healthy
   ✅ Database: Connected
   ✅ Collector: Ready ← NOVO!
   ⏰ Checking scheduled tasks...
   📥 Collector running for Portal 1... ← NOVO!
   ✅ 12 documents downloaded ← NOVO!
```

---

## 🎯 Garantias do Orquestrador (com Collector)

O `start.rs` garante:

1. ✅ **Ordem correta** - Serviços iniciados na sequência certa
2. ✅ **Dependências** - Verifica tudo antes de iniciar
3. ✅ **Health checks** - Monitora saúde continuamente
4. ✅ **Reconfiguração** - Adapta-se a mudanças no dashboard
5. ✅ **Resiliência** - Retry automático em caso de falhas
6. ✅ **Graceful shutdown** - Para tudo corretamente
7. ✅ **Logging** - Registra todas as operações
8. ✅ **Métricas** - Coleta dados de performance
9. ✅ **Downloads Organizados** - Collector organiza por origem ← NOVO!
10. ✅ **Metadados Salvos** - Tudo registrado em raw_documents ← NOVO!

---

## 💡 Como Usar com Collector

```bash
cd G:\Hive-Hub\News-main
cargo run --bin start start
```

**Isso garante:**
- ✅ Tudo inicia na ordem certa
- ✅ Configurações coletadas do dashboard
- ✅ Collector Service ativo
- ✅ Downloads organizados em `downloads/<origem>/<data>/`
- ✅ Metadados salvos em `raw_documents`
- ✅ Módulos orquestrados até conclusão
- ✅ Monitoramento ativo

---

## 📝 Interação Detalhada com Collector

### Como funciona

1. **Backend inicia com Collector Service**
   ```rust
   let collector = CollectorService::new(
       db_pool,
       "G:/Hive-Hub/News-main/downloads"
   );
   ```

2. **Scheduler configura tarefas**
   - Detecta portais ativos
   - Configura frequências
   - Agenda execução do Collector

3. **Collector executa downloads**
   - Busca artigos das APIs
   - Faz download de PDFs
   - Organiza em `downloads/<origem>/<YYYY-MM-DD>/`
   - Salva metadados no banco

4. **Monitoramento contínuo**
   - Verifica status do Collector
   - Coleta métricas de downloads
   - Monitora espaço em disco
   - Alertas de erros

### Estrutura de Download

**Diretório base:** `G:\Hive-Hub\News-main\downloads`

**Organização:** `<origem>/<YYYY-MM-DD>/<arquivo>.pdf`

**Exemplos:**
- `downloads/arxiv/2025-10-27/article_001.pdf`
- `downloads/nature/2025-10-27/article_002.pdf`
- `downloads/science/2025-10-27/article_003.pdf`

---

**🎯 O orquestrador `start.rs` coordena todo o News System, incluindo o Collector Service para downloads automáticos organizados por origem! 🧠**

**📚 Veja documentação completa do Collector: `docs/PHASE1_COLLECTOR.md`**

