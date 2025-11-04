# Script para reiniciar o Backend do Dashboard
Write-Host "🔧 Reiniciando Backend na porta 3005..." -ForegroundColor Cyan

# Parar processos existentes
Get-Process | Where-Object {$_.ProcessName -like "*news-backend*"} | Stop-Process -Force -ErrorAction SilentlyContinue

# Navegar para o diretório do backend
Set-Location "G:\Hive-Hub\News-main\news-backend"

# Iniciar o backend
Write-Host "✅ Iniciando backend..." -ForegroundColor Green
cargo run








