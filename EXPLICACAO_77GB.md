# 🔍 Explicação: Por que 77GB se o projeto tem ~2GB?

## ❓ O Problema

Você está certo! A matemática não bate:
- Projeto (`~/hive-news`): ~2GB
- Sistema usando: **77GB de 96GB**

**Onde estão os outros ~75GB?**

## 🎯 Resposta: Docker e Sistema

O `du -sh *` só mostra o diretório atual do projeto. Os 77GB estão **fora** do diretório do projeto, principalmente em:

### 1. **Docker** (`/var/lib/docker`) - **MAIOR CONSUMIDOR**

Docker armazena tudo em `/var/lib/docker`, não no diretório do projeto:

```bash
# Ver espaço do Docker
sudo du -sh /var/lib/docker

# Ver detalhado
sudo du -sh /var/lib/docker/*
```

**O que Docker armazena:**
- **overlay2/**: Camadas de imagens e containers (pode ser 10-50GB+)
- **containers/**: Logs e dados de containers rodando
- **volumes/**: Volumes persistentes (PostgreSQL, etc.)
- **buildkit/**: Cache de builds
- **image/**: Metadados de imagens

**Solução:**
```bash
# Ver espaço usado pelo Docker
docker system df -v

# Limpar imagens não usadas
docker image prune -a

# Limpar tudo (CUIDADO: remove imagens, containers parados, volumes não usados)
docker system prune -a --volumes
```

### 2. **Logs do Sistema** (`/var/log`)

Sistema Linux gera muitos logs:
- `journald`: Logs do systemd (pode ser 1-5GB+)
- `/var/log/`: Logs de aplicações
- Logs rotativos que acumulam

**Solução:**
```bash
# Ver tamanho dos logs
sudo journalctl --disk-usage

# Limpar logs antigos
sudo journalctl --vacuum-time=7d  # Mantém apenas 7 dias
sudo journalctl --vacuum-size=500M  # Limita a 500MB

# Ver logs grandes
sudo find /var/log -type f -size +100M -exec ls -lh {} +
```

### 3. **Cache do Sistema** (`/var/cache`)

- `apt`: Cache de pacotes (pode ser 500MB-2GB)
- `nginx`: Cache do nginx
- Outros caches

**Solução:**
```bash
# Limpar cache do apt
sudo apt clean
sudo apt autoclean

# Ver cache do nginx
sudo du -sh /var/cache/nginx
```

### 4. **Outros Diretórios**

- `/opt`: Software instalado
- `/usr/local`: Binários locais
- `/tmp` e `/var/tmp`: Arquivos temporários
- Snap packages (`/var/lib/snapd`): Se usar snap

## 🔍 Como Descobrir Onde Estão os 77GB

### Método 1: Script Automático (Recomendado)

```bash
# No servidor, após fazer pull
chmod +x scripts/analyze-system-disk.sh
sudo ./scripts/analyze-system-disk.sh
```

### Método 2: Comandos Manuais

```bash
# 1. Ver top diretórios na raiz
sudo du -h --max-depth=1 / | sort -rh | head -20

# 2. Ver espaço do Docker
sudo du -sh /var/lib/docker
sudo du -sh /var/lib/docker/*

# 3. Ver logs do sistema
sudo journalctl --disk-usage
sudo du -sh /var/log

# 4. Ver cache
sudo du -sh /var/cache/*

# 5. Ver home directories
sudo du -sh /home/* /root
```

## 📊 Exemplo de Distribuição Típica (77GB)

```
/var/lib/docker/overlay2:      ~40-50GB  (Docker - imagens e containers)
/var/lib/docker/containers:     ~5-10GB   (Logs de containers)
/var/lib/docker/volumes:       ~1-2GB    (Volumes persistentes)
/var/log:                      ~2-5GB    (Logs do sistema)
/var/cache:                    ~1-2GB   (Cache)
/home e /root:                 ~2-3GB    (Projetos e dados)
/usr:                          ~5-10GB   (Sistema base)
/opt:                          ~1-2GB    (Software adicional)
Outros:                        ~5-10GB   (Sistema, swap, etc.)
────────────────────────────────────────
TOTAL:                         ~77GB
```

## 🎯 Plano de Ação para Liberar Espaço

### 1. Docker (Pode liberar 20-40GB)

```bash
# Ver o que está usando espaço
docker system df -v

# Limpar imagens não usadas
docker image prune -a

# Limpar containers parados
docker container prune

# Limpar volumes não usados (CUIDADO: pode remover dados)
docker volume prune

# Limpar tudo (mais agressivo)
docker system prune -a --volumes
```

### 2. Logs do Sistema (Pode liberar 2-5GB)

```bash
# Limpar logs do systemd (mantém 7 dias)
sudo journalctl --vacuum-time=7d

# OU limitar a 500MB
sudo journalctl --vacuum-size=500M

# Limpar logs antigos em /var/log
sudo find /var/log -type f -name "*.log.*" -mtime +30 -delete
sudo find /var/log -type f -name "*.gz" -mtime +30 -delete
```

### 3. Cache do Sistema (Pode liberar 1-2GB)

```bash
# Limpar cache do apt
sudo apt clean
sudo apt autoclean

# Limpar cache do nginx (se houver)
sudo rm -rf /var/cache/nginx/*
```

### 4. Projeto (Pode liberar ~500MB-1GB)

```bash
# Já sabemos: downloads, target, etc.
./scripts/cleanup-disk.sh
cd news-backend && cargo clean && cd ..
```

## 📝 Comandos Rápidos para Diagnóstico

```bash
# Ver top 20 diretórios no sistema TODO
sudo du -h --max-depth=1 / 2>/dev/null | sort -rh | head -20

# Ver apenas Docker
sudo du -sh /var/lib/docker/*

# Ver logs
sudo journalctl --disk-usage
sudo du -sh /var/log/*

# Ver cache
sudo du -sh /var/cache/*

# Ver espaço total
df -h /
```

## ✅ Resumo

**Por que 77GB?**
- Docker: ~40-50GB (overlay2, containers, volumes)
- Sistema: ~15-20GB (logs, cache, software)
- Projeto: ~2GB (o que você viu)
- Outros: ~5-10GB

**Solução:**
1. Execute `sudo ./scripts/analyze-system-disk.sh` para ver detalhes
2. Limpe Docker: `docker system prune -a`
3. Limpe logs: `sudo journalctl --vacuum-time=7d`
4. Limpe cache: `sudo apt clean`
5. Limpe projeto: `./scripts/cleanup-disk.sh`

Isso pode liberar **30-50GB** facilmente!

