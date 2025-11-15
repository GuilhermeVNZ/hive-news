#!/bin/bash
# Script para corrigir problemas com o RSS collector do AIResearch
# Execute no servidor: bash fix_rss_collector.sh

cd ~/hive-news

echo "=== CORREÇÃO: RSS Collector AIResearch ==="
echo ""

# 1. Verificar/criar diretório data
echo "1️⃣ Verificando diretório /data:"
if [ ! -d "data" ]; then
    echo "   Criando diretório data..."
    mkdir -p data
    chmod 755 data
fi
echo "   ✅ Diretório data existe"
echo ""

# 2. Verificar permissões
echo "2️⃣ Verificando permissões:"
ls -la data/ 2>/dev/null | head -5
echo ""

# 3. Testar feed URL com mais detalhes
echo "3️⃣ Testando feed URL com mais detalhes:"
FEED_URL="https://www.airesearch.news/rss"
echo "   Testando: $FEED_URL"

# Teste 1: DNS
echo "   - Verificando DNS..."
if nslookup www.airesearch.news > /dev/null 2>&1; then
    echo "     ✅ DNS resolve corretamente"
else
    echo "     ❌ DNS não resolve"
    echo "     ⚠️  Problema: DNS não está funcionando no servidor"
fi

# Teste 2: Conectividade
echo "   - Testando conectividade..."
if timeout 5 curl -s -o /dev/null -w "%{http_code}" "$FEED_URL" > /tmp/feed_test.txt 2>&1; then
    HTTP_CODE=$(cat /tmp/feed_test.txt)
    if [ "$HTTP_CODE" = "200" ]; then
        echo "     ✅ Feed URL está acessível (HTTP 200)"
        echo "     📋 Primeiras linhas do feed:"
        curl -s "$FEED_URL" | head -10
    else
        echo "     ⚠️  Feed URL retornou HTTP $HTTP_CODE"
    fi
else
    echo "     ❌ Erro ao conectar (timeout ou conexão recusada)"
    echo "     ⚠️  Problema: Servidor pode não ter acesso à internet ou firewall bloqueando"
    echo "     📋 Tentando com wget..."
    if wget -q --spider --timeout=5 "$FEED_URL" 2>&1; then
        echo "     ✅ wget conseguiu acessar"
    else
        echo "     ❌ wget também falhou"
    fi
fi
echo ""

# 4. Criar collectors_config.json manualmente se não existir
echo "4️⃣ Verificando/criando collectors_config.json:"
if [ ! -f "data/collectors_config.json" ]; then
    echo "   ⚠️  Arquivo não existe, criando estrutura básica..."
    cat > data/collectors_config.json << 'EOF'
{
  "collectors": [],
  "updated_at": "2025-11-15T00:00:00Z"
}
EOF
    echo "   ✅ Arquivo criado (vazio, será preenchido pelo sync)"
else
    echo "   ✅ Arquivo já existe"
fi
echo ""

# 5. Verificar se o backend consegue escrever no diretório
echo "5️⃣ Testando escrita no diretório data:"
TEST_FILE="data/.write_test"
if touch "$TEST_FILE" 2>/dev/null; then
    rm -f "$TEST_FILE"
    echo "   ✅ Permissões de escrita OK"
else
    echo "   ❌ Sem permissão de escrita"
    echo "   💡 Solução: sudo chown -R \$USER:\$USER data/"
fi
echo ""

# 6. Verificar logs do sync
echo "6️⃣ Verificando logs de sync mais recentes:"
docker compose logs --tail=50 backend 2>/dev/null | grep -E "SYNC|collectors_config|Failed to save" | tail -10
echo ""

# 7. Sugestões de correção
echo "=== SUGESTÕES DE CORREÇÃO ==="
echo ""
echo "Se o feed URL não está acessível:"
echo "  1. Verificar se o servidor tem acesso à internet"
echo "  2. Verificar firewall/proxy"
echo "  3. Testar manualmente: curl -v https://www.airesearch.news/rss"
echo ""
echo "Se o collectors_config.json não está sendo criado:"
echo "  1. Verificar permissões: ls -la data/"
echo "  2. Criar manualmente: mkdir -p data && touch data/collectors_config.json"
echo "  3. Verificar logs do backend: docker compose logs backend | grep SYNC"
echo ""
echo "Para forçar sync manualmente:"
echo "  docker compose exec backend news-backend --test-news-collector"
echo ""



