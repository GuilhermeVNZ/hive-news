#!/bin/bash
# Script para analisar uso de espaço em TODO o sistema (não apenas o projeto)
# Uso: ./scripts/analyze-system-disk.sh

set -e

echo "🔍 Análise Completa de Espaço no Sistema"
echo "========================================"
echo ""

# 1. Espaço total do sistema
echo "📊 Espaço Total do Sistema:"
echo "----------------------------"
df -h / | tail -1 | awk '{print "Total: " $2 " | Usado: " $3 " (" $5 ") | Disponível: " $4}'
echo ""

# 2. Top 20 diretórios na raiz (/)
echo "📁 Top 20 Diretórios na Raiz (/):"
echo "----------------------------------"
sudo du -h --max-depth=1 / 2>/dev/null | sort -rh | head -20 | awk '{printf "%-10s %s\n", $1, $2}'
echo ""

# 3. Análise detalhada de diretórios comuns que consomem espaço

# Docker
echo "🐳 Docker (Principal Consumidor):"
echo "---------------------------------"
if [ -d "/var/lib/docker" ]; then
    echo "  /var/lib/docker: $(sudo du -sh /var/lib/docker 2>/dev/null | cut -f1)"
    echo "  overlay2: $(sudo du -sh /var/lib/docker/overlay2 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  containers: $(sudo du -sh /var/lib/docker/containers 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  volumes: $(sudo du -sh /var/lib/docker/volumes 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  buildkit: $(sudo du -sh /var/lib/docker/buildkit 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  image: $(sudo du -sh /var/lib/docker/image 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
fi
echo ""

# Logs do sistema
echo "📋 Logs do Sistema:"
echo "------------------"
if [ -d "/var/log" ]; then
    echo "  /var/log: $(sudo du -sh /var/log 2>/dev/null | cut -f1)"
    echo "  journald: $(sudo journalctl --disk-usage 2>/dev/null | awk '{print $7, $8}' || echo "N/A")"
    echo "  Top 10 maiores logs:"
    sudo find /var/log -type f -exec du -h {} + 2>/dev/null | sort -rh | head -10 | awk '{printf "    %-10s %s\n", $1, $2}'
fi
echo ""

# Cache do sistema
echo "💾 Cache do Sistema:"
echo "--------------------"
if [ -d "/var/cache" ]; then
    echo "  /var/cache: $(sudo du -sh /var/cache 2>/dev/null | cut -f1)"
    echo "  apt: $(sudo du -sh /var/cache/apt 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
    echo "  nginx: $(sudo du -sh /var/cache/nginx 2>/dev/null | cut -f1 2>/dev/null || echo "N/A")"
fi
echo ""

# Home directories
echo "👤 Diretórios Home:"
echo "-------------------"
if [ -d "/home" ]; then
    echo "  /home: $(sudo du -sh /home 2>/dev/null | cut -f1)"
    sudo du -sh /home/* 2>/dev/null | sort -rh | head -10 | awk '{printf "    %-10s %s\n", $1, $2}'
fi
if [ -d "/root" ]; then
    echo "  /root: $(sudo du -sh /root 2>/dev/null | cut -f1)"
fi
echo ""

# Usuários e processos
echo "🔍 Outros Diretórios Comuns:"
echo "----------------------------"
for dir in /opt /usr/local /tmp /var/tmp; do
    if [ -d "$dir" ]; then
        echo "  $dir: $(sudo du -sh "$dir" 2>/dev/null | cut -f1)"
    fi
done
echo ""

# Docker detalhado
echo "🐳 Docker Detalhado (docker system df):"
echo "----------------------------------------"
docker system df 2>/dev/null || echo "  Docker não disponível ou precisa de sudo"
echo ""

# Verificar se há snap (pode consumir muito espaço)
if command -v snap &> /dev/null; then
    echo "📦 Snap Packages:"
    echo "----------------"
    sudo du -sh /var/lib/snapd 2>/dev/null | awk '{print "  /var/lib/snapd: " $1}'
    echo ""
fi

# Resumo
echo "📊 Resumo dos Maiores Consumidores:"
echo "-----------------------------------"
echo "  (Executando análise completa...)"
sudo du -h --max-depth=1 / 2>/dev/null | sort -rh | head -15 | awk '{printf "  %-10s %s\n", $1, $2}'
echo ""

echo "✅ Análise do sistema concluída!"
echo ""
echo "💡 Dica: Para ver apenas o projeto, use: ./scripts/analyze-disk-usage.sh"
echo ""

