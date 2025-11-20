# ✅ Checklist de Deploy - Garantir Frontend Sem Cache

## ⚠️ IMPORTANTE: Há commits locais não enviados!

Verifique com: `git log origin/main..HEAD --oneline`

---

## 📋 Passo a Passo Completo

### 1. ✅ Verificar e Commitar Mudanças Locais

```powershell
cd G:\Hive-Hub\News-main

# Ver mudanças não commitadas
git status

# Se houver mudanças, commitar
git add .
git commit -m "feat: atualizações pendentes"
```

### 2. 📤 Enviar para Git (Remoto)

```powershell
# Ver commits locais não enviados
git log origin/main..HEAD --oneline

# Push para o remoto
git push origin main

# OU se a branch for diferente:
git push origin master
```

### 3. 🔄 No Servidor: Rebuild SEM CACHE

#### Opção A: Script Automático (Recomendado)

```bash
cd /path/to/News-main
./rebuild-without-cache.sh
```

#### Opção B: Comandos Manuais

```bash
cd /path/to/News-main

# 1. Parar containers
docker-compose down

# 2. Pull do Git (importante!)
git pull origin main

# 3. Rebuild SEM CACHE
docker-compose build --no-cache airesearch scienceai dashboard

# 4. Subir containers
docker-compose up -d

# 5. Verificar logs
docker-compose logs -f airesearch
docker-compose logs -f scienceai
```

### 4. 🌐 Limpar Cache do Navegador

**Obrigatório após rebuild!**

- **Chrome/Edge**: `Ctrl + Shift + R` (ou `Cmd + Shift + R` no Mac)
- **Firefox**: `Ctrl + Shift + R`
- **DevTools**: `F12` → Network → ☑️ Disable cache → `Ctrl + Shift + R`

### 5. ✅ Verificar se Funcionou

#### Verificar Versão do Build

1. Abra DevTools (`F12`)
2. Vá para Network tab
3. Recarregue a página (`Ctrl + Shift + R`)
4. Procure por arquivos JS/CSS:
   - `assets/index-XXXXX.js` (hash deve ser diferente)
   - `assets/index-XXXXX.css` (hash deve ser diferente)

#### Verificar Logs dos Containers

```bash
# Ver logs em tempo real
docker-compose logs -f airesearch
docker-compose logs -f scienceai

# Ver últimas 50 linhas
docker-compose logs --tail=50 airesearch
```

#### Verificar Arquivos Dentro dos Containers

```bash
# Ver arquivos do Next.js (airesearch)
docker-compose exec airesearch ls -la /app/.next/static/chunks/

# Ver arquivos do Vite (scienceai)
docker-compose exec scienceai ls -la /usr/share/nginx/html/assets/
```

---

## 🚀 Comando Rápido (One-Liner)

### No Local (Windows PowerShell)
```powershell
cd G:\Hive-Hub\News-main
git status
git add .
git commit -m "feat: atualizações"
git push origin main
```

### No Servidor (Linux/Mac)
```bash
cd /path/to/News-main
git pull origin main
docker-compose down
docker-compose build --no-cache airesearch scienceai dashboard
docker-compose up -d
docker-compose logs -f
```

---

## 🐛 Troubleshooting

### Problema: "Mudanças ainda não aparecem no servidor"

**Causa 1: Código não foi enviado para Git**
```powershell
# Verificar commits locais não enviados
git log origin/main..HEAD --oneline

# Se houver commits, fazer push
git push origin main
```

**Causa 2: Servidor não fez pull do Git**
```bash
# No servidor, verificar se código está atualizado
cd /path/to/News-main
git pull origin main
git log --oneline -5
```

**Causa 3: Docker usou cache**
```bash
# Rebuild FORÇADO sem cache
docker-compose down
docker-compose build --no-cache airesearch scienceai dashboard
docker-compose up -d
```

**Causa 4: Cache do navegador**
- Limpe cache do navegador (`Ctrl + Shift + R`)
- Ou use modo anônimo/privado para testar

### Problema: "Docker ainda usa cache mesmo com --no-cache"

```bash
# Solução mais agressiva
docker-compose down --rmi all
docker system prune -f
docker-compose build --no-cache
docker-compose up -d
```

### Problema: "Assets JS/CSS não mudam de hash"

Isso é normal se não houver mudanças no código. Para forçar mudança:

1. Fazer uma mudança trivial no código (ex: adicionar comentário)
2. Commitar e fazer push
3. Rebuild sem cache
4. Verificar novo hash

---

## 📝 Checklist Final

- [ ] Código foi commitado localmente
- [ ] Push foi feito para o repositório remoto
- [ ] Servidor fez `git pull` do repositório
- [ ] Containers foram rebuildados com `--no-cache`
- [ ] Logs não mostram erros
- [ ] Cache do navegador foi limpo
- [ ] Arquivos JS/CSS têm novos hashes (se houve mudanças)
- [ ] Site está funcionando corretamente

---

## 🔍 Verificação Rápida

### Verificar Commits Locais vs Remoto
```powershell
git log origin/main..HEAD --oneline
```

### Verificar Status do Git
```powershell
git status
```

### Verificar Última Atualização no Servidor
```bash
# No servidor
cd /path/to/News-main
git log --oneline -1
git status
```

### Verificar Build dos Containers
```bash
docker-compose ps
docker-compose logs --tail=20 airesearch
docker-compose logs --tail=20 scienceai
```

---

## 💡 Dicas

1. **Sempre faça `git pull` no servidor antes de rebuild**
2. **Use `--no-cache` quando houver mudanças no código**
3. **Limpe cache do navegador após rebuild**
4. **Verifique logs após rebuild para garantir que não há erros**
5. **Use DevTools → Network para verificar novos hashes de assets**

---

## 📚 Documentação Relacionada

- `REBUILD_WITHOUT_CACHE.md` - Documentação completa sobre rebuild
- `COMANDOS_REBUILD.md` - Lista de comandos úteis
- `rebuild-without-cache.ps1` - Script automático para Windows

