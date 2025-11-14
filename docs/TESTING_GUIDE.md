# 🧪 News System - Testing Guide

## 🎯 Arquivo Principal de Testes

Todos os testes começam a partir de `start.rs` localizado em:
```
News-main/start.rs
```

## 🚀 Como Usar

### Compilar e Executar

```bash
cd News-main
cargo run -- start
```

### Comandos Disponíveis

#### 1. **Backend Server**
```bash
cargo run -- backend
```
Inicia o servidor backend em `http://localhost:3001`

#### 2. **Frontend Dashboard**
```bash
cargo run -- frontend
```
Inicia o dashboard em `http://localhost:1420`

#### 3. **Collector Service**
```bash
cargo run -- collector
```
Testa o serviço de coleta de documentos

#### 4. **Run All Tests**
```bash
cargo run -- test-all
```
Executa toda a suite de testes

## 📁 Estrutura de Testes

```
News-main/
├── start.rs                      # 🎯 Entry point para todos os testes
├── Cargo.toml                    # Binário start
├── tests/                        # Testes integrados
│   ├── integration/
│   │   ├── collector_test.rs
│   │   ├── scheduler_test.rs
│   │   └── api_test.rs
│   └── unit/
│       ├── services_test.rs
│       └── models_test.rs
├── news-backend/                 # Backend (Rust + Axum)
│   └── tests/
│       └── backend_test.rs
└── news-dashboard/               # Frontend (React + Tauri)
    └── tests/
        └── frontend_test.rs
```

## 🧪 Executando Testes

### Backend Tests
```bash
cd News-main
cargo test --package news-backend
```

### Integration Tests
```bash
cd News-main
cargo test --test collector_test
```

### Frontend Tests
```bash
cd News-main/news-dashboard
npm test
```

#### AIResearch (Next.js)
```bash
cd News-main/apps/frontend-next/airesearch
npm run type-check
npm run lint
npm test
npm run test:coverage
```

## 📝 Criando Novos Testes

### Para adicionar um novo teste:

1. **Crie o arquivo de teste**:
```rust
// tests/integration/my_test.rs

#[tokio::test]
async fn test_my_feature() {
    // Your test code here
}
```

2. **Registre em `start.rs`**:
```rust
match command {
    "my-test" => {
        println!("🧪 Running my test...");
        // Run test
    }
    // ...
}
```

## 🔍 Estrutura dos Testes

### 1. **Unit Tests**
Testam componentes individuais:
- Services
- Models
- Utilities

### 2. **Integration Tests**
Testam integração entre componentes:
- Collector → Database
- API → Collector
- Scheduler → Services

### 3. **E2E Tests**
Testam fluxos completos:
- Coleta completa
- Dashboard → Backend
- Fluxo end-to-end

## 🎯 Workflow de Testes

1. **Desenvolvimento**: Testes unitários e de integração
2. **Pre-Commit**: Todos os testes devem passar
3. **CI/CD**: Executa toda a suite de testes
4. **Produção**: Monitoramento e testes de regressão

## 📊 Coverage

Meta de cobertura: **95%+**

```bash
# Verificar cobertura
cargo llvm-cov --html
```

## 🚨 Testes Críticos

### Backend
- ✅ Autenticação (JWT)
- ✅ CRUD de páginas
- ✅ Collector service
- ✅ Database transactions

### Frontend
- ✅ Componentes UI
- ✅ Navegação
- ✅ Integração com API

### Integration
- ✅ Coleta de documentos
- ✅ Agendamento de tarefas
- ✅ Download de arquivos

## 🛠️ Debugging Tests

### Runners em Background
```bash
# Terminal 1: Backend
cd news-backend && cargo run

# Terminal 2: Frontend
cd news-dashboard && npm run dev

# Terminal 3: Start
cargo run -- start test-all
```

### Logs
```bash
# Logs do backend
RUST_LOG=debug cargo run

# Logs do frontend
npm run dev -- --debug
```

## 📚 Próximos Passos

- [ ] Implementar testes para Collector
- [ ] Adicionar E2E tests
- [ ] Configurar CI/CD
- [ ] Documentar casos de teste

---

**Sempre inicie os testes a partir de `start.rs`** 🎯

