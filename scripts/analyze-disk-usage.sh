#!/bin/bash

# Script para analisar uso de disco antes da limpeza

echo "📊 Análise de uso de disco do servidor"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Uso geral
echo "💾 Uso geral de disco:"
df -h /
echo ""

# Diretórios do projeto
echo "📁 Tamanho dos diretórios do projeto:"
if [ -d "./downloads" ]; then
    echo "  downloads: $(du -sh ./downloads 2>/dev/null | cut -f1 || echo '0')"
fi
if [ -d "./output" ]; then
    echo "  output: $(du -sh ./output 2>/dev/null | cut -f1 || echo '0')"
fi
if [ -d "./logs" ]; then
    echo "  logs: $(du -sh ./logs 2>/dev/null | cut -f1 || echo '0')"
fi
if [ -d "./images" ]; then
    echo "  images: $(du -sh ./images 2>/dev/null | cut -f1 || echo '0')"
fi
echo ""

# Docker
if command -v docker &> /dev/null; then
    echo "🐳 Uso do Docker:"
    echo "  Imagens: $(docker images -q | wc -l) imagens"
    echo "  Containers: $(docker ps -a -q | wc -l) containers"
    echo "  Volumes: $(docker volume ls -q | wc -l) volumes"
    echo "  Tamanho total Docker:"
    docker system df 2>/dev/null || true
    echo ""
fi

# Top 10 maiores arquivos no projeto
echo "📄 Top 10 maiores arquivos no projeto (exceto .git):"
find . -type f -not -path './.git/*' -exec du -h {} + 2>/dev/null | sort -rh | head -10 || true
echo ""

# Arquivos antigos que podem ser removidos
echo "🗓️  Arquivos antigos que podem ser removidos:"
echo "  Logs em logs/:"
find ./logs -type f -name "*.log" -mtime +7 2>/dev/null | wc -l | xargs echo "    "
echo "  (PDFs são limpos automaticamente pelo sistema de coleta)"

