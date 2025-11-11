#!/bin/bash
# ====================================
# Script de Coleta Automática
# ====================================
# Executa o pipeline completo de coleta e geração de artigos
# Para uso em cron jobs ou agendamento manual

set -e

# Configuração (ajustar para seu servidor)
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
cd "$BACKEND_DIR" || {
    log "ERROR: Não foi possível acessar $BACKEND_DIR"
    exit 1
}

# Verificar se o binário existe
if [ ! -f "target/release/news-backend" ]; then
    log "ERROR: Binário não encontrado em target/release/news-backend"
    log "Execute 'cargo build --release' primeiro"
    exit 1
fi

# Verificar .env
if [ ! -f ".env" ]; then
    log "ERROR: Arquivo .env não encontrado"
    exit 1
fi

# Executar pipeline
log "Executando pipeline completo..."
START_TIME=$(date +%s)

if ./target/release/news-backend pipeline >> "$LOG_FILE" 2>&1; then
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    log "✓ Pipeline executado com sucesso (${DURATION}s)"
    
    # Contar novos artigos de hoje
    TODAY=$(date +%Y-%m-%d)
    ARTICLES_SCIENCEAI=$(find "$BASE_DIR/output/ScienceAI" -maxdepth 1 -type d -name "${TODAY}*" 2>/dev/null | wc -l)
    ARTICLES_AIRESEARCH=$(find "$BASE_DIR/output/AIResearch" -maxdepth 1 -type d -name "${TODAY}*" 2>/dev/null | wc -l)
    
    log "Artigos ScienceAI hoje: $ARTICLES_SCIENCEAI"
    log "Artigos AIResearch hoje: $ARTICLES_AIRESEARCH"
    log "Total hoje: $((ARTICLES_SCIENCEAI + ARTICLES_AIRESEARCH))"
else
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    log "ERROR: Pipeline falhou após ${DURATION}s"
    log "Verifique o log para detalhes: $LOG_FILE"
    exit 1
fi

# Limpeza de arquivos temporários antigos (> 7 dias)
log "Limpando arquivos temporários..."
TEMP_DELETED=$(find "$BASE_DIR/downloads/temp" -type f -mtime +7 -delete -print 2>/dev/null | wc -l)
if [ "$TEMP_DELETED" -gt 0 ]; then
    log "✓ $TEMP_DELETED arquivo(s) temporário(s) removido(s)"
else
    log "✓ Nenhum arquivo temporário antigo para remover"
fi

# Backup do registry (se mudou nas últimas 24h)
REGISTRY_FILE="$BASE_DIR/articles_registry.json"
if [ -f "$REGISTRY_FILE" ]; then
    if [ $(find "$REGISTRY_FILE" -mtime -1 2>/dev/null | wc -l) -gt 0 ]; then
        BACKUP_DIR="$BASE_DIR/backups"
        mkdir -p "$BACKUP_DIR"
        cp "$REGISTRY_FILE" "$BACKUP_DIR/registry-$TIMESTAMP.json"
        log "✓ Backup do registry criado: registry-$TIMESTAMP.json"
        
        # Manter apenas últimos 30 backups
        BACKUP_COUNT=$(ls -1 "$BACKUP_DIR"/registry-*.json 2>/dev/null | wc -l)
        if [ "$BACKUP_COUNT" -gt 30 ]; then
            ls -1t "$BACKUP_DIR"/registry-*.json | tail -n +31 | xargs rm -f
            log "✓ Backups antigos removidos (mantendo últimos 30)"
        fi
    fi
fi

# Estatísticas do disco
DISK_USAGE=$(df -h "$BASE_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
OUTPUT_SIZE=$(du -sh "$BASE_DIR/output" 2>/dev/null | awk '{print $1}')
log "Uso do disco: ${DISK_USAGE}%"
log "Tamanho do output: ${OUTPUT_SIZE}"

# Alerta se disco > 80%
if [ "$DISK_USAGE" -gt 80 ]; then
    log "⚠️  ALERTA: Uso de disco acima de 80%!"
fi

# Remover logs antigos (> 30 dias)
OLD_LOGS=$(find "$LOG_DIR" -type f -name "auto-collect-*.log" -mtime +30 -delete -print 2>/dev/null | wc -l)
if [ "$OLD_LOGS" -gt 0 ]; then
    log "✓ $OLD_LOGS log(s) antigo(s) removido(s)"
fi

# Fim
log "========================================="
log "Coleta automática concluída com sucesso"
log "========================================="

exit 0


