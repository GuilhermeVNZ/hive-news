#!/bin/bash
# ====================================
# Health Check Script
# ====================================
# Verifica se o sistema está funcionando corretamente
# Pode ser usado em cron para monitoramento contínuo

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$BASE_DIR/logs"
LOG_FILE="$LOG_DIR/health-check.log"
TODAY=$(date +%Y-%m-%d)

# Criar diretório de logs
mkdir -p "$LOG_DIR"

# Função de logging
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Contadores
ERRORS=0
WARNINGS=0

# Início
log "========================================="
log "Health Check"
log "========================================="

# 1. Verificar processo do backend
log "1. Verificando processo do backend..."
if pgrep -f "news-backend" > /dev/null; then
    PID=$(pgrep -f "news-backend")
    UPTIME=$(ps -p $PID -o etime= | tr -d ' ')
    log "   ✓ Backend está rodando (PID: $PID, Uptime: $UPTIME)"
else
    log "   ✗ Backend NÃO está rodando!"
    ((ERRORS++))
fi

# 2. Verificar API endpoint
log "2. Verificando API endpoint..."
if command -v curl &> /dev/null; then
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/health 2>/dev/null || echo "000")
    if [ "$HTTP_CODE" = "200" ]; then
        log "   ✓ API respondendo corretamente (HTTP $HTTP_CODE)"
    else
        log "   ✗ API não está respondendo! (HTTP $HTTP_CODE)"
        ((ERRORS++))
    fi
else
    log "   ⚠️  curl não instalado, pulando verificação de API"
    ((WARNINGS++))
fi

# 3. Verificar última coleta
log "3. Verificando última coleta..."
LAST_ARTICLES_SCIENCEAI=$(find "$BASE_DIR/output/ScienceAI" -maxdepth 1 -type d -name "${TODAY}*" 2>/dev/null | wc -l)
LAST_ARTICLES_AIRESEARCH=$(find "$BASE_DIR/output/AIResearch" -maxdepth 1 -type d -name "${TODAY}*" 2>/dev/null | wc -l)

if [ "$LAST_ARTICLES_SCIENCEAI" -gt 0 ] || [ "$LAST_ARTICLES_AIRESEARCH" -gt 0 ]; then
    log "   ✓ Artigos coletados hoje: ScienceAI=$LAST_ARTICLES_SCIENCEAI, AIResearch=$LAST_ARTICLES_AIRESEARCH"
else
    log "   ⚠️  Nenhum artigo coletado hoje ainda"
    ((WARNINGS++))
fi

# 4. Verificar espaço em disco
log "4. Verificando espaço em disco..."
DISK_USAGE=$(df -h "$BASE_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -lt 70 ]; then
    log "   ✓ Espaço em disco OK: ${DISK_USAGE}% usado"
elif [ "$DISK_USAGE" -lt 85 ]; then
    log "   ⚠️  Espaço em disco alto: ${DISK_USAGE}% usado"
    ((WARNINGS++))
else
    log "   ✗ Espaço em disco CRÍTICO: ${DISK_USAGE}% usado!"
    ((ERRORS++))
fi

# 5. Verificar arquivo .env
log "5. Verificando configuração..."
if [ -f "$BASE_DIR/news-backend/.env" ]; then
    if grep -q "JWT_SECRET=" "$BASE_DIR/news-backend/.env" && \
       grep -q "DEFAULT_ADMIN_PASSWORD=" "$BASE_DIR/news-backend/.env"; then
        log "   ✓ Arquivo .env configurado corretamente"
    else
        log "   ✗ Arquivo .env está incompleto!"
        ((ERRORS++))
    fi
else
    log "   ✗ Arquivo .env não encontrado!"
    ((ERRORS++))
fi

# 6. Verificar registry
log "6. Verificando registry..."
REGISTRY_FILE="$BASE_DIR/articles_registry.json"
if [ -f "$REGISTRY_FILE" ]; then
    REGISTRY_SIZE=$(du -h "$REGISTRY_FILE" | awk '{print $1}')
    REGISTRY_ENTRIES=$(grep -c '"article_id"' "$REGISTRY_FILE" 2>/dev/null || echo "0")
    log "   ✓ Registry OK: $REGISTRY_ENTRIES artigos ($REGISTRY_SIZE)"
else
    log "   ⚠️  Registry não encontrado (será criado na primeira coleta)"
    ((WARNINGS++))
fi

# 7. Verificar logs recentes
log "7. Verificando logs recentes..."
RECENT_LOGS=$(find "$LOG_DIR" -type f -name "auto-collect-*.log" -mtime -1 2>/dev/null | wc -l)
if [ "$RECENT_LOGS" -gt 0 ]; then
    LAST_LOG=$(ls -1t "$LOG_DIR"/auto-collect-*.log 2>/dev/null | head -1)
    if grep -q "concluída com sucesso" "$LAST_LOG" 2>/dev/null; then
        log "   ✓ Última coleta bem-sucedida"
    else
        log "   ⚠️  Última coleta pode ter falhado"
        ((WARNINGS++))
    fi
else
    log "   ⚠️  Nenhum log de coleta recente (últimas 24h)"
    ((WARNINGS++))
fi

# 8. Verificar uso de memória
log "8. Verificando uso de memória..."
if [ -f "/proc/meminfo" ]; then
    MEM_TOTAL=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    MEM_AVAILABLE=$(grep MemAvailable /proc/meminfo | awk '{print $2}')
    MEM_USED_PERCENT=$((100 - (MEM_AVAILABLE * 100 / MEM_TOTAL)))
    
    if [ "$MEM_USED_PERCENT" -lt 80 ]; then
        log "   ✓ Memória OK: ${MEM_USED_PERCENT}% usado"
    elif [ "$MEM_USED_PERCENT" -lt 90 ]; then
        log "   ⚠️  Memória alta: ${MEM_USED_PERCENT}% usado"
        ((WARNINGS++))
    else
        log "   ✗ Memória CRÍTICA: ${MEM_USED_PERCENT}% usado!"
        ((ERRORS++))
    fi
else
    log "   ⚠️  Não foi possível verificar memória"
    ((WARNINGS++))
fi

# Resumo
log "========================================="
log "Resumo do Health Check"
log "========================================="
if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    log "Status: ✓ SAUDÁVEL"
    log "Todos os sistemas funcionando normalmente"
elif [ "$ERRORS" -eq 0 ]; then
    log "Status: ⚠️  ATENÇÃO"
    log "Sistema operacional com $WARNINGS aviso(s)"
else
    log "Status: ✗ CRÍTICO"
    log "Sistema com $ERRORS erro(s) e $WARNINGS aviso(s)"
    log "Ação necessária!"
fi
log "========================================="

# Limpar logs de health check antigos (> 7 dias)
find "$LOG_DIR" -name "health-check.log.*" -mtime +7 -delete 2>/dev/null

# Exit code: 0=ok, 1=warnings, 2=errors
if [ "$ERRORS" -gt 0 ]; then
    exit 2
elif [ "$WARNINGS" -gt 0 ]; then
    exit 1
else
    exit 0
fi








































