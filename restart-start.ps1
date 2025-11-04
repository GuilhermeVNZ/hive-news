# Script para iniciar o Orquestrador (start.rs)
Write-Host "🚀 Iniciando News System Orchestrator (start.rs)..." -ForegroundColor Cyan
Write-Host ""

# Navegar para o diretório do News-main
Set-Location "G:\Hive-Hub\News-main"

# Iniciar o orquestrador
Write-Host "✅ Iniciando orquestrador..." -ForegroundColor Green
Write-Host "   Comando: cargo run --bin start start" -ForegroundColor Yellow
Write-Host ""

cargo run --bin start start








