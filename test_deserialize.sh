#!/bin/bash
# Script para testar deserialização do system_config.json no servidor
# Execute no servidor: bash test_deserialize.sh

cd ~/hive-news

echo "=== Testando deserialização do system_config.json ==="
echo ""

# Verificar se o arquivo existe
if [ ! -f "news-backend/system_config.json" ]; then
    echo "❌ Arquivo não encontrado!"
    exit 1
fi

# Contar collectors no JSON usando jq
echo "📊 Verificando JSON com jq:"
COLLECTOR_COUNT=$(cat news-backend/system_config.json | jq '.sites.airesearch.collectors | length')
echo "   Total de collectors no JSON: $COLLECTOR_COUNT"
echo ""

# Listar todos os collectors
echo "📋 Listando collectors:"
cat news-backend/system_config.json | jq '.sites.airesearch.collectors[] | {id, enabled, collector_type, feed_url}' | head -40
echo ""

# Verificar se o rss_airesearch_news está presente
echo "🔍 Verificando se rss_airesearch_news está presente:"
if cat news-backend/system_config.json | jq -e '.sites.airesearch.collectors[] | select(.id == "rss_airesearch_news")' > /dev/null; then
    echo "   ✅ rss_airesearch_news encontrado no JSON"
    cat news-backend/system_config.json | jq '.sites.airesearch.collectors[] | select(.id == "rss_airesearch_news")'
else
    echo "   ❌ rss_airesearch_news NÃO encontrado no JSON"
fi
echo ""

# Verificar se há problemas de sintaxe JSON
echo "🔍 Verificando sintaxe JSON:"
if cat news-backend/system_config.json | jq . > /dev/null 2>&1; then
    echo "   ✅ JSON válido"
else
    echo "   ❌ JSON inválido!"
    cat news-backend/system_config.json | jq . 2>&1 | head -20
fi
echo ""

# Verificar se o campo destinations está presente no 4º collector
echo "🔍 Verificando campo 'destinations' no rss_airesearch_news:"
cat news-backend/system_config.json | jq '.sites.airesearch.collectors[] | select(.id == "rss_airesearch_news") | .destinations'
echo ""

echo "=== Fim do teste ==="




