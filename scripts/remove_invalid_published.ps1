# Script para remover artigos Published que não têm generated_title e não têm title.txt

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"

Write-Host "=== Removendo Artigos Published Inválidos ===" -ForegroundColor Red
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host "Output: $outputDir" -ForegroundColor Gray
Write-Host ""
Write-Host "⚠️  ATENÇÃO: Este script irá REMOVER artigos e pastas permanentemente!" -ForegroundColor Yellow
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$toRemove = @()
$removedFromRegistry = 0
$removedFolders = 0
$folderErrors = 0

function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $Id
}

# Identificar artigos para remover
Write-Host "=== Identificando artigos inválidos ===" -ForegroundColor Cyan
foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    # Verificar se não tem generated_title
    $hasGenerated = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if ($hasGenerated) {
        continue
    }
    
    # Verificar se não tem title.txt no filesystem
    $hasTitleFile = $false
    
    # Primeiro tentar pelo output_dir
    if ($article.output_dir) {
        $outputDirPath = $article.output_dir -replace '\\', '/'
        $titleFilePath = Join-Path $outputDirPath "title.txt"
        
        if (Test-Path $titleFilePath) {
            $hasTitleFile = $true
        }
    }
    
    # Se não encontrou, buscar em AIResearch ou ScienceAI
    if (-not $hasTitleFile) {
        $arxivId = Get-ArxivId $id
        
        foreach ($siteDir in @("AIResearch", "ScienceAI")) {
            $sitePath = Join-Path $outputDir $siteDir
            if (Test-Path $sitePath) {
                $folders = Get-ChildItem -Path $sitePath -Directory -ErrorAction SilentlyContinue | Where-Object { 
                    $_.Name -like "*$arxivId*" -or $_.Name -like "*$id*"
                }
                
                foreach ($folder in $folders) {
                    $titleFile = Join-Path $folder.FullName "title.txt"
                    if (Test-Path $titleFile) {
                        $hasTitleFile = $true
                        break
                    }
                }
                
                if ($hasTitleFile) { break }
            }
        }
    }
    
    # Se não tem generated_title E não tem title.txt, adicionar à lista para remover
    if (-not $hasTitleFile) {
        $toRemove += [PSCustomObject]@{
            ID = $id
            OutputDir = $article.output_dir
            Title = if ($article.title) { $article.title.Substring(0, [Math]::Min(60, $article.title.Length)) } else { "N/A" }
        }
    }
}

Write-Host "Artigos identificados para remoção: $($toRemove.Count)" -ForegroundColor Yellow
Write-Host ""

if ($toRemove.Count -eq 0) {
    Write-Host "✅ Nenhum artigo inválido encontrado!" -ForegroundColor Green
    exit 0
}

# Mostrar lista de artigos que serão removidos
Write-Host "=== Artigos que serão removidos ===" -ForegroundColor Red
$toRemove | Format-Table -AutoSize
Write-Host ""

# Confirmar remoção (automático)
Write-Host "⚠️  Removendo $($toRemove.Count) artigos..." -ForegroundColor Yellow
Write-Host "   - Artigos serão removidos do registry" -ForegroundColor Gray
Write-Host "   - Pastas serão removidas do filesystem" -ForegroundColor Gray
Write-Host "   - Esta ação NÃO pode ser desfeita!" -ForegroundColor Red
Write-Host ""

Write-Host ""
Write-Host "=== Removendo artigos ===" -ForegroundColor Cyan

# Remover do registry
Write-Host "Removendo do registry..." -ForegroundColor Gray
foreach ($item in $toRemove) {
    if ($registry.articles.PSObject.Properties.Name -contains $item.ID) {
        $registry.articles.PSObject.Properties.Remove($item.ID)
        $removedFromRegistry++
        Write-Host "[$removedFromRegistry] ✅ Removido do registry: $($item.ID)" -ForegroundColor Green
    }
}

# Remover pastas do filesystem
Write-Host ""
Write-Host "Removendo pastas do filesystem..." -ForegroundColor Gray
foreach ($item in $toRemove) {
    $removed = $false
    
    # Tentar remover pelo output_dir
    if ($item.OutputDir) {
        $outputDirPath = $item.OutputDir -replace '\\', '/'
        if (Test-Path $outputDirPath) {
            try {
                Remove-Item -Path $outputDirPath -Recurse -Force -ErrorAction Stop
                $removed = $true
                $removedFolders++
                Write-Host "[$removedFolders] ✅ Pasta removida: $outputDirPath" -ForegroundColor Green
            } catch {
                $folderErrors++
                Write-Host "[ERRO] Não foi possível remover: $outputDirPath - $($_.Exception.Message)" -ForegroundColor Red
            }
        }
    }
    
    # Se não removeu pelo output_dir, tentar buscar e remover
    if (-not $removed) {
        $arxivId = Get-ArxivId $item.ID
        
        foreach ($siteDir in @("AIResearch", "ScienceAI")) {
            $sitePath = Join-Path $outputDir $siteDir
            if (Test-Path $sitePath) {
                $folders = Get-ChildItem -Path $sitePath -Directory -ErrorAction SilentlyContinue | Where-Object { 
                    $_.Name -like "*$arxivId*" -or $_.Name -like "*$($item.ID)*"
                }
                
                foreach ($folder in $folders) {
                    try {
                        Remove-Item -Path $folder.FullName -Recurse -Force -ErrorAction Stop
                        $removed = $true
                        $removedFolders++
                        Write-Host "[$removedFolders] ✅ Pasta removida: $($folder.FullName)" -ForegroundColor Green
                    } catch {
                        $folderErrors++
                        Write-Host "[ERRO] Não foi possível remover: $($folder.FullName) - $($_.Exception.Message)" -ForegroundColor Red
                    }
                }
            }
        }
    }
    
    if (-not $removed) {
        Write-Host "[AVISO] Pasta não encontrada para: $($item.ID)" -ForegroundColor Yellow
    }
}

# Salvar registry atualizado
Write-Host ""
Write-Host "=== Salvando registry atualizado ===" -ForegroundColor Cyan
try {
    $json = $registry | ConvertTo-Json -Depth 100
    $json | Set-Content $registryPath -Encoding UTF8
    Write-Host "✅ Registry salvo com sucesso!" -ForegroundColor Green
} catch {
    Write-Host "❌ ERRO ao salvar registry: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Resumo
Write-Host ""
Write-Host "=== Resumo ===" -ForegroundColor Cyan
Write-Host "Artigos identificados: $($toRemove.Count)" -ForegroundColor White
Write-Host "✅ Removidos do registry: $removedFromRegistry" -ForegroundColor Green
Write-Host "✅ Pastas removidas: $removedFolders" -ForegroundColor Green
if ($folderErrors -gt 0) {
    Write-Host "⚠️  Erros ao remover pastas: $folderErrors" -ForegroundColor Yellow
}
Write-Host ""

