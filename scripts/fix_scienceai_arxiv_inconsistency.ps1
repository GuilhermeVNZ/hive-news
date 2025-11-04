# Script para corrigir inconsistência de artigos arXiv no registry do ScienceAI

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"
$airesearchDir = Join-Path $outputDir "AIResearch"

Write-Host "=== Corrigindo Inconsistencia de Artigos arXiv ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$fixed = 0
$checked = 0

# Verificar artigos Published para ScienceAI
foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    # Verificar se é artigo arXiv
    if ($id -notmatch "^\d{4}\.\d{4,6}") {
        continue
    }
    
    # Verificar se tem output_dir apontando para ScienceAI
    $hasScienceAIOutputDir = $false
    if ($article.output_dir) {
        $outputDirPath = $article.output_dir -replace '\\', '/'
        if ($outputDirPath -like "*ScienceAI*") {
            $hasScienceAIOutputDir = $true
        }
    }
    
    # Verificar destinations
    $hasScienceAIDest = $false
    $hasAIResearchDest = $false
    if ($article.destinations) {
        foreach ($dest in $article.destinations) {
            if ($dest.ToLower() -eq "scienceai") {
                $hasScienceAIDest = $true
            }
            if ($dest.ToLower() -eq "airesearch") {
                $hasAIResearchDest = $true
            }
        }
    }
    
    # Verificar se existe no filesystem do ScienceAI
    $existsInScienceAI = $false
    if ($hasScienceAIOutputDir) {
        $outputDirPath = $article.output_dir
        if (Test-Path $outputDirPath) {
            $titleFile = Join-Path $outputDirPath "title.txt"
            if (Test-Path $titleFile) {
                $existsInScienceAI = $true
            }
        }
    }
    
    # Verificar se existe no filesystem do AIResearch
    $existsInAIResearch = $false
    $arxivId = $id
    if (Test-Path $airesearchDir) {
        $folders = Get-ChildItem -Path $airesearchDir -Directory -ErrorAction SilentlyContinue | Where-Object { 
            $_.Name -like "*$arxivId*"
        }
        
        foreach ($folder in $folders) {
            $titleFile = Join-Path $folder.FullName "title.txt"
            if (Test-Path $titleFile) {
                $existsInAIResearch = $true
                break
            }
        }
    }
    
    # Se tem output_dir apontando para ScienceAI mas não existe lá, e existe no AIResearch
    if ($hasScienceAIOutputDir -and -not $existsInScienceAI -and $existsInAIResearch) {
        $checked++
        
        # Corrigir: remover output_dir do ScienceAI se destinations não incluir scienceai
        if (-not $hasScienceAIDest) {
            Write-Host "[$checked] Corrigindo ID: $id" -ForegroundColor Yellow
            Write-Host "   Output Dir atual: $($article.output_dir)" -ForegroundColor Gray
            Write-Host "   Destinations: $($article.destinations -join ', ')" -ForegroundColor Gray
            Write-Host "   Existe no AIResearch: $existsInAIResearch" -ForegroundColor Gray
            Write-Host "   Existe no ScienceAI: $existsInScienceAI" -ForegroundColor Gray
            
            # Remover output_dir se destinations não incluir scienceai
            $article.output_dir = $null
            $article.PSObject.Properties.Remove('output_dir')
            
            $fixed++
            Write-Host "   OK Output_dir removido (artigo nao e destinado ao ScienceAI)" -ForegroundColor Green
        }
    }
}

# Salvar registry atualizado
Write-Host ""
Write-Host "=== Salvando registry atualizado ===" -ForegroundColor Cyan
try {
    $json = $registry | ConvertTo-Json -Depth 100
    $json | Set-Content $registryPath -Encoding UTF8
    Write-Host "OK Registry salvo com sucesso!" -ForegroundColor Green
} catch {
    Write-Host "ERRO ao salvar registry: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Resumo
Write-Host ""
Write-Host "=== Resumo ===" -ForegroundColor Cyan
Write-Host "Verificados: $checked" -ForegroundColor White
Write-Host "Corrigidos: $fixed" -ForegroundColor Green
Write-Host ""

