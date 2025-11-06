# Script para testar o pipeline completo de notícias (Collector + Writer)
Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  📰 TESTE COMPLETO: COLLECTOR + WRITER DE NOTÍCIAS          ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Parar processos existentes que podem estar bloqueando o executável
Write-Host "🛑 Parando processos existentes..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -like "*news-backend*"} | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Navegar para o diretório do backend
Set-Location "G:\Hive-Hub\News-main\news-backend"

# Executar o pipeline completo
Write-Host ""
Write-Host "✅ Executando pipeline completo..." -ForegroundColor Green
Write-Host "   Inclui: Coleta → Filtro → Escrita → Limpeza" -ForegroundColor Gray
Write-Host ""
cargo run --release pipeline





