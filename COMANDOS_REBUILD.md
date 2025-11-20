# 🔄 Comandos para Rebuild Sem Cache

## ⚡ Script Automático (Recomendado)

Execute o script PowerShell que faz tudo automaticamente:

```powershell
cd G:\Hive-Hub\News-main
.\rebuild-without-cache.ps1
```

## 📋 Comandos Manuais

### 1. Parar containers e rebuild sem cache

```powershell
cd G:\Hive-Hub\News-main
docker-compose down
docker-compose build --no-cache airesearch scienceai dashboard
docker-compose up -d
```

### 2. Rebuild apenas um serviço específico

```powershell
# Apenas airesearch
docker-compose build --no-cache airesearch
docker-compose up -d airesearch

# Apenas scienceai
docker-compose build --no-cache scienceai
docker-compose up -d scienceai

# Apenas dashboard
docker-compose build --no-cache dashboard
docker-compose up -d dashboard
```

### 3. Rebuild COMPLETO (incluindo backend)

```powershell
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### 4. Limpar cache do Docker (mais agressivo)

```powershell
# Parar e remover tudo
docker-compose down --rmi all

# Rebuild sem cache
docker-compose build --no-cache

# Subir
docker-compose up -d
```

### 5. Verificar se funcionou

```powershell
# Ver logs
docker-compose logs -f airesearch
docker-compose logs -f scienceai

# Ver status
docker-compose ps

# Ver arquivos dentro do container
docker-compose exec airesearch ls -la /app/.next/static/chunks/
```

## 🌐 Limpar Cache do Navegador

### Chrome/Edge
- `Ctrl + Shift + R` (hard refresh)
- `F12` → Network → ☑️ Disable cache

### Firefox
- `Ctrl + Shift + R` (hard refresh)
- `F12` → Network → ☑️ Disable cache

### DevTools
Abra o DevTools (`F12`) e:
1. Network tab
2. Marque "Disable cache"
3. Recarregue a página (`Ctrl + Shift + R`)

## ✅ Checklist de Verificação

- [ ] Código foi commitado no Git
- [ ] Push foi feito para o repositório remoto
- [ ] Containers foram rebuildados com `--no-cache`
- [ ] Cache do navegador foi limpo (`Ctrl + Shift + R`)
- [ ] Arquivos JS/CSS têm novos hashes (verificar no Network tab)
- [ ] Logs não mostram erros

## 🐛 Problemas Comuns

### "Mudanças ainda não aparecem"

1. **Verificar se código está no servidor:**
   ```powershell
   # Ver arquivos dentro do container
   docker-compose exec airesearch cat /app/src/components/HomeClient.tsx
   ```

2. **Verificar se build foi feito:**
   ```powershell
   docker-compose exec airesearch ls -la /app/.next/static/
   ```

3. **Limpar cache do nginx:**
   ```powershell
   docker-compose exec airesearch rm -rf /var/cache/nginx/*
   docker-compose restart airesearch
   ```

### "Docker está usando cache mesmo com --no-cache"

Isso pode acontecer se as layers anteriores estão em cache. Solução:

```powershell
# Remover TODAS as imagens do projeto
docker-compose down --rmi all
docker system prune -f

# Rebuild completo
docker-compose build --no-cache
docker-compose up -d
```

## 📝 Notas

- `--no-cache` força rebuild completo (mais lento, mas garante mudanças)
- Cache do navegador pode persistir mesmo após rebuild
- Assets JS/CSS com hash no nome devem mudar automaticamente
- Next.js ISR revalida em background mesmo com cache

