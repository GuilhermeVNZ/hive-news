# 🚀 News System - Sistema de Gestão de Notícias

Sistema unificado para gerenciar múltiplos portais de conteúdo com coleta, curadoria e distribuição automatizadas.

## 📋 Visão Geral

O News System é composto por:

1. **Dashboard de Controle** (`news-dashboard/`) - Interface administrativa
2. **Backend API** (`news-backend/`) - API RESTful e serviços
3. **Orquestrador** (`start.rs`) - Cérebro central do sistema
4. **Portais** (`apps/frontend-next/`) - Portais de conteúdo

## 🎯 Quick Start

### Iniciar Sistema Completo

```powershell
cd G:\Hive-Hub\News-main
cargo run --bin start start
```

Ou use os scripts:
```powershell
.\start-system.ps1
```

```cmd
start-system.bat
```

## 🔧 Componentes

### 🎨 Dashboard (`news-dashboard/`)

Interface administrativa React + Tauri para:
- Gerenciar páginas (AIResearch, ScienceAI)
- Configurar fontes de notícias
- Definir intervalos de coleta
- Vincular redes sociais
- Visualizar logs e status

**Acesse**: http://localhost:1420

### 🔧 Backend (`news-backend/`)

API RESTful em Rust + Axum:
- Autenticação JWT
- CRUD de páginas
- Gerenciamento de fontes
- Serviços de coleta
- Scheduler de tarefas

**Acesse**: http://localhost:3001

### ⚙️ Orquestrador (`start.rs`)

Cérebro central que:
- ✅ Inicia todos os serviços
- ✅ Coleta configurações do dashboard
- ✅ Orquestra módulos até conclusão
- ✅ Monitora saúde do sistema

## 📚 Documentação

Todos os documentos principais estão em `docs/`:

- **Orquestrador**: `docs/ORCHESTRATOR.md` - Sistema de orquestração central
- **Coletor**: `docs/PHASE1_COLLECTOR.md` - Coleta de documentos (arXiv, Nature, Science, etc.)
- **Arquitetura**: `docs/ARCHITECTURE.md` - Visão geral do sistema
- **Testes**: `docs/TESTING_GUIDE.md` - Guia de testes

Ver índice completo: `docs/README.md`

## 🎯 Comandos Disponíveis

### Sistema
```bash
cargo run --bin start start       # Sistema completo
cargo run --bin start backend     # Apenas backend
cargo run --bin start frontend    # Apenas dashboard
cargo run --bin start status      # Verificar status
```

### Pipeline de News
```bash
cd news-backend
cargo run --bin news-backend -- pipeline           # Pipeline completo (collect → filter → write → cleanup)
cargo run --bin news-backend -- pipeline-debug     # Pipeline com logging ultra-detalhado
cargo run --bin news-backend -- write-news         # Processar apenas news (JSONs de RSS/HTML)
cargo run --bin news-backend -- cleanup-news       # Limpar apenas news processadas
```

### Pipeline de Artigos (PDFs)
```bash
cd news-backend
cargo run --bin news-backend -- write              # Processar PDFs aprovados (filtered/)
```

### Utilitários
```bash
cd news-backend
cargo run --bin clean-articles                     # Limpar formatação markdown de todos os artigos
cargo run --bin clean-articles -- <diretório>      # Limpar artigo específico
```

## 🏗️ Arquitetura

```
┌─────────────────────────────────────┐
│      ORQUESTRADOR (start.rs)       │
│                                     │
│  Gerencia ciclo de vida             │
│  Coordena módulos                   │
│  Monitora saúde                     │
└─────┬──────────────┬──────────────┘
      │              │
      ▼              ▼
  ┌──────────┐   ┌──────────────┐
  │ Backend  │   │ Frontends    │
  │  :3001   │   │  :1420 / 8080│
  └──────────┘   └──────────────┘
```

## 🚀 Próximos Passos

Veja `docs/PHASE1_ETAPA1_COLLECTOR.md` para implementar a coleta de documentos.

---

**Inicie com: `cargo run --bin start start`** 🎯
