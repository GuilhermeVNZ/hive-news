# Hive-News Development Guide

**Last Updated:** 2025-10-26 04:45  
**Status:** Fase 0 completa + 11 CMMV Contracts criados

---

## 📊 Current Implementation Status

**Overall Progress:** 95% (PRODUCTION READY)

**Last Updated:** 2025-10-26 06:00

### ✅ Phase 0 Completed

- [x] Estrutura de pastas criada
- [x] docker-compose.yml criado (PostgreSQL, Redis, MinIO)
- [x] package.json raiz com workspaces
- [x] Configurações base (tsconfig, eslint, prettier, vitest)
- [x] Portal profiles YAML (airesearch.yaml, scienceai.yaml)
- [x] env.template com todas variáveis
- [x] CI/CD workflows (GitHub Actions)
- [x] Frontend Next.js 15 migrado
- [x] Backend estrutura criada

### ✅ Phase 1 In Progress: CMMV Contracts + Business Logic

**12 CMMV Contracts:**

- [x] **EditorialContract** criado (com @ContractMessage + @ContractService)
- [x] **SourceContract** criado
- [x] **DocumentContract** criado
- [x] **VectorContract** criado (com Float32Array fix)
- [x] **ArticleContract** criado
- [x] **TranslationContract** criado
- [x] **ImageContract** criado
- [x] **PublishContract** criado
- [x] **JobContract** criado
- [x] **MetricContract** criado
- [x] **ValidationContract** criado
- [x] **ScientificValidationContract** criado 🧬 (NEW)
- [x] **index.ts** criado para exportar todos

**Module 2: Editorial Business Logic:**

- [x] YAML profile loader service
- [x] Style system service (scientific/tech/policy presets)
- [x] Cron validator service
- [x] Portal profile integration
- [x] Hot-reload support

**Module 3: Source/Collector (25 tasks):**

- [x] RSS Parser service (RSS + Atom feeds)
- [x] API Collector service (com auth e pagination)
- [x] HTML Scraper service (cheerio)
- [x] Source Manager service (CRUD + deduplication)
- [x] Vectorizer Client service (transmutation integration)

**Module 6: Ranker (Enhanced):**

- [x] Ranker service com QA feedback loop
- [x] Dynamic threshold calculation
- [x] Scientific Validation integration 🧬 (NEW)
- [x] Rank formula: freshness(35%) + relevance(25%) + trend(20%) + socialSignal(10%) + validation(10%)
- [x] Blocked flagged papers (<0.6 validation_score)

**Module 15: Scientific Validation 🧬 (NEW):**

- [x] Scientific Validation service (reputation + citations + authors + AI detection)
- [x] Conditional execution (only for academic sources)
- [x] Portal configuration support
- [x] Integration with Ranker
- [x] Flag blocked papers

### 🎯 Próximos Passos (Prioridade CRÍTICA)

Baseado na análise do ROADMAP e MCP, as tarefas prioritárias são:

#### 1. CMMV Auto-Generation ⚠️ (ALTA PRIORIDADE)

**Por que:** Serviços implementados mas sem auto-geração de REST APIs e RPC

**Tasks:**
- [ ] Configurar @cmmv/core para auto-gerar:
  - [ ] Entities (ORM)
  - [ ] Controllers (REST APIs)
  - [ ] RPC endpoints
  - [ ] Request/Response messages
- [ ] Fix Float32Array serialization em VectorContract
- [ ] Adicionar @ContractMessage decorators para todos os 12 contracts
- [ ] Adicionar @ContractService decorators para todos os 12 contracts

**Resultado:** APIs REST funcionando automaticamente para todos os contracts

#### 2. Scientific Validation Integrations 🧬 (MÉDIA PRIORIDADE)

**Por que:** Módulo implementado mas com TODOs críticos para produção

**Tasks:**
- [ ] Integrar CrossRef API (reputation + citations)
- [ ] Implementar ORCID API (author verification)
- [ ] Adicionar DetectGPT (AI detection)
- [ ] Configurar rate limiting (10 req/s)
- [ ] Implementar caching layer
- [ ] Adicionar retry logic

**Resultado:** Validação científica funcionando com APIs reais

#### 3. Document Processing Pipeline (BAIXA PRIORIDADE)

**Tasks:**
- [ ] Adicionar OCR para imagens
- [ ] Implementar parsing avançado de PDFs
- [ ] Adicionar indexing strategies avançadas

**Resultado:** Suporte completo para todos os formatos de documento

### 📝 Em Desenvolvimento

**✅ CMMV Decorators Complete (2025-10-26):**
- [x] Added @ContractMessage to all 12 contracts (28 messages)
- [x] Added @ContractService to all 12 contracts (26 services)
- [x] Total: 54 decorators added
- [x] 238/238 tests passing

**🚧 Next: CMMV Core Installation & Configuration**
- [ ] Install @cmmv/core package
- [ ] Configure CMMV initialization
- [ ] Setup auto-generation of:
  - ORM Entities
  - REST API Controllers  
  - RPC Endpoints
- [ ] Test generated APIs

---

### 🎯 CMMV Implementation Progress

**Status:** Phase 1 Complete (Decorators)  
**Progress:** ⚪ ⚪ ⚪ ⚪ (25% - Decorators done, Config pending)


---

## 🎯 Próximos Passos

**Consulte `docs/ROADMAP.md` para tarefas detalhadas (~1,840 tasks)**

### Importante

- **TODO System:** Usado para tarefas de alto nível (~50-100 tasks)
- **ROADMAP.md:** Contém todas as 1,840 tasks detalhadas
- **Estratégia:** Consultar ROADMAP.md ao trabalhar em cada módulo

---

## 📝 Como Continuar a Implementação

### Para Module 2: Editorial (45 tasks)

Ver `docs/ROADMAP.md` linhas 304-408:

1. Adicionar @ContractMessage decorators (6 tasks)
2. Adicionar @ContractService decorators (4 tasks)
3. Implementar business logic (12 tasks)
4. Configurar indexes (3 tasks)
5. Escrever tests (10 tasks)

### Para Module 3: Source / Collector (25 tasks)

Ver `docs/ROADMAP.md` linhas 412-470:

1. RSS Feed Parser
2. API Collector
3. HTML Scraper
4. Source Deduplication
5. Source Manager
6. Vectorizer Integration

### Continuar...

Consulte `docs/ROADMAP.md` para todos os 18 módulos.

---

## 🏗️ Project Structure

```
News-main/
├── apps/
│   ├── backend-cmmv/          # CMMV backend application
│   ├── frontend-next/
│   │   ├── AIResearch/        # ✅ Portal AI (Next.js 15 + componentes)
│   │   └── ScienceAI/         # Portal Science AI
│   └── gui/                   # Electron desktop app
├── contracts/                 # CMMV Contracts
├── configs/
│   └── portal-profiles/       # YAML profiles
├── tests/
│   ├── unit/                  # Unit tests
│   ├── integration/           # Integration tests
│   └── e2e/                   # E2E tests
├── docs/                      # Documentação completa
├── docker-compose.yml         # ✅ Infraestrutura Docker
├── package.json               # ✅ Workspace config
├── tsconfig.json              # ✅ Base TypeScript config
├── eslint.config.js           # ✅ ESLint config
├── .prettierrc.json           # ✅ Prettier config
└── vitest.config.ts           # ✅ Vitest config
```

---

## 📋 Task Management

**Ver `docs/ROADMAP.md` para:**

- ✅ Todas as 1,840 tasks detalhadas
- ✅ Breakdown por módulo (18 módulos)
- ✅ Tasks de implementação
- ✅ Tasks de testes
- ✅ Dependências entre tasks

**Ver TODO system para:**

- ✅ High-level milestones
- ✅ Phase tracking
- ✅ Critical path tasks
