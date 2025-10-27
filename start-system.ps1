# News System - Start Script
# Executa o orquestrador completo do sistema

Write-Host "🚀 News System - Starting Full Orchestrator" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Navegar para o diretório
Set-Location "G:\Hive-Hub\News-main"

# Executar o orquestrador
cargo run --bin start start

Write-Host ""
Write-Host "✅ System started successfully!" -ForegroundColor Green

