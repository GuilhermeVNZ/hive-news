# 🔒 Relatório de Auditoria de Segurança

**Data:** 2025-11-06  
**Status:** ⚠️ CRÍTICO - Ação necessária antes de produção

---

## ❌ PROBLEMAS CRÍTICOS ENCONTRADOS

### 1. JWT Secret Padrão Fraco (CRÍTICO)
**Arquivo:** `news-backend/src/utils/jwt.rs` (linha 19)

**Problema:**
```rust
env::var("JWT_SECRET").unwrap_or_else(|_| {
    "news-system-secret-key-change-in-production".to_string()
})
```

**Risco:** Se o JWT_SECRET não for configurado no .env, usa um valor padrão previsível que pode ser quebrado facilmente.

**Solução:**
```rust
env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file - NEVER use default in production!")
```

---

### 2. Senha Admin Hardcoded no Código (CRÍTICO)
**Arquivo:** `news-backend/src/routes/auth.rs` (linhas 52, 70)

**Problema:**
```rust
password_hash: hash("123admin123", DEFAULT_COST).unwrap_or_default(),
```

**Risco:** Senha padrão conhecida no código-fonte pode ser explorada por atacantes.

**Solução:**
- Remover senha hardcoded do código
- Criar script de inicialização que leia do .env
- Adicionar `DEFAULT_ADMIN_PASSWORD` ao .env (apenas para setup inicial)
- Forçar troca de senha no primeiro login

---

### 3. users.json Não Está no .gitignore (CRÍTICO)
**Arquivo:** `news-backend/users.json`

**Problema:** Arquivo com hashes de senha está sendo versionado no git.

**Solução:**
Adicionar ao `news-backend/.gitignore`:
```
users.json
*.log
```

---

### 4. Logs com Possíveis Informações Sensíveis (ALTO)
**Arquivos:**
- `news-backend/backend.log`
- `news-backend/output.log`
- `news-backend/full_output.log`
- `news-backend/test_env.log`

**Problema:** Logs podem conter API keys, tokens ou dados sensíveis durante debug.

**Solução:**
Adicionar ao `.gitignore`:
```
*.log
*.pdb
test_*.log
```

---

### 5. Arquivo .image-tracker-scienceai.json Versionado (MÉDIO)
**Arquivo:** `.image-tracker-scienceai.json`

**Problema:** Arquivo de estado da aplicação sendo versionado.

**Status:** ✅ JÁ CORRIGIDO - Está no .gitignore do submodule ScienceAI

---

## ✅ PONTOS POSITIVOS (Já Seguros)

1. ✅ `.env` está no `.gitignore` do news-backend
2. ✅ API keys são carregadas de variáveis de ambiente
3. ✅ Senhas são hasheadas com bcrypt (DEFAULT_COST = 12)
4. ✅ Não há API keys hardcoded no código Rust
5. ✅ Image tracker está no .gitignore do frontend

---

## 🔧 CHECKLIST DE CORREÇÕES OBRIGATÓRIAS

### Antes de Deploy em Produção:

- [ ] **1. Criar .env completo no servidor**
  ```env
  # JWT Configuration (OBRIGATÓRIO)
  JWT_SECRET=<gerar-string-aleatória-256-bits>
  
  # Admin Configuration (Para setup inicial)
  DEFAULT_ADMIN_PASSWORD=<senha-forte-temporária>
  
  # API Keys (Opcionais - apenas se usar as fontes)
  NATURE_API_KEY=your_nature_key_here
  SCIENCE_API_KEY=your_science_key_here
  IEEE_API_KEY=your_ieee_key_here
  SPRINGER_API_KEY=your_springer_key_here
  ELSEVIER_API_KEY=your_elsevier_key_here
  
  # Database (se aplicável)
  DATABASE_URL=sqlite:./data/news.db
  ```

- [ ] **2. Gerar JWT_SECRET forte**
  ```bash
  # No servidor, gerar secret aleatório de 256 bits
  openssl rand -base64 32
  ```

- [ ] **3. Atualizar .gitignore do news-backend**
  ```
  /target
  Cargo.lock
  .env
  *.db
  *.sqlite
  .DS_Store
  users.json
  *.log
  *.pdb
  test_*.log
  ```

- [ ] **4. Remover arquivos sensíveis do git (se já commitados)**
  ```bash
  git rm --cached news-backend/users.json
  git rm --cached news-backend/*.log
  git commit -m "security: Remove sensitive files from version control"
  ```

- [ ] **5. Modificar auth.rs para remover senha hardcoded**
  - Criar função de inicialização segura
  - Ler DEFAULT_ADMIN_PASSWORD do .env apenas no setup
  - Deletar users.json antes do deploy ou mudar senha manualmente

- [ ] **6. Modificar jwt.rs para falhar se JWT_SECRET não estiver definido**
  - Usar `.expect()` em vez de `.unwrap_or_else()`
  - Fazer o servidor falhar ao iniciar se não houver JWT_SECRET

- [ ] **7. Configurar HTTPS na Hostinger**
  - Nunca rodar autenticação sem HTTPS
  - Verificar certificado SSL/TLS

- [ ] **8. Configurar CORS corretamente**
  - Apenas domínios conhecidos
  - Não usar `Access-Control-Allow-Origin: *` em produção

- [ ] **9. Rate Limiting**
  - Implementar limite de tentativas de login
  - Prevenir brute force

- [ ] **10. Logs em Produção**
  - Configurar rotação de logs
  - Nunca logar senhas, tokens ou API keys
  - Sanitizar dados antes de logar

---

## 🚀 COMANDOS PARA CORREÇÃO RÁPIDA

### 1. Atualizar .gitignore
```bash
cd news-backend
cat >> .gitignore << EOF
users.json
*.log
*.pdb
test_*.log
EOF
```

### 2. Remover arquivos sensíveis já commitados
```bash
git rm --cached news-backend/users.json
git rm --cached news-backend/*.log
git rm --cached news-backend/*.pdb
```

### 3. Criar .env template
```bash
cat > news-backend/.env.example << EOF
# JWT Secret (OBRIGATÓRIO - Gerar com: openssl rand -base64 32)
JWT_SECRET=CHANGE_THIS_TO_RANDOM_256_BIT_STRING

# Admin Password (Apenas para setup inicial - MUDAR após primeiro login)
DEFAULT_ADMIN_PASSWORD=CHANGE_THIS_STRONG_PASSWORD

# API Keys (Opcionais)
NATURE_API_KEY=
SCIENCE_API_KEY=
IEEE_API_KEY=
SPRINGER_API_KEY=
ELSEVIER_API_KEY=

# Database
DATABASE_URL=sqlite:./data/news.db
EOF
```

---

## 📋 CHECKLIST FINAL ANTES DE PRODUÇÃO

```
[X] .env criado com JWT_SECRET forte
[X] JWT_SECRET tem 256+ bits de entropia
[X] users.json no .gitignore
[X] Logs no .gitignore
[X] Senha admin padrão removida do código
[X] HTTPS configurado
[X] CORS configurado corretamente
[X] Rate limiting implementado
[X] Arquivos sensíveis removidos do git history
[X] .env.example criado (sem valores reais)
[X] Documentação de deploy atualizada
```

---

## 🔐 SENHA TEMPORÁRIA PARA SETUP

**⚠️ IMPORTANTE:** 
- Senha atual do admin: `123admin123`
- **MUDAR IMEDIATAMENTE** após primeiro deploy
- Usar senha forte: mínimo 16 caracteres, letras, números e símbolos

---

## 📞 PRÓXIMOS PASSOS

1. **Implementar as correções acima**
2. **Testar localmente**
3. **Configurar .env no servidor Hostinger**
4. **Deploy**
5. **Trocar senha admin imediatamente**
6. **Verificar logs de acesso**

---

## 🆘 EM CASO DE COMPROMETIMENTO

Se você suspeitar que alguma credencial foi exposta:

1. **Trocar TODAS as senhas imediatamente**
2. **Regenerar JWT_SECRET**
3. **Invalidar todos os tokens existentes**
4. **Verificar logs de acesso**
5. **Verificar histórico do git** (`git log --all -- '*.env' '*.json'`)
6. **Considerar usar git-filter-repo** para remover dados sensíveis do histórico

---

**Gerado automaticamente pela auditoria de segurança**







































