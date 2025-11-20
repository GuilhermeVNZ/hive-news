#!/bin/bash
# Script para analisar uso de espaço em disco no servidor
# Uso: ./scripts/analyze-disk-usage.sh [diretório]

set -e

BASE_DIR="${1:-.}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Análise de Uso de Espaço em Disco"
echo "===================================="
echo ""

# 1. Espaço total do sistema
echo "📊 Espaço Total do Sistema:"
echo "----------------------------"
df -h / | tail -1 | awk '{print "Total: " $2 " | Usado: " $3 " (" $5 ") | Disponível: " $4}'
echo ""

# 2. Top 20 diretórios no projeto
echo "📁 Top 20 Diretórios no Projeto (${PROJECT_ROOT}):"
echo "---------------------------------------------------"
du -h --max-depth=1 "$PROJECT_ROOT" 2>/dev/null | sort -rh | head -20 | awk '{printf "%-10s %s\n", $1, $2}'
echo ""

# 3. Análise detalhada de diretórios específicos
echo "📂 Análise Detalhada por Diretório:"
echo "------------------------------------"

# Downloads
if [ -d "$PROJECT_ROOT/downloads" ]; then
    echo ""
    echo "📥 Downloads:"
    echo "  Total: $(du -sh "$PROJECT_ROOT/downloads" 2>/dev/null | cut -f1)"
    echo "  PDFs: $(find "$PROJECT_ROOT/downloads" -name "*.pdf" -type f 2>/dev/null | wc -l) arquivos"
    echo "  Tamanho PDFs: $(find "$PROJECT_ROOT/downloads" -name "*.pdf" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1)"
    echo "  Cache: $(du -sh "$PROJECT_ROOT/downloads/cache" 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  Raw: $(du -sh "$PROJECT_ROOT/downloads/raw" 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  Temp: $(du -sh "$PROJECT_ROOT/downloads/temp" 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
fi

# Output
if [ -d "$PROJECT_ROOT/output" ]; then
    echo ""
    echo "📤 Output (Artigos Gerados):"
    echo "  Total: $(du -sh "$PROJECT_ROOT/output" 2>/dev/null | cut -f1)"
    echo "  Sites: $(find "$PROJECT_ROOT/output" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l) sites"
    echo "  Artigos: $(find "$PROJECT_ROOT/output" -name "article.md" -type f 2>/dev/null | wc -l) artigos"
    # Tamanho por site
    echo "  Por site:"
    for site_dir in "$PROJECT_ROOT/output"/*; do
        if [ -d "$site_dir" ]; then
            site_name=$(basename "$site_dir")
            site_size=$(du -sh "$site_dir" 2>/dev/null | cut -f1)
            article_count=$(find "$site_dir" -name "article.md" -type f 2>/dev/null | wc -l)
            echo "    - $site_name: $site_size ($article_count artigos)"
        fi
    done
fi

# Images
if [ -d "$PROJECT_ROOT/images" ]; then
    echo ""
    echo "🖼️  Images:"
    echo "  Total: $(du -sh "$PROJECT_ROOT/images" 2>/dev/null | cut -f1)"
    echo "  JPG: $(find "$PROJECT_ROOT/images" -name "*.jpg" -o -name "*.jpeg" -type f 2>/dev/null | wc -l) arquivos"
    echo "  PNG: $(find "$PROJECT_ROOT/images" -name "*.png" -type f 2>/dev/null | wc -l) arquivos"
    echo "  WebP: $(find "$PROJECT_ROOT/images" -name "*.webp" -type f 2>/dev/null | wc -l) arquivos"
    echo "  Tamanho JPG: $(find "$PROJECT_ROOT/images" \( -name "*.jpg" -o -name "*.jpeg" \) -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
    echo "  Tamanho PNG: $(find "$PROJECT_ROOT/images" -name "*.png" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
    echo "  Tamanho WebP: $(find "$PROJECT_ROOT/images" -name "*.webp" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
fi

# Logs
if [ -d "$PROJECT_ROOT/logs" ]; then
    echo ""
    echo "📋 Logs:"
    echo "  Total: $(du -sh "$PROJECT_ROOT/logs" 2>/dev/null | cut -f1)"
    echo "  Arquivos: $(find "$PROJECT_ROOT/logs" -type f 2>/dev/null | wc -l) arquivos"
    echo "  Mais antigo: $(find "$PROJECT_ROOT/logs" -type f -printf '%T+ %p\n' 2>/dev/null | sort | head -1 | cut -d' ' -f2- || echo "N/A")"
    echo "  Mais recente: $(find "$PROJECT_ROOT/logs" -type f -printf '%T+ %p\n' 2>/dev/null | sort | tail -1 | cut -d' ' -f2- || echo "N/A")"
    echo "  Top 10 maiores:"
    find "$PROJECT_ROOT/logs" -type f -exec du -h {} + 2>/dev/null | sort -rh | head -10 | awk '{printf "    %-10s %s\n", $1, $2}'
fi

# Docker
echo ""
echo "🐳 Docker:"
echo "  Imagens: $(docker images --format "{{.Repository}}:{{.Tag}}" 2>/dev/null | wc -l) imagens"
echo "  Tamanho imagens: $(docker images --format "{{.Size}}" 2>/dev/null | awk '{sum+=$1} END {print sum "GB"}' 2>/dev/null || echo "N/A")"
echo "  Containers: $(docker ps -a --format "{{.Names}}" 2>/dev/null | wc -l) containers"
echo "  Volumes: $(docker volume ls --format "{{.Name}}" 2>/dev/null | wc -l) volumes"
echo "  Tamanho volumes:"
docker volume ls --format "{{.Name}}" 2>/dev/null | while read vol; do
    size=$(docker system df -v 2>/dev/null | grep "$vol" | awk '{print $3}' || echo "N/A")
    echo "    - $vol: $size"
done

# Target (Rust build artifacts)
if [ -d "$PROJECT_ROOT/target" ] || [ -d "$PROJECT_ROOT/news-backend/target" ]; then
    echo ""
    echo "🦀 Rust Build Artifacts (target/):"
    if [ -d "$PROJECT_ROOT/target" ]; then
        echo "  Workspace target: $(du -sh "$PROJECT_ROOT/target" 2>/dev/null | cut -f1)"
    fi
    if [ -d "$PROJECT_ROOT/news-backend/target" ]; then
        echo "  Backend target: $(du -sh "$PROJECT_ROOT/news-backend/target" 2>/dev/null | cut -f1)"
        echo "  Release binaries: $(du -sh "$PROJECT_ROOT/news-backend/target/release" 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
        echo "  Debug binaries: $(du -sh "$PROJECT_ROOT/news-backend/target/debug" 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    fi
fi

# Node modules
echo ""
echo "📦 Node Modules:"
find "$PROJECT_ROOT" -name "node_modules" -type d 2>/dev/null | while read dir; do
    size=$(du -sh "$dir" 2>/dev/null | cut -f1)
    rel_path=$(realpath --relative-to="$PROJECT_ROOT" "$dir" 2>/dev/null || echo "$dir")
    echo "  - $rel_path: $size"
done

# 4. Top 20 arquivos maiores
echo ""
echo "📄 Top 20 Arquivos Mais Grandes:"
echo "--------------------------------"
find "$PROJECT_ROOT" -type f -exec du -h {} + 2>/dev/null | sort -rh | head -20 | awk '{printf "%-10s %s\n", $1, $2}'
echo ""

# 5. Arquivos por tipo
echo "📊 Tamanho por Tipo de Arquivo:"
echo "-------------------------------"
echo "  PDFs: $(find "$PROJECT_ROOT" -name "*.pdf" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo "  Imagens (JPG/PNG): $(find "$PROJECT_ROOT" \( -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" \) -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo "  Imagens WebP: $(find "$PROJECT_ROOT" -name "*.webp" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo "  Logs (.log): $(find "$PROJECT_ROOT" -name "*.log" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo "  JSON: $(find "$PROJECT_ROOT" -name "*.json" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo "  Markdown: $(find "$PROJECT_ROOT" -name "*.md" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")"
echo ""

# 6. Resumo de limpeza sugerida
echo "💡 Sugestões de Limpeza:"
echo "------------------------"
TOTAL_SIZE=$(du -sh "$PROJECT_ROOT" 2>/dev/null | cut -f1)

# Verificar logs antigos (>30 dias)
OLD_LOGS=$(find "$PROJECT_ROOT/logs" -type f -mtime +30 2>/dev/null | wc -l)
if [ "$OLD_LOGS" -gt 0 ]; then
    OLD_LOGS_SIZE=$(find "$PROJECT_ROOT/logs" -type f -mtime +30 -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")
    echo "  ⚠️  Logs antigos (>30 dias): $OLD_LOGS arquivos ($OLD_LOGS_SIZE)"
    echo "     Comando: find logs -type f -mtime +30 -delete"
fi

# Verificar PDFs duplicados ou não usados
PDF_COUNT=$(find "$PROJECT_ROOT/downloads" -name "*.pdf" -type f 2>/dev/null | wc -l)
if [ "$PDF_COUNT" -gt 0 ]; then
    PDF_SIZE=$(find "$PROJECT_ROOT/downloads" -name "*.pdf" -type f -exec du -ch {} + 2>/dev/null | tail -1 | cut -f1 2>/dev/null || echo "0")
    echo "  ⚠️  PDFs em downloads: $PDF_COUNT arquivos ($PDF_SIZE)"
    echo "     Comando: ./scripts/cleanup-disk.sh"
fi

# Verificar target/ (build artifacts)
if [ -d "$PROJECT_ROOT/target" ] || [ -d "$PROJECT_ROOT/news-backend/target" ]; then
    TARGET_SIZE=""
    if [ -d "$PROJECT_ROOT/target" ]; then
        TARGET_SIZE=$(du -sh "$PROJECT_ROOT/target" 2>/dev/null | cut -f1)
    elif [ -d "$PROJECT_ROOT/news-backend/target" ]; then
        TARGET_SIZE=$(du -sh "$PROJECT_ROOT/news-backend/target" 2>/dev/null | cut -f1)
    fi
    if [ -n "$TARGET_SIZE" ]; then
        echo "  ⚠️  Build artifacts (target/): $TARGET_SIZE"
        echo "     Comando: cargo clean (no diretório do projeto Rust)"
    fi
fi

# Verificar node_modules
NODE_MODULES_COUNT=$(find "$PROJECT_ROOT" -name "node_modules" -type d 2>/dev/null | wc -l)
if [ "$NODE_MODULES_COUNT" -gt 0 ]; then
    echo "  ⚠️  node_modules: $NODE_MODULES_COUNT diretórios"
    echo "     Comando: find . -name node_modules -type d -exec rm -rf {} + (CUIDADO!)"
fi

echo ""
echo "✅ Análise concluída!"
echo ""
