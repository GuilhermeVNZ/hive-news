# 🔄 Rebuild Docker Sem Cache - Guia Completo

Este guia documenta os comandos necessários para garantir que os frontends sejam reconstruídos sem usar cache, garantindo que todas as mudanças sejam aplicadas corretamente.

## ⚠️ Problema

Quando mudanças são feitas no frontend mas o Docker usa cache:
- O servidor pode estar rodando versão antiga do código
- Mudanças não aparecem mesmo após rebuild
- Assets JS/CSS são servidos do cache do navegador

## ✅ Solução: Rebuild Completo Sem Cache

### 1. Parar Todos os Containers

```bash
docker-compose down
```

### 2. Remover Cache do Docker Build

```bash
# Remover todas as imagens relacionadas ao projeto
docker-compose down --rmi all

# OU remover apenas imagens órfãs
docker image prune -f

# OU remover tudo (mais agressivo)
docker system prune -a -f
```

### 3. Rebuild SEM CACHE

```bash
# Rebuild TODOS os serviços sem cache
docker-compose build --no-cache

# OU rebuild apenas serviços específicos
docker-compose build --no-cache airesearch
docker-compose build --no-cache scienceai
docker-compose build --no-cache dashboard
docker-compose build --no-cache backend
```

### 4. Subir os Containers

```bash
# Subir todos os serviços
docker-compose up -d

# OU subir apenas serviços específicos
docker-compose up -d airesearch scienceai dashboard backend
```

## 📋 Script Completo (One-Liner)

### Windows PowerShell
```powershell
cd G:\Hive-Hub\News-main
docker-compose down
docker-compose build --no-cache airesearch scienceai dashboard
docker-compose up -d
```

### Linux/Mac Bash
```bash
cd /path/to/News-main
docker-compose down
docker-compose build --no-cache airesearch scienceai dashboard
docker-compose up -d
```

## 🔍 Verificar se Funcionou

### 1. Verificar Logs dos Containers

```bash
# Ver logs do airesearch
docker-compose logs -f airesearch

# Ver logs do scienceai
docker-compose logs -f scienceai

# Ver logs do dashboard
docker-compose logs -f dashboard
```

### 2. Verificar Versão do Build

Os arquivos JS/CSS devem ter novos hashes nos nomes. Verifique:
- `assets/index-XXXXX.js` (hash diferente)
- `assets/index-XXXXX.css` (hash diferente)

### 3. Limpar Cache do Navegador

**Chrome/Edge:**
- `Ctrl + Shift + R` (Windows/Linux)
- `Cmd + Shift + R` (Mac)
- Ou DevTools → Network → Disable cache

**Firefox:**
- `Ctrl + Shift + R` (Windows/Linux)
- `Cmd + Shift + R` (Mac)

## 🐛 Troubleshooting

### Problema: Mudanças ainda não aparecem

1. **Verificar se código foi commitado:**
   ```bash
   git status
   git log --oneline -5
   ```

2. **Verificar se arquivos foram copiados no Dockerfile:**
   - Verificar `COPY` commands no Dockerfile
   - Verificar se `APP_DIR` está correto

3. **Verificar se build foi executado:**
   ```bash
   docker-compose exec airesearch ls -la /app/.next/static
   ```

4. **Limpar cache do nginx dentro do container:**
   ```bash
   docker-compose exec airesearch rm -rf /var/cache/nginx/*
   docker-compose restart airesearch
   ```

### Problema: Nginx servindo arquivos antigos

1. **Verificar configuração do nginx:**
   - Assets JS/CSS têm cache de 1 ano
   - Nomes de arquivo com hash devem mudar automaticamente

2. **Forçar reload do nginx:**
   ```bash
   docker-compose exec airesearch nginx -s reload
   ```

## 📝 Notas Importantes

- **Cache do Docker Build**: `--no-cache` força rebuild completo, mas é mais lento
- **Cache do Navegador**: Assets com hash no nome são cache-busted automaticamente
- **Cache do Nginx**: Arquivos estáticos são servidos com headers de cache longo (isso é bom!)
- **ISR (Next.js)**: Páginas são revalidadas em background mesmo com cache

## 🚀 Comandos Rápidos

```bash
# Rebuild rápido (usa cache)
docker-compose build && docker-compose up -d

# Rebuild completo (sem cache)
docker-compose build --no-cache && docker-compose up -d

# Rebuild apenas frontends
docker-compose build --no-cache airesearch scienceai dashboard && docker-compose up -d

# Ver versões dos containers
docker-compose ps

# Ver logs em tempo real
docker-compose logs -f
```

