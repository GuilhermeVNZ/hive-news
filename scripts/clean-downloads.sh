#!/bin/bash

# Script para limpar TODO o conteúdo da pasta downloads
# ATENÇÃO: Este script remove TODOS os arquivos e pastas dentro de downloads/

set -e

echo "🧹 Limpando pasta downloads..."
echo ""

DOWNLOADS_DIR="./downloads"

if [ ! -d "$DOWNLOADS_DIR" ]; then
    echo "❌ Diretório downloads/ não encontrado!"
    exit 1
fi

# Calcular tamanho antes
BEFORE=$(du -sh "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")
BEFORE_BYTES=$(du -sb "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")

# Contar arquivos e pastas
FILE_COUNT=$(find "$DOWNLOADS_DIR" -type f 2>/dev/null | wc -l || echo "0")
DIR_COUNT=$(find "$DOWNLOADS_DIR" -mindepth 1 -type d 2>/dev/null | wc -l || echo "0")

echo "📊 Antes da limpeza:"
echo "   Tamanho: $BEFORE"
echo "   Arquivos: $FILE_COUNT"
echo "   Pastas: $DIR_COUNT"
echo ""

# Confirmar ação
read -p "⚠️  Tem certeza que deseja apagar TODO o conteúdo de downloads/? (digite 'SIM' para confirmar): " CONFIRM

if [ "$CONFIRM" != "SIM" ]; then
    echo "❌ Operação cancelada."
    exit 1
fi

echo ""
echo "🗑️  Removendo conteúdo..."

# Remover TODO o conteúdo (mas manter a pasta downloads/)
find "$DOWNLOADS_DIR" -mindepth 1 -delete 2>/dev/null || true

# Verificar resultado
AFTER=$(du -sh "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")
AFTER_BYTES=$(du -sb "$DOWNLOADS_DIR" 2>/dev/null | cut -f1 || echo "0")
FREED=$((BEFORE_BYTES - AFTER_BYTES))

echo ""
echo "✅ Limpeza concluída!"
echo ""
echo "📊 Depois da limpeza:"
echo "   Tamanho: $AFTER"
echo "   Espaço liberado: $(numfmt --to=iec-i --suffix=B $FREED 2>/dev/null || echo "${FREED} bytes")"
echo ""
echo "💾 Espaço em disco atual:"
df -h / | tail -1

