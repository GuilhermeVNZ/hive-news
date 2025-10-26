# Hive-News Deployment Script (PowerShell)

Write-Host "🚀 Starting Hive-News deployment..." -ForegroundColor Green

# Check prerequisites
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Docker not found" -ForegroundColor Red
    exit 1
}

# Build Docker image
Write-Host "📦 Building Docker image..." -ForegroundColor Cyan
docker build -t hivenews/backend:latest -f docker/Dockerfile .

# Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Cyan
npm test

Write-Host "✅ Deployment complete!" -ForegroundColor Green



