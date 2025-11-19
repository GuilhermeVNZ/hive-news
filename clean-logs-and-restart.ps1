# Script para limpar logs e reiniciar backend e frontends
# Uso: .\clean-logs-and-restart.ps1

Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Limpeza de Logs e Reinicialização" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Navegar para o diretório do projeto
Set-Location "G:\Hive-Hub\News-main"

# 1. Limpar logs do Docker (container logs)
Write-Host "📋 1. Limpando logs dos containers Docker..." -ForegroundColor Yellow
docker compose -f docker-compose.prod.yml logs --clear backend airesearch scienceai dashboard 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Logs dos containers limpos" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Alguns serviços podem não estar rodando" -ForegroundColor Yellow
}
Write-Host ""

# 2. Limpar logs da aplicação (pasta ./logs)
Write-Host "📋 2. Limpando logs da aplicação (pasta ./logs)..." -ForegroundColor Yellow
if (Test-Path ".\logs") {
    $logFiles = Get-ChildItem -Path ".\logs" -File -Recurse -ErrorAction SilentlyContinue
    if ($logFiles) {
        $logFiles | Remove-Item -Force -ErrorAction SilentlyContinue
        Write-Host "   ✅ $($logFiles.Count) arquivo(s) de log removido(s)" -ForegroundColor Green
    } else {
        Write-Host "   ℹ️  Pasta de logs está vazia" -ForegroundColor Gray
    }
} else {
    Write-Host "   ℹ️  Pasta de logs não existe" -ForegroundColor Gray
}
Write-Host ""

# 3. Limpar logs do Docker daemon (opcional - requer privilégios)
Write-Host "📋 3. Verificando uso de disco pelos logs do Docker..." -ForegroundColor Yellow
$dockerDiskUsage = docker system df 2>$null
if ($dockerDiskUsage) {
    Write-Host $dockerDiskUsage -ForegroundColor Gray
}
Write-Host ""

# 4. Parar serviços
Write-Host "🛑 4. Parando serviços..." -ForegroundColor Yellow
docker compose -f docker-compose.prod.yml stop backend airesearch scienceai dashboard 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Serviços parados" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Alguns serviços podem não estar rodando" -ForegroundColor Yellow
}
Write-Host ""

# 5. Aguardar 2 segundos
Write-Host "⏳ Aguardando 2 segundos..." -ForegroundColor Gray
Start-Sleep -Seconds 2
Write-Host ""

# 6. Iniciar serviços
Write-Host "🚀 5. Iniciando serviços..." -ForegroundColor Yellow
docker compose -f docker-compose.prod.yml up -d backend airesearch scienceai dashboard
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Serviços iniciados" -ForegroundColor Green
} else {
    Write-Host "   ❌ Erro ao iniciar serviços" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 7. Verificar status
Write-Host "📊 6. Verificando status dos serviços..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
docker compose -f docker-compose.prod.yml ps backend airesearch scienceai dashboard
Write-Host ""

# 8. Mostrar últimas linhas dos logs
Write-Host "📋 7. Últimas linhas dos logs (aguarde 5 segundos)..." -ForegroundColor Yellow
Start-Sleep -Seconds 5
Write-Host ""
Write-Host "--- Backend ---" -ForegroundColor Cyan
docker compose -f docker-compose.prod.yml logs --tail=10 backend
Write-Host ""
Write-Host "--- AIResearch ---" -ForegroundColor Cyan
docker compose -f docker-compose.prod.yml logs --tail=5 airesearch
Write-Host ""
Write-Host "--- ScienceAI ---" -ForegroundColor Cyan
docker compose -f docker-compose.prod.yml logs --tail=5 scienceai
Write-Host ""

Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  ✅ Limpeza e reinicialização concluídas" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "💡 Comandos úteis:" -ForegroundColor Yellow
Write-Host "   - Ver logs em tempo real: docker compose -f docker-compose.prod.yml logs -f [servico]" -ForegroundColor Gray
Write-Host "   - Ver status: docker compose -f docker-compose.prod.yml ps" -ForegroundColor Gray
Write-Host "   - Parar tudo: docker compose -f docker-compose.prod.yml down" -ForegroundColor Gray
Write-Host ""



