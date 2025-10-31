# Dashboard de Gerenciamento com Autenticação - Implementação

## ✅ O que foi implementado

### Backend (Rust + Axum)

#### 1. Autenticação JWT
- ✅ `src/utils/jwt.rs` - Gerar e verificar tokens JWT
- ✅ `src/routes/auth.rs` - Endpoints de autenticação:
  - `POST /api/auth/login` - Login com username/password
  - `POST /api/auth/logout` - Logout
  - `GET /api/auth/me` - Obter usuário atual
  - `POST /api/auth/change-password` - Alterar senha

#### 2. Gerenciamento de Collectors
- ✅ `src/utils/config_manager.rs` - Gerenciador de configuração JSON
- ✅ `src/routes/collectors.rs` - Endpoints para gerenciar collectors:
  - `GET /api/collectors` - Listar todos os collectors
  - `GET /api/collectors/enabled` - Listar apenas collectors ativos
  - `PUT /api/collectors/:id/status` - Ativar/desativar collector
  - `PUT /api/collectors/:id/config` - Atualizar configuração do collector

#### 3. Arquivo de Configuração
- ✅ `collectors_config.json` - JSON com configuração dos collectors
  - Lista de collectors disponíveis (arXiv, PMC, Semantic Scholar, etc.)
  - Status (enabled/disabled)
  - API keys (opcionais)
  - Configurações específicas (categoria, max_results, etc.)

### Frontend (React - ainda precisa ser implementado)

#### Pendente:
- ⏳ Página de Login (`src/pages/Login.tsx`)
- ⏳ Context de Autenticação (`src/context/AuthContext.tsx`)
- ⏳ Proteção de Rotas (`src/components/ProtectedRoute.tsx`)
- ⏳ Página Sources atualizada (`src/pages/Sources.tsx`)
  - Conectar com API real
  - Mostrar status dos collectors (enabled/disabled)
  - Toggle para ativar/desativar collectors
  - Formulário para configurar collectors

## 📋 Próximos passos

### 1. Criar página de login

### 2. Criar AuthContext para gerenciar estado de autenticação

### 3. Atualizar App.tsx para proteger rotas

### 4. Atualizar Sources.tsx para usar API real

## 🔐 Credenciais padrão

- **Username**: `admin`
- **Password**: `admin123`

⚠️ **IMPORTANTE**: Alterar estas credenciais em produção!

## 📝 Arquivo de configuração

O arquivo `collectors_config.json` será criado automaticamente no primeiro acesso com:

```json
{
  "collectors": [
    {
      "id": "arxiv",
      "name": "arXiv",
      "enabled": true,
      "api_key": null,
      "config": {
        "category": "cs.AI",
        "max_results": 10
      }
    },
    {
      "id": "pmc",
      "name": "PubMed Central",
      "enabled": false,
      "api_key": null,
      "config": {}
    },
    {
      "id": "semantic_scholar",
      "name": "Semantic Scholar",
      "enabled": false,
      "api_key": null,
      "config": {}
    }
  ],
  "updated_at": "2025-01-01T00:00:00Z"
}
```

## 🚀 Como usar

### 1. Iniciar backend:
```bash
cd news-backend
cargo run
```

### 2. Fazer login:
```bash
curl -X POST http://localhost:3001/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

### 3. Listar collectors:
```bash
curl http://localhost:3001/api/collectors \
  -H "Authorization: Bearer SEU_TOKEN"
```

### 4. Ativar/desativar collector:
```bash
curl -X PUT http://localhost:3001/api/collectors/arxiv/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer SEU_TOKEN" \
  -d '{"enabled": true}'
```

## 📚 Integração com start.rs

O `start.rs` precisa ser atualizado para ler `collectors_config.json` e executar apenas os collectors que estão `enabled: true`.














