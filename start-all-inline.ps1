# Script para iniciar Backend e Frontend no mesmo terminal usando jobs
# News System - Start All Services (Inline)

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

# Função para iniciar backend
function Start-Backend {
    param($dir)
    Set-Location $dir
    cargo run --release
}

# Função para iniciar frontend
function Start-Frontend {
    param($dir)
    Set-Location $dir
    npm run dev
}

# Iniciar Backend em background job
Write-Host "📦 Iniciando Backend (Rust)..." -ForegroundColor Yellow
$backendJob = Start-Job -ScriptBlock ${function:Start-Backend} -ArgumentList $backendDir

# Aguardar um pouco
Start-Sleep -Seconds 3

# Iniciar Frontend em background job
Write-Host "🌐 Iniciando Frontend (Vite)..." -ForegroundColor Yellow
$frontendJob = Start-Job -ScriptBlock ${function:Start-Frontend} -ArgumentList $frontendDir

Write-Host ""
Write-Host "✅ Serviços iniciados em background!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Backend: http://localhost:8080 (porta padrão)" -ForegroundColor Cyan
Write-Host "📍 Frontend: http://localhost:5173 (porta padrão do Vite)" -ForegroundColor Cyan
Write-Host ""
Write-Host "📊 Para ver logs:" -ForegroundColor Yellow
Write-Host "   Receive-Job -Id $($backendJob.Id) -Keep    # Backend logs" -ForegroundColor Gray
Write-Host "   Receive-Job -Id $($frontendJob.Id) -Keep   # Frontend logs" -ForegroundColor Gray
Write-Host ""
Write-Host "🛑 Para encerrar:" -ForegroundColor Yellow
Write-Host "   Stop-Job -Id $($backendJob.Id),$($frontendJob.Id)" -ForegroundColor Gray
Write-Host "   Remove-Job -Id $($backendJob.Id),$($frontendJob.Id)" -ForegroundColor Gray
Write-Host ""
Write-Host "💡 Pressione Ctrl+C para encerrar tudo" -ForegroundColor Yellow
Write-Host ""

# Manter o script rodando e mostrar logs
try {
    while ($true) {
        Start-Sleep -Seconds 5
        
        # Mostrar logs do backend
        $backendOutput = Receive-Job -Id $backendJob.Id -ErrorAction SilentlyContinue
        if ($backendOutput) {
            Write-Host "[BACKEND] $backendOutput" -ForegroundColor Green
        }
        
        # Mostrar logs do frontend
        $frontendOutput = Receive-Job -Id $frontendJob.Id -ErrorAction SilentlyContinue
        if ($frontendOutput) {
            Write-Host "[FRONTEND] $frontendOutput" -ForegroundColor Blue
        }
        
        # Verificar se jobs ainda estão rodando
        if ($backendJob.State -eq "Failed" -or $frontendJob.State -eq "Failed") {
            Write-Host "❌ Um dos serviços falhou!" -ForegroundColor Red
            break
        }
    }
} finally {
    Write-Host ""
    Write-Host "🛑 Encerrando serviços..." -ForegroundColor Yellow
    Stop-Job -Id $backendJob.Id,$frontendJob.Id -ErrorAction SilentlyContinue
    Remove-Job -Id $backendJob.Id,$frontendJob.Id -ErrorAction SilentlyContinue
    Write-Host "✅ Serviços encerrados!" -ForegroundColor Green
}



