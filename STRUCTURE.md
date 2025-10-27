# 📁 News System - Estrutura do Projeto

## 🎯 Arquivo Principal

**Todos os testes começam a partir de:**
```
News-main/start.rs
```

## 📂 Estrutura Atual

```
News-main/
├── start.rs                      # 🎯 Entry point para TODOS os testes
├── Cargo.toml                    # Binário start
├── TESTING_GUIDE.md              # Guia de testes
├── STRUCTURE.md                  # Este arquivo
│
├── docs/                         # 📚 Documentação
│   └── PHASE1_ETAPA1_COLLECTOR.md
│
├── news-backend/                 # 🔧 Backend Rust
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # Servidor Axum
│   │   ├── db/                   # Camada de banco
│   │   ├── routes/               # Endpoints API
│   │   │   ├── auth.rs
│   │   │   ├── pages.rs
│   │   │   ├── sources.rs
│   │   │   ├── logs.rs
│   │   │   └── collector.rs      ✅ Etapa 1 - Rotas
│   │   ├── services/             # Lógica de negócios
│   │   │   ├── auth_service.rs
│   │   │   ├── page_service.rs
│   │   │   ├── collector_service.rs    ✅ Etapa 1 - Service
│   │   │   └── scheduler_service.rs   ✅ Etapa 1 - Scheduler
│   │   ├── models/               # Modelos de dados
│   │   │   └── raw_document.rs        ✅ Etapa 1 - Model
│   │   ├── middleware/           # Middleware customizado
│   │   └── utils/                # Utilitários
│   └── migrations/
│       ├── 001_create_tables.sql
│       └── 002_create_raw_documents.sql ✅ Etapa 1 - Schema
│
├── news-dashboard/               # 🎨 Frontend React + Tauri
│   ├── package.json
│   ├── src/
│   │   ├── components/
│   │   │   ├── Layout.tsx       # Sidebar
│   │   │   └── ui/
│   │   │       ├── button.tsx
│   │   │       ├── card.tsx
│   │   │       └── badge.tsx
│   │   ├── pages/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── PagesConfig.tsx
│   │   │   ├── Sources.tsx
│   │   │   └── Logs.tsx
│   │   └── lib/
│   │       └── utils.ts
│   └── src-tauri/               # Tauri config
│
└── apps/                         # 📱 Portais de conteúdo
    └── frontend-next/
        └── AIResearch/          # Portal existente
```

## 🧩 Etapas do Sistema

### ✅ Etapa 0: Dashboard de Controle
- **Localização**: `news-backend/` + `news-dashboard/`
- **Status**: Implementada com identidade visual
- **Funcionalidades**: 
  - Backend API
  - Dashboard admin
  - Configuração de páginas

### ✅ Etapa 1: Collector (Coleta e Download)
- **Localização**: `news-backend/src/services/collector_service.rs`
- **Status**: Estrutura criada, pendente implementação
- **Funcionalidades**:
  - Download de documentos
  - Scheduler de tarefas
  - API endpoints

### 📋 Próximas Etapas
- ⏳ Etapa 2: Extração de texto
- ⏳ Etapa 3: Embedding e indexação
- ⏳ Etapa 4: Rankeamento
- ⏳ Etapa 5: Geração de conteúdo

## 🧪 Como Testar

### 1. Via start.rs (Recomendado)
```bash
cd News-main
cargo run -- start backend
cargo run -- start frontend
cargo run -- start collector
```

### 2. Diretamente
```bash
# Backend
cd News-main/news-backend
cargo run

# Frontend
cd News-main/news-dashboard
npm run dev
```

## 📝 Documentação

- `TESTING_GUIDE.md`: Como usar o sistema de testes
- `docs/PHASE1_ETAPA1_COLLECTOR.md`: Detalhes da Etapa 1
- `STRUCTURE.md`: Este arquivo

## 🚀 Endpoints API

### Backend (http://localhost:3001)
```
POST   /api/auth/login
POST   /api/auth/logout
GET    /api/auth/me

GET    /api/pages
POST   /api/pages
GET    /api/pages/:id
PUT    /api/pages/:id
DELETE /api/pages/:id

GET    /api/sources
POST   /api/sources

GET    /api/logs

POST   /api/collector/start              ✅ Etapa 1
GET    /api/collector/status/:portal_id  ✅ Etapa 1
GET    /api/collector/logs              ✅ Etapa 1
```

### Frontend (http://localhost:1420)
```
/           Dashboard
/pages      Pages Configuration
/sources    Sources Management
/logs       Collection Logs
```

## 📊 Banco de Dados

### Tabelas Criadas
1. `pages_config` - Configuração de portais
2. `users` - Usuários e autenticação
3. `collection_logs` - Logs de coleta
4. `raw_documents` - Documentos baixados ✅ Etapa 1

## 🎯 Resumo Etapa 1

### Arquivos Criados
- ✅ `src/services/collector_service.rs`
- ✅ `src/services/scheduler_service.rs`
- ✅ `src/models/raw_document.rs`
- ✅ `src/routes/collector.rs`
- ✅ `migrations/002_create_raw_documents.sql`
- ✅ `docs/PHASE1_ETAPA1_COLLECTOR.md`

### Status
- ✅ Estrutura criada
- ✅ Compilação OK
- ⏳ Implementação pendente
- ⏳ Testes pendentes

---

**🎯 Sempre inicie os testes com `cargo run -- start [comando]`**

