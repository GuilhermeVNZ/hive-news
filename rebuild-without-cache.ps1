# Script para rebuild completo dos containers sem usar cache
# Garante que todas as mudanças sejam aplicadas corretamente

Write-Host "🔄 Rebuild Docker Sem Cache" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
Write-Host ""

# 1. Parar todos os containers
Write-Host "1️⃣  Parando containers..." -ForegroundColor Yellow
docker-compose down
Write-Host "   ✅ Containers parados" -ForegroundColor Green
Write-Host ""

# 2. Remover imagens antigas (opcional - mais agressivo)
Write-Host "2️⃣  Removendo imagens antigas..." -ForegroundColor Yellow
docker-compose down --rmi local 2>$null
Write-Host "   ✅ Imagens locais removidas" -ForegroundColor Green
Write-Host ""

# 3. Rebuild sem cache
Write-Host "3️⃣  Rebuilding containers SEM CACHE..." -ForegroundColor Yellow
Write-Host "   Isso pode levar alguns minutos..." -ForegroundColor Gray

# Rebuild apenas frontends (mais rápido)
docker-compose build --no-cache airesearch
docker-compose build --no-cache scienceai
docker-compose build --no-cache dashboard

# OU rebuild tudo:
# docker-compose build --no-cache

Write-Host "   ✅ Rebuild completo" -ForegroundColor Green
Write-Host ""

# 4. Subir containers
Write-Host "4️⃣  Iniciando containers..." -ForegroundColor Yellow
docker-compose up -d
Write-Host "   ✅ Containers iniciados" -ForegroundColor Green
Write-Host ""

# 5. Verificar status
Write-Host "5️⃣  Verificando status dos containers..." -ForegroundColor Yellow
docker-compose ps
Write-Host ""

# 6. Mostrar logs
Write-Host "6️⃣  Mostrando logs (últimas 20 linhas)..." -ForegroundColor Yellow
Write-Host ""
Write-Host "--- AIResearch ---" -ForegroundColor Cyan
docker-compose logs --tail=20 airesearch
Write-Host ""
Write-Host "--- ScienceAI ---" -ForegroundColor Cyan
docker-compose logs --tail=20 scienceai
Write-Host ""
Write-Host "--- Dashboard ---" -ForegroundColor Cyan
docker-compose logs --tail=20 dashboard
Write-Host ""

Write-Host "✅ Rebuild completo!" -ForegroundColor Green
Write-Host ""
Write-Host "📝 Próximos passos:" -ForegroundColor Yellow
Write-Host "   1. Limpe o cache do navegador (Ctrl+Shift+R)" -ForegroundColor White
Write-Host "   2. Verifique se os arquivos JS/CSS têm novos hashes" -ForegroundColor White
Write-Host "   3. Verifique logs em tempo real: docker-compose logs -f" -ForegroundColor White
Write-Host ""

