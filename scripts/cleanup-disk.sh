#!/bin/bash

# Script de limpeza de disco para o servidor
# Remove arquivos antigos e cache desnecessário

set -e

echo "🧹 Iniciando limpeza de disco..."
echo ""

# Função para calcular tamanho de diretório
get_size() {
    du -sh "$1" 2>/dev/null | cut -f1 || echo "0"
}

# Função para mostrar uso antes/depois
show_space() {
    echo "💾 Espaço em disco atual:"
    df -h / | tail -1
    echo ""
}

show_space

TOTAL_FREED=0

# 1. Limpar logs antigos (>7 dias)
echo "📋 1. Limpando logs antigos (>7 dias)..."
LOGS_DIR="./logs"
if [ -d "$LOGS_DIR" ]; then
    BEFORE=$(du -sb "$LOGS_DIR" 2>/dev/null | cut -f1 || echo "0")
    find "$LOGS_DIR" -type f -name "*.log" -mtime +7 -delete 2>/dev/null || true
    find "$LOGS_DIR" -type f -name "*.log.*" -mtime +7 -delete 2>/dev/null || true
    AFTER=$(du -sb "$LOGS_DIR" 2>/dev/null | cut -f1 || echo "0")
    FREED=$((BEFORE - AFTER))
    TOTAL_FREED=$((TOTAL_FREED + FREED))
    echo "   ✅ Logs: $(numfmt --to=iec-i --suffix=B $FREED 2>/dev/null || echo "${FREED}B") liberados"
else
    echo "   ⏭️  Diretório de logs não encontrado"
fi
echo ""

# 2. Limpar PDFs órfãos e processados da pasta downloads
echo "📥 2. Limpando PDFs órfãos e processados da pasta downloads..."
DOWNLOADS_DIR="./downloads"
REGISTRY_FILE="./articles_registry.json"

if [ -d "$DOWNLOADS_DIR" ]; then
    BEFORE=$(du -sb "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")
    
    if [ -f "$REGISTRY_FILE" ]; then
        # Ler IDs de artigos processados (Published ou Rejected) do registry
        PROCESSED_IDS=$(jq -r '.articles // {} | to_entries[] | select(.value.status == "Published" or .value.status == "Rejected") | .key' "$REGISTRY_FILE" 2>/dev/null || echo "")
        
        if [ -n "$PROCESSED_IDS" ]; then
            echo "$PROCESSED_IDS" | while read -r article_id; do
                # Remover PDFs deste artigo em qualquer subpasta de downloads/
                find "$DOWNLOADS_DIR" -type f -name "${article_id}.pdf" -delete 2>/dev/null || true
                find "$DOWNLOADS_DIR" -type f -name "${article_id}*.pdf" -delete 2>/dev/null || true
            done
            echo "   ✅ PDFs processados (Published/Rejected) removidos"
        fi
        
        # Remover PDFs órfãos (não referenciados no registry)
        # Obter todos os IDs do registry
        REGISTRY_IDS=$(jq -r '.articles // {} | keys[]' "$REGISTRY_FILE" 2>/dev/null || echo "")
        
        # Encontrar todos os PDFs em downloads/
        find "$DOWNLOADS_DIR" -type f -name "*.pdf" | while read -r pdf_path; do
            pdf_name=$(basename "$pdf_path" .pdf)
            # Extrair possível ID do artigo do nome do arquivo
            article_id=$(echo "$pdf_name" | grep -oE '[0-9]{4}\.[0-9]{4,5}(v[0-9]+)?' | head -1 || echo "$pdf_name")
            
            # Se o ID não está no registry, é um PDF órfão
            if [ -z "$REGISTRY_IDS" ] || ! echo "$REGISTRY_IDS" | grep -q "^${article_id}$"; then
                # Verificar se o PDF tem mais de 1 dia (evitar remover PDFs recém baixados)
                if [ -n "$(find "$pdf_path" -mtime +1 2>/dev/null)" ]; then
                    rm -f "$pdf_path" 2>/dev/null || true
                fi
            fi
        done
        echo "   ✅ PDFs órfãos (>1 dia) removidos"
    else
        # Se não há registry, remover PDFs antigos (>7 dias) como fallback
        echo "   ⚠️  Registry não encontrado, removendo PDFs antigos (>7 dias)"
        find "$DOWNLOADS_DIR" -type f -name "*.pdf" -mtime +7 -delete 2>/dev/null || true
    fi
    
    # Limpar diretórios vazios
    find "$DOWNLOADS_DIR" -type d -empty -delete 2>/dev/null || true
    
    AFTER=$(du -sb "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")
    FREED=$((BEFORE - AFTER))
    TOTAL_FREED=$((TOTAL_FREED + FREED))
    echo "   ✅ Downloads: $(numfmt --to=iec-i --suffix=B $FREED 2>/dev/null || echo "${FREED}B") liberados"
else
    echo "   ⏭️  Diretório de downloads não encontrado"
fi
echo ""

# 3. Limpar output de artigos antigos (>90 dias) - manter apenas publicados
echo "📝 3. Limpando artigos antigos não publicados (>90 dias)..."
OUTPUT_DIR="./output"
if [ -d "$OUTPUT_DIR" ]; then
    BEFORE=$(du -sb "$OUTPUT_DIR" 2>/dev/null | cut -f1 || echo "0")
    # Remove artigos não publicados há mais de 90 dias
    # (Mantém artigos que podem estar referenciados no registry)
    find "$OUTPUT_DIR" -type f -name "*.md" -mtime +90 -exec grep -l "draft\|unpublished" {} \; 2>/dev/null | \
        xargs rm -f 2>/dev/null || true
    find "$OUTPUT_DIR" -type d -empty -delete 2>/dev/null || true
    AFTER=$(du -sb "$OUTPUT_DIR" 2>/dev/null | cut -f1 || echo "0")
    FREED=$((BEFORE - AFTER))
    TOTAL_FREED=$((TOTAL_FREED + FREED))
    echo "   ✅ Output: $(numfmt --to=iec-i --suffix=B $FREED 2>/dev/null || echo "${FREED}B") liberados"
else
    echo "   ⏭️  Diretório de output não encontrado"
fi
echo ""

# 4. Limpar cache do Docker
echo "🐳 4. Limpando cache do Docker..."
if command -v docker &> /dev/null; then
    # Remover imagens não utilizadas
    BEFORE_IMAGES=$(docker images -q | wc -l)
    docker image prune -af --filter "until=168h" 2>/dev/null || true
    AFTER_IMAGES=$(docker images -q | wc -l)
    echo "   ✅ Imagens Docker removidas: $((BEFORE_IMAGES - AFTER_IMAGES))"
    
    # Remover containers parados
    docker container prune -f 2>/dev/null || true
    echo "   ✅ Containers parados removidos"
    
    # Remover volumes não utilizados (CUIDADO: pode remover dados importantes)
    # docker volume prune -f 2>/dev/null || true
    
    # Remover build cache antigo
    docker builder prune -af --filter "until=168h" 2>/dev/null || true
    echo "   ✅ Build cache limpo"
else
    echo "   ⏭️  Docker não encontrado"
fi
echo ""

# 5. Limpar cache do sistema (apt, npm, etc)
echo "📦 5. Limpando cache do sistema..."
if command -v apt-get &> /dev/null; then
    BEFORE_APT=$(du -sb /var/cache/apt 2>/dev/null | cut -f1 || echo "0")
    apt-get clean 2>/dev/null || true
    apt-get autoremove -y 2>/dev/null || true
    AFTER_APT=$(du -sb /var/cache/apt 2>/dev/null | cut -f1 || echo "0")
    FREED_APT=$((BEFORE_APT - AFTER_APT))
    TOTAL_FREED=$((TOTAL_FREED + FREED_APT))
    echo "   ✅ Cache apt: $(numfmt --to=iec-i --suffix=B $FREED_APT 2>/dev/null || echo "${FREED_APT}B") liberado"
fi

# Limpar cache npm (se existir)
if [ -d ~/.npm ]; then
    BEFORE_NPM=$(du -sb ~/.npm 2>/dev/null | cut -f1 || echo "0")
    npm cache clean --force 2>/dev/null || true
    AFTER_NPM=$(du -sb ~/.npm 2>/dev/null | cut -f1 || echo "0")
    FREED_NPM=$((BEFORE_NPM - AFTER_NPM))
    TOTAL_FREED=$((TOTAL_FREED + FREED_NPM))
    echo "   ✅ Cache npm: $(numfmt --to=iec-i --suffix=B $FREED_NPM 2>/dev/null || echo "${FREED_NPM}B") liberado"
fi
echo ""

# 6. Limpar arquivos temporários
echo "🗑️  6. Limpando arquivos temporários..."
TMP_DIRS=("/tmp" "/var/tmp")
for TMP_DIR in "${TMP_DIRS[@]}"; do
    if [ -d "$TMP_DIR" ]; then
        BEFORE_TMP=$(du -sb "$TMP_DIR" 2>/dev/null | cut -f1 || echo "0")
        find "$TMP_DIR" -type f -atime +7 -delete 2>/dev/null || true
        find "$TMP_DIR" -type d -empty -delete 2>/dev/null || true
        AFTER_TMP=$(du -sb "$TMP_DIR" 2>/dev/null | cut -f1 || echo "0")
        FREED_TMP=$((BEFORE_TMP - AFTER_TMP))
        TOTAL_FREED=$((TOTAL_FREED + FREED_TMP))
        echo "   ✅ $TMP_DIR: $(numfmt --to=iec-i --suffix=B $FREED_TMP 2>/dev/null || echo "${FREED_TMP}B") liberado"
    fi
done
echo ""

# 7. Limpar logs do sistema (>30 dias)
echo "📋 7. Limpando logs do sistema (>30 dias)..."
if [ -d "/var/log" ]; then
    # Limpar logs antigos (exceto logs críticos)
    find /var/log -type f -name "*.log" -mtime +30 ! -name "syslog" ! -name "auth.log" ! -name "kern.log" -delete 2>/dev/null || true
    find /var/log -type f -name "*.gz" -mtime +30 -delete 2>/dev/null || true
    journalctl --vacuum-time=30d 2>/dev/null || true
    echo "   ✅ Logs do sistema limpos"
fi
echo ""

# Mostrar resumo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Limpeza concluída!"
echo ""
echo "💾 Espaço total liberado: $(numfmt --to=iec-i --suffix=B $TOTAL_FREED 2>/dev/null || echo "${TOTAL_FREED} bytes")"
echo ""
show_space

# Mostrar maiores diretórios
echo "📊 Maiores diretórios no projeto:"
du -h --max-depth=1 . 2>/dev/null | sort -hr | head -10 || true

