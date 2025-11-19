# Script para testar o collector de notícias (RSS/HTML)
Write-Host "📰 Testando Collectors de Notícias (RSS/HTML)..." -ForegroundColor Cyan
Write-Host ""

# Navegar para o diretório do backend
Set-Location "G:\Hive-Hub\News-main\news-backend"

# Executar o teste do collector de notícias
Write-Host "✅ Executando teste do collector de notícias..." -ForegroundColor Green
cargo run --release test-news-collector












































