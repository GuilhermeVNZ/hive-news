# Configuração de Agendamento Automático

Este guia explica como configurar a coleta automática de notícias para alimentar os sites ScienceAI e AIResearch.

---

## 📋 Visão Geral

O sistema de coleta automática roda em horários programados e:
1. Coleta notícias de todas as fontes configuradas
2. Filtra e categoriza artigos
3. Gera conteúdo editorial automaticamente
4. Publica nos sites ScienceAI e AIResearch

---

## ⏰ Horários Recomendados

### Coleta Principal (Diária)
- **6h da manhã**: Coleta geral (robótica, IA, quantum)
- **12h (meio-dia)**: Atualização mid-day
- **18h (tarde)**: Coleta vespertina
- **0h (meia-noite)**: Limpeza e manutenção

### Coleta Prioritária (A cada 3h)
- Fontes principais: OpenAI, Google AI, Anthropic, Microsoft
- Horários: 3h, 9h, 15h, 21h

---

## 🔧 Método 1: Systemd Timers (Linux Moderno)

### 1.1. Criar Timer de Coleta Principal

**Arquivo:** `/etc/systemd/system/news-collector.service`

```ini
[Unit]
Description=News Collector Service
After=network.target

[Service]
Type=oneshot
User=<seu-usuario>
WorkingDirectory=<caminho-completo>/news-backend
Environment="RUST_LOG=info"
ExecStart=<caminho-completo>/news-backend/target/release/news-backend pipeline
StandardOutput=append:/var/log/news-collector.log
StandardError=append:/var/log/news-collector.error.log

[Install]
WantedBy=multi-user.target
```

**Arquivo:** `/etc/systemd/system/news-collector.timer`

```ini
[Unit]
Description=News Collector Timer
Requires=news-collector.service

[Timer]
OnCalendar=*-*-* 06,12,18:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

### 1.2. Ativar Timer

```bash
# Substituir <seu-usuario> e <caminho-completo> nos arquivos acima

# Recarregar systemd
sudo systemctl daemon-reload

# Habilitar e iniciar timer
sudo systemctl enable news-collector.timer
sudo systemctl start news-collector.timer

# Verificar status
sudo systemctl status news-collector.timer
sudo systemctl list-timers | grep news
```

### 1.3. Criar Timer de Limpeza

**Arquivo:** `/etc/systemd/system/news-cleanup.service`

```ini
[Unit]
Description=News Cleanup Service
After=network.target

[Service]
Type=oneshot
User=<seu-usuario>
WorkingDirectory=<caminho-completo>/news-backend
ExecStart=<caminho-completo>/news-backend/target/release/news-backend clean-old-articles 30
StandardOutput=append:/var/log/news-cleanup.log
StandardError=append:/var/log/news-cleanup.error.log
```

**Arquivo:** `/etc/systemd/system/news-cleanup.timer`

```ini
[Unit]
Description=News Cleanup Timer (Daily at midnight)
Requires=news-cleanup.service

[Timer]
OnCalendar=*-*-* 00:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

Ativar:
```bash
sudo systemctl daemon-reload
sudo systemctl enable news-cleanup.timer
sudo systemctl start news-cleanup.timer
```

---

## 🔧 Método 2: Cron Jobs (Linux Tradicional/Hostinger)

### 2.1. Editar Crontab

```bash
crontab -e
```

### 2.2. Adicionar Jobs

```cron
# News Collection - 4x por dia (6h, 12h, 18h, 0h)
0 6,12,18,0 * * * cd /caminho/completo/news-backend && ./target/release/news-backend pipeline >> /caminho/logs/collector-$(date +\%Y\%m\%d).log 2>&1

# Priority Sources - A cada 3 horas
0 */3 * * * cd /caminho/completo/news-backend && ./target/release/news-backend test-news-collector >> /caminho/logs/priority-$(date +\%Y\%m\%d).log 2>&1

# Cleanup - Diariamente à meia-noite
0 0 * * * cd /caminho/completo/news-backend && ./target/release/news-backend clean-old-articles 30 >> /caminho/logs/cleanup.log 2>&1

# Backup Registry - Semanalmente (domingo 2h)
0 2 * * 0 cd /caminho/completo && cp articles_registry.json backups/registry-$(date +\%Y\%m\%d).json
```

### 2.3. Verificar Cron

```bash
# Listar jobs ativos
crontab -l

# Ver logs do cron
tail -f /var/log/cron
# ou
tail -f /caminho/logs/collector-*.log
```

---

## 🔧 Método 3: Scripts Customizados (Máximo Controle)

### 3.1. Criar Script de Coleta

**Arquivo:** `deploy/auto-collect.sh`

```bash
#!/bin/bash
# Script de coleta automática com logging avançado

set -e

# Configuração
BASE_DIR="/caminho/completo/news"
BACKEND_DIR="$BASE_DIR/news-backend"
LOG_DIR="$BASE_DIR/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/auto-collect-$TIMESTAMP.log"

# Criar diretório de logs
mkdir -p "$LOG_DIR"

# Função de logging
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Início
log "========================================="
log "Iniciando coleta automática"
log "========================================="

# CD para diretório do backend
cd "$BACKEND_DIR" || exit 1

# Verificar se o binário existe
if [ ! -f "target/release/news-backend" ]; then
    log "ERROR: Binário não encontrado. Execute 'cargo build --release' primeiro."
    exit 1
fi

# Executar pipeline
log "Executando pipeline completo..."
if ./target/release/news-backend pipeline >> "$LOG_FILE" 2>&1; then
    log "✓ Pipeline executado com sucesso"
    
    # Contar novos artigos
    ARTICLES_SCIENCEAI=$(find "$BASE_DIR/output/ScienceAI" -maxdepth 1 -type d -name "$(date +%Y-%m-%d)*" | wc -l)
    ARTICLES_AIRESEARCH=$(find "$BASE_DIR/output/AIResearch" -maxdepth 1 -type d -name "$(date +%Y-%m-%d)*" | wc -l)
    
    log "Novos artigos ScienceAI: $ARTICLES_SCIENCEAI"
    log "Novos artigos AIResearch: $ARTICLES_AIRESEARCH"
else
    log "ERROR: Pipeline falhou!"
    exit 1
fi

# Limpeza de arquivos antigos (> 7 dias)
log "Limpando arquivos temporários antigos..."
find "$BASE_DIR/downloads/temp" -type f -mtime +7 -delete 2>/dev/null || true
log "✓ Limpeza concluída"

# Backup do registry (se mudou nas últimas 24h)
REGISTRY_FILE="$BASE_DIR/articles_registry.json"
if [ -f "$REGISTRY_FILE" ] && [ $(find "$REGISTRY_FILE" -mtime -1 2>/dev/null | wc -l) -gt 0 ]; then
    BACKUP_DIR="$BASE_DIR/backups"
    mkdir -p "$BACKUP_DIR"
    cp "$REGISTRY_FILE" "$BACKUP_DIR/registry-$TIMESTAMP.json"
    log "✓ Backup do registry criado"
fi

# Remover logs antigos (> 30 dias)
find "$LOG_DIR" -type f -name "auto-collect-*.log" -mtime +30 -delete 2>/dev/null || true

log "========================================="
log "Coleta automática concluída"
log "========================================="
```

Tornar executável:
```bash
chmod +x deploy/auto-collect.sh
```

### 3.2. Adicionar ao Cron

```cron
# Coleta com script customizado - 4x por dia
0 6,12,18,0 * * * /caminho/completo/news/deploy/auto-collect.sh
```

---

## 📊 Monitoramento e Alertas

### 4.1. Script de Verificação de Saúde

**Arquivo:** `deploy/health-check.sh`

```bash
#!/bin/bash
# Verifica se o sistema está funcionando corretamente

BASE_DIR="/caminho/completo/news"
LOG_FILE="$BASE_DIR/logs/health-check.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Verificar processo do backend
if pgrep -f "news-backend" > /dev/null; then
    log "✓ Backend está rodando"
else
    log "✗ Backend NÃO está rodando!"
    # Opcional: enviar alerta por email
    # echo "Backend news parou!" | mail -s "ALERTA: Backend News" admin@example.com
fi

# Verificar última coleta
LAST_ARTICLE=$(find "$BASE_DIR/output/ScienceAI" -maxdepth 1 -type d -name "$(date +%Y-%m-%d)*" | head -1)
if [ -n "$LAST_ARTICLE" ]; then
    log "✓ Artigos coletados hoje"
else
    log "⚠ Nenhum artigo coletado hoje"
fi

# Verificar espaço em disco
DISK_USAGE=$(df -h "$BASE_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 80 ]; then
    log "⚠ Uso de disco alto: ${DISK_USAGE}%"
fi
```

### 4.2. Adicionar Health Check ao Cron

```cron
# Verificar saúde a cada hora
0 * * * * /caminho/completo/news/deploy/health-check.sh
```

---

## 🚀 Configuração Completa na Hostinger

### Passo 1: Acesso SSH

```bash
ssh usuario@seu-dominio.com
```

### Passo 2: Navegar para o diretório

```bash
cd ~/public_html/news  # ou caminho onde você instalou
```

### Passo 3: Tornar scripts executáveis

```bash
chmod +x deploy/*.sh
chmod +x news-backend/target/release/news-backend
```

### Passo 4: Configurar Cron via cPanel

1. Acesse cPanel
2. Vá em "Cron Jobs" ou "Tarefas Agendadas"
3. Adicione:

```
# Coleta principal - 4x/dia
0 6,12,18,0 * * * /home/usuario/public_html/news/deploy/auto-collect.sh

# Health check - a cada hora
0 * * * * /home/usuario/public_html/news/deploy/health-check.sh

# Limpeza - diariamente
0 1 * * * cd /home/usuario/public_html/news/news-backend && ./target/release/news-backend clean-old-articles 30
```

---

## 📝 Logs e Troubleshooting

### Ver logs em tempo real

```bash
# Coletor
tail -f logs/auto-collect-*.log

# Backend
tail -f logs/backend.log

# Cron (sistema)
tail -f /var/log/cron
```

### Testar manualmente

```bash
# Testar coleta
cd news-backend
./target/release/news-backend test-news-collector

# Testar pipeline completo
./target/release/news-backend pipeline

# Verificar health
curl http://localhost:3000/api/health
```

---

## 🔐 Segurança

### Proteger Logs

```bash
# Criar diretório de logs fora do public_html
mkdir -p ~/news-logs
chmod 700 ~/news-logs

# Atualizar scripts para usar este diretório
LOG_DIR="$HOME/news-logs"
```

### Rotação de Logs

Adicionar ao cron:
```cron
# Rotacionar logs semanalmente
0 3 * * 0 find ~/news-logs -name "*.log" -mtime +7 -exec gzip {} \; && find ~/news-logs -name "*.log.gz" -mtime +30 -delete
```

---

## ✅ Checklist Final

- [ ] Scripts criados e executáveis
- [ ] Cron jobs configurados
- [ ] Logs funcionando
- [ ] Health check ativo
- [ ] Testado manualmente
- [ ] Backup automático configurado
- [ ] Monitoramento ativo
- [ ] Alertas configurados (opcional)

---

## 📞 Suporte

Para problemas:
1. Verifique logs em `logs/`
2. Execute manualmente para debug
3. Verifique permissões dos arquivos
4. Confirme que .env está configurado

**Gerado automaticamente pela configuração de produção**
























