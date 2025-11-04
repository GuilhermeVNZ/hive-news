# Script para reiniciar o Frontend do Dashboard
Write-Host "🎨 Reiniciando Dashboard na porta 1420..." -ForegroundColor Cyan

# Parar processos Node.js relacionados ao dashboard (se houver)
$nodeProcesses = Get-Process | Where-Object {$_.ProcessName -eq "node"}
foreach ($proc in $nodeProcesses) {
    try {
        $cmdline = (Get-CimInstance Win32_Process -Filter "ProcessId = $($proc.Id)").CommandLine
        if ($cmdline -like "*news-dashboard*" -or $cmdline -like "*1420*") {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            Write-Host "   Parado processo Node.js: $($proc.Id)" -ForegroundColor Yellow
        }
    } catch {
        # Ignorar erros ao verificar command line
    }
}

# Navegar para o diretório do dashboard
Set-Location "G:\Hive-Hub\News-main\news-dashboard"

# Iniciar o dashboard
Write-Host "✅ Iniciando dashboard..." -ForegroundColor Green
npm run dev








