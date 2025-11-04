# Script para iniciar Backend e Frontend simultaneamente
# News System - Start All Services

Write-Host "🚀 Iniciando News System - Backend e Frontend" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Definir diretórios
$backendDir = "G:\Hive-Hub\News-main\news-backend"
$frontendDir = "G:\Hive-Hub\News-main\apps\frontend-next\ScienceAI"

# Verificar se os diretórios existem
if (-not (Test-Path $backendDir)) {
    Write-Host "❌ Erro: Diretório do backend não encontrado: $backendDir" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $frontendDir)) {
    Write-Host "❌ Erro: Diretório do frontend não encontrado: $frontendDir" -ForegroundColor Red
    exit 1
}

# Iniciar Backend em nova janela
Write-Host "📦 Iniciando Backend (Rust)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$backendDir'; Write-Host '🔧 Backend iniciando...' -ForegroundColor Green; cargo run --release"

# Aguardar um pouco para o backend iniciar
Start-Sleep -Seconds 3

# Iniciar Frontend em nova janela
Write-Host "🌐 Iniciando Frontend (Vite)..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$frontendDir'; Write-Host '⚡ Frontend iniciando...' -ForegroundColor Green; npm run dev"

Write-Host ""
Write-Host "✅ Serviços iniciados em janelas separadas!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Backend: http://localhost:8080 (porta padrão)" -ForegroundColor Cyan
Write-Host "📍 Frontend: http://localhost:5173 (porta padrão do Vite)" -ForegroundColor Cyan
Write-Host ""
Write-Host "💡 Para encerrar, feche as janelas do PowerShell ou use Ctrl+C em cada uma" -ForegroundColor Yellow
Write-Host ""



