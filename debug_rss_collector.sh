#!/bin/bash
# Script para diagnosticar problemas com o RSS collector do AIResearch
# Execute no servidor: bash debug_rss_collector.sh

cd ~/hive-news

echo "=== DIAGNÓSTICO: RSS Collector AIResearch ==="
echo ""

# 1. Verificar se o collector está no system_config.json
echo "1️⃣ Verificando system_config.json:"
if grep -q "rss_airesearch_news" news-backend/system_config.json; then
    echo "   ✅ Collector encontrado no system_config.json"
    echo "   📋 Detalhes do collector:"
    grep -A 12 '"id": "rss_airesearch_news"' news-backend/system_config.json | head -15
else
    echo "   ❌ Collector NÃO encontrado no system_config.json"
    exit 1
fi
echo ""

# 2. Verificar se o collector está no collectors_config.json
echo "2️⃣ Verificando collectors_config.json:"
if [ -f "data/collectors_config.json" ]; then
    if grep -q "rss_airesearch_news" data/collectors_config.json; then
        echo "   ✅ Collector encontrado no collectors_config.json"
        echo "   📋 Detalhes:"
        grep -A 8 '"id": "rss_airesearch_news"' data/collectors_config.json | head -10
    else
        echo "   ❌ Collector NÃO encontrado no collectors_config.json"
        echo "   ⚠️  Problema: Sync não está funcionando"
    fi
else
    echo "   ❌ Arquivo collectors_config.json não existe"
    echo "   ⚠️  Problema: Sync nunca foi executado ou falhou"
fi
echo ""

# 3. Verificar se o feed URL está acessível
echo "3️⃣ Testando feed URL:"
FEED_URL="https://www.airesearch.news/rss"
if curl -s -o /dev/null -w "%{http_code}" "$FEED_URL" | grep -q "200"; then
    echo "   ✅ Feed URL está acessível (HTTP 200)"
    echo "   📋 Primeiras linhas do feed:"
    curl -s "$FEED_URL" | head -20
else
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$FEED_URL")
    echo "   ❌ Feed URL retornou HTTP $HTTP_CODE"
    echo "   ⚠️  Problema: Feed não está acessível"
fi
echo ""

# 4. Verificar logs do backend
echo "4️⃣ Verificando logs do backend (últimas 100 linhas):"
echo "   Procurando por 'rss_airesearch_news' nos logs..."
docker compose logs --tail=100 backend 2>/dev/null | grep -i "rss_airesearch_news" | tail -10
if [ $? -ne 0 ]; then
    echo "   ⚠️  Nenhuma menção ao collector nos logs recentes"
fi
echo ""

# 5. Verificar se o collector está sendo processado
echo "5️⃣ Verificando se o collector está na lista de RSS collectors:"
docker compose logs --tail=200 backend 2>/dev/null | grep -E "DEBUG.*Adding RSS collector|RSS.*collector\(s\)|IDs:" | tail -5
echo ""

# 6. Verificar erros de coleta
echo "6️⃣ Verificando erros de coleta RSS:"
docker compose logs --tail=200 backend 2>/dev/null | grep -iE "rss.*error|rss.*failed|feed.*error" | tail -5
if [ $? -ne 0 ]; then
    echo "   ✅ Nenhum erro de RSS encontrado nos logs"
fi
echo ""

# 7. Verificar se há artigos sendo rejeitados como duplicados
echo "7️⃣ Verificando rejeições por duplicatas:"
docker compose logs --tail=200 backend 2>/dev/null | grep -iE "duplicate|rejected.*airesearch" | tail -5
echo ""

echo "=== FIM DO DIAGNÓSTICO ==="
echo ""
echo "📋 Próximos passos baseados nos resultados:"
echo "   - Se collector não está no collectors_config.json: problema de sync"
echo "   - Se feed URL não está acessível: problema de rede/URL"
echo "   - Se collector não aparece nos logs: problema de deserialização"
echo "   - Se há erros de coleta: verificar logs detalhados"
echo "   - Se artigos estão sendo rejeitados: problema de lógica de duplicatas"























