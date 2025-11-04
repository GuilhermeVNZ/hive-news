# Script para preencher generated_title para artigos Published do ScienceAI que nao tem

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"

Write-Host "=== Preenchendo generated_title para ScienceAI ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$updated = 0
$notFound = 0

function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $null
}

function Find-ArticleInFilesystem {
    param(
        [string]$Id,
        [string]$BaseDir
    )
    
    if (-not (Test-Path $BaseDir)) {
        return $false, $null
    }
    
    $arxivId = Get-ArxivId $Id
    
    # Procurar por diferentes formatos de pasta
    $folders = Get-ChildItem -Path $BaseDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $folderName = $folder.Name
        
        # Tentar matching por ID completo
        if ($folderName -like "*$Id*") {
            $titleFile = Join-Path $folder.FullName "title.txt"
            if (Test-Path $titleFile) {
                $title = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                return $true, $title
            }
        }
        
        # Para artigos arXiv, tentar matching por arXiv ID
        if ($arxivId) {
            if ($folderName -like "*$arxivId*") {
                $titleFile = Join-Path $folder.FullName "title.txt"
                if (Test-Path $titleFile) {
                    $title = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                    return $true, $title
                }
            }
        }
    }
    
    return $false, $null
}

# Verificar artigos Published para ScienceAI
foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    # Verificar se tem destino ScienceAI
    $hasScienceAIDest = $false
    if ($article.destinations) {
        foreach ($dest in $article.destinations) {
            if ($dest.ToLower() -eq "scienceai") {
                $hasScienceAIDest = $true
                break
            }
        }
    }
    
    if (-not $hasScienceAIDest -and $article.output_dir) {
        $outputDirPath = $article.output_dir -replace '\\', '/'
        if ($outputDirPath -like "*ScienceAI*") {
            $hasScienceAIDest = $true
        }
    }
    
    if ($hasScienceAIDest) {
        # Verificar se tem generated_title
        $hasGeneratedTitle = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
        
        if (-not $hasGeneratedTitle) {
            # Procurar title.txt no filesystem
            $foundInFilesystem, $titleFromFile = Find-ArticleInFilesystem $id $scienceaiDir
            
            if ($foundInFilesystem -and $titleFromFile) {
                # Atualizar registry
                if ($article.PSObject.Properties.Name -contains "generated_title") {
                    $article.generated_title = $titleFromFile
                } else {
                    $article | Add-Member -NotePropertyName "generated_title" -NotePropertyValue $titleFromFile -Force
                }
                $updated++
                Write-Host "[$updated] OK ID: $id" -ForegroundColor Green
                Write-Host "   Titulo: $($titleFromFile.Substring(0, [Math]::Min(60, $titleFromFile.Length)))..." -ForegroundColor Gray
            } else {
                $notFound++
                Write-Host "[$notFound] ERRO ID: $id - title.txt nao encontrado" -ForegroundColor Red
            }
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
Write-Host "Atualizados: $updated" -ForegroundColor Green
Write-Host "title.txt nao encontrado: $notFound" -ForegroundColor Red
Write-Host ""

