# Script para encerrar todos os processos do Backend e Frontend
# News System - Kill All Processes

Write-Host "Encerrando processos do Backend e Frontend..." -ForegroundColor Red
Write-Host "=============================================" -ForegroundColor Red
Write-Host ""

# Coletar todos os PIDs relacionados
$allPids = @()

# Processos do Backend
Write-Host "Procurando processos do Backend..." -ForegroundColor Yellow
$backendProcesses = Get-Process -ErrorAction SilentlyContinue | Where-Object { 
    $_.Path -like "*news-backend*" -or 
    $_.ProcessName -like "*news-backend*" -or
    ($_.ProcessName -eq "cargo" -and $_.Path -like "*News-main*")
}

if ($backendProcesses) {
    Write-Host "Encontrados $($backendProcesses.Count) processo(s) do Backend:" -ForegroundColor Cyan
    $backendProcesses | ForEach-Object { 
        Write-Host "  - $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
        $allPids += $_.Id
    }
} else {
    Write-Host "Nenhum processo do Backend encontrado" -ForegroundColor Green
}

# Processos do Frontend
Write-Host "Procurando processos do Frontend..." -ForegroundColor Yellow
$frontendProcesses = Get-Process -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -like "*ScienceAI*" -or
    $_.Path -like "*frontend-next*" -or
    ($_.ProcessName -eq "node" -and ($_.Path -like "*News-main*" -or $_.Path -like "*ScienceAI*" -or $_.Path -like "*frontend-next*")) -or
    $_.ProcessName -like "*vite*"
}

if ($frontendProcesses) {
    Write-Host "Encontrados $($frontendProcesses.Count) processo(s) do Frontend:" -ForegroundColor Cyan
    $frontendProcesses | ForEach-Object { 
        Write-Host "  - $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
        $allPids += $_.Id
    }
} else {
    Write-Host "Nenhum processo do Frontend encontrado" -ForegroundColor Green
}

# Remover duplicatas
$allPids = $allPids | Select-Object -Unique

# Encerrar processos
if ($allPids.Count -gt 0) {
    Write-Host ""
    Write-Host "Encerrando $($allPids.Count) processo(s)..." -ForegroundColor Red
    
    $successCount = 0
    $failCount = 0
    
    foreach ($pid in $allPids) {
        try {
            $process = Get-Process -Id $pid -ErrorAction SilentlyContinue
            if ($process) {
                Stop-Process -Id $pid -Force -ErrorAction Stop
                Write-Host "  Processo $pid ($($process.ProcessName)) encerrado" -ForegroundColor Green
                $successCount++
            }
        } catch {
            Write-Host "  Erro ao encerrar processo $pid : $_" -ForegroundColor Red
            $failCount++
        }
    }
    
    # Aguardar um pouco para garantir que os processos foram encerrados
    Start-Sleep -Seconds 2
    
    Write-Host ""
    Write-Host "Resumo:" -ForegroundColor Cyan
    Write-Host "  Encerrados: $successCount" -ForegroundColor Green
    if ($failCount -gt 0) {
        Write-Host "  Falhas: $failCount" -ForegroundColor Red
    }
} else {
    Write-Host ""
    Write-Host "Nenhum processo encontrado para encerrar!" -ForegroundColor Green
}

# Verificação final
Write-Host ""
Write-Host "Verificacao final..." -ForegroundColor Yellow
$remaining = Get-Process -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -like "*news-backend*" -or
    $_.Path -like "*ScienceAI*" -or
    $_.Path -like "*frontend-next*" -or
    ($_.ProcessName -eq "cargo" -and $_.Path -like "*News-main*") -or
    ($_.ProcessName -eq "node" -and ($_.Path -like "*News-main*" -or $_.Path -like "*ScienceAI*" -or $_.Path -like "*frontend-next*"))
}

if ($remaining) {
    Write-Host "Ainda ha processos rodando:" -ForegroundColor Yellow
    $remaining | Format-Table ProcessName, Id, Path -AutoSize
    Write-Host "Tente executar novamente ou encerre manualmente" -ForegroundColor Yellow
} else {
    Write-Host "Todos os processos foram encerrados com sucesso!" -ForegroundColor Green
}

Write-Host ""
