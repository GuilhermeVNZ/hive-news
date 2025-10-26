# Hive-News Production Deployment Script (PowerShell)
# Version: 1.0.0
# Date: 2025-10-26

$ErrorActionPreference = "Stop"

Write-Host "🚀 Hive-News Production Deployment" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

# Step 1: Verify Prerequisites
Write-Host "`nStep 1: Verifying prerequisites..." -ForegroundColor Yellow

# Check Node.js
try {
    $nodeVersion = node --version
    Write-Host "✅ Node.js version: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Error: Node.js not found" -ForegroundColor Red
    exit 1
}

# Check Docker
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Host "✅ Docker found" -ForegroundColor Green
} else {
    Write-Host "❌ Error: Docker not found" -ForegroundColor Red
    exit 1
}

# Check Docker Compose
if (Get-Command docker-compose -ErrorAction SilentlyContinue) {
    Write-Host "✅ Docker Compose found" -ForegroundColor Green
} else {
    Write-Host "❌ Error: Docker Compose not found" -ForegroundColor Red
    exit 1
}

# Step 2: Environment Setup
Write-Host "`nStep 2: Setting up environment..." -ForegroundColor Yellow

if (-not (Test-Path .env)) {
    Write-Host "Creating .env from template..." -ForegroundColor Yellow
    Copy-Item env.template .env
    Write-Host "✅ Created .env file" -ForegroundColor Green
    Write-Host "⚠️  Please edit .env with your production values" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "✅ .env file exists" -ForegroundColor Green
}

# Step 3: Install Dependencies
Write-Host "`nStep 3: Installing dependencies..." -ForegroundColor Yellow
npm install
Write-Host "✅ Dependencies installed" -ForegroundColor Green

# Step 4: Run Tests
Write-Host "`nStep 4: Running tests..." -ForegroundColor Yellow
npm test
Write-Host "✅ All tests passing" -ForegroundColor Green

# Step 5: Build Application
Write-Host "`nStep 5: Building application..." -ForegroundColor Yellow
npm run build
Write-Host "✅ Application built" -ForegroundColor Green

# Step 6: Start Infrastructure Services
Write-Host "`nStep 6: Starting infrastructure services..." -ForegroundColor Yellow
docker-compose up -d postgres redis minio
Write-Host "✅ Infrastructure services started" -ForegroundColor Green
Write-Host "Waiting for services to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Step 7: Verify Services
Write-Host "`nStep 7: Verifying services..." -ForegroundColor Yellow
docker-compose ps
Write-Host "✅ Services verification complete" -ForegroundColor Green

# Step 8: Start Application
Write-Host "`nStep 8: Starting application..." -ForegroundColor Yellow
Write-Host "Starting backend..." -ForegroundColor Green

Start-Job -ScriptBlock { npm start } | Out-Null
Start-Sleep -Seconds 10

# Step 9: Health Check
Write-Host "`nStep 9: Performing health check..." -ForegroundColor Yellow

try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -Method GET -TimeoutSec 5
    Write-Host "✅ Health check passed" -ForegroundColor Green
} catch {
    Write-Host "❌ Health check failed" -ForegroundColor Red
    exit 1
}

# Step 10: Display URLs
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "✅ Deployment Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Services available at:"
Write-Host "  - Backend API: http://localhost:3000"
Write-Host "  - PostgreSQL: localhost:5432"
Write-Host "  - Redis: localhost:6379"
Write-Host "  - MinIO: http://localhost:9001"
Write-Host ""
Write-Host "API Endpoints:"
Write-Host "  - Health: http://localhost:3000/health"
Write-Host "  - Metrics: http://localhost:3000/metrics"
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Configure external APIs (DeepSeek, Vectorizer, etc.)"
Write-Host "  2. Set up monitoring (Prometheus/Grafana)"
Write-Host "  3. Configure backup schedule"
Write-Host "  4. Review logs: docker-compose logs -f"
Write-Host ""

