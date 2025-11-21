#!/bin/bash

# 🧹 Script de Limpeza Completa de Cache do Servidor
# Uso: ./scripts/clear-all-cache.sh

echo "🧹 Iniciando limpeza completa de cache..."

# 1. Parar containers
echo "📦 Parando containers..."
docker compose down

# 2. Limpeza completa do Docker
echo "🐳 Limpando cache do Docker..."
docker system prune -af
docker builder prune -af
docker image prune -af
docker container prune -f
docker network prune -f

# 3. Mostrar espaço liberado
echo "📊 Espaço em disco após limpeza:"
df -h

# 4. Rebuild sem cache
echo "🔨 Fazendo rebuild sem cache..."
docker compose build --no-cache

# 5. Subir containers
echo "🚀 Subindo containers..."
docker compose up -d

# 6. Verificar status
echo "✅ Status dos containers:"
docker compose ps

echo "🎉 Limpeza completa finalizada!"
echo "💡 Dica: Force refresh no navegador (Ctrl+F5)"
