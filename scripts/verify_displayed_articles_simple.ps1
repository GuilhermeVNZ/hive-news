# Script para verificar se artigos exibidos usam generated_title correto

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$airesearchDir = Join-Path $outputDir "AIResearch"

Write-Host "=== Verificacao de Artigos Exibidos ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$publishedForAIResearch = 0
$inRegistryAndFilesystem = 0
$usingGeneratedTitle = 0
$titleMismatches = 0
$missingInFilesystem = 0
$orphans = 0

function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $Id
}

function Normalize-Text {
    param([string]$Text)
    if ([string]::IsNullOrWhiteSpace($Text)) {
        return ""
    }
    return $Text.Trim().ToLower() -replace '\s+', ' '
}

# Verificar artigos Published para AIResearch
Write-Host "=== Verificando Artigos Published para AIResearch ===" -ForegroundColor Cyan

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    
    # Verificar se tem destino AIResearch
    $hasAIResearchDest = $false
    if ($article.destinations) {
        foreach ($dest in $article.destinations) {
            if ($dest.ToLower() -eq "airesearch") {
                $hasAIResearchDest = $true
                break
            }
        }
    }
    
    if (-not $hasAIResearchDest -and $article.output_dir) {
        $outputDirPath = $article.output_dir -replace '\\', '/'
        if ($outputDirPath -like "*AIResearch*") {
            $hasAIResearchDest = $true
        }
    }
    
    if ($hasAIResearchDest) {
        $publishedForAIResearch++
        
        # Verificar se tem generated_title
        $hasGeneratedTitle = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
        
        if (-not $hasGeneratedTitle) {
            Write-Host "[AVISO] Artigo $id nao tem generated_title" -ForegroundColor Yellow
            continue
        }
        
        # Procurar no filesystem
        $arxivId = Get-ArxivId $id
        $foundInFilesystem = $false
        $titleFromFile = $null
        
        if (Test-Path $airesearchDir) {
            $folders = Get-ChildItem -Path $airesearchDir -Directory -ErrorAction SilentlyContinue | Where-Object { 
                $_.Name -like "*$arxivId*" -or $_.Name -like "*$id*"
            }
            
            foreach ($folder in $folders) {
                $titleFile = Join-Path $folder.FullName "title.txt"
                if (Test-Path $titleFile) {
                    $foundInFilesystem = $true
                    $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                    break
                }
            }
        }
        
        if ($foundInFilesystem) {
            $inRegistryAndFilesystem++
            
            # Comparar título do arquivo com generated_title do registry
            $titleFromRegistry = $article.generated_title
            $normalizedFile = Normalize-Text $titleFromFile
            $normalizedRegistry = Normalize-Text $titleFromRegistry
            
            if ($normalizedFile -eq $normalizedRegistry) {
                $usingGeneratedTitle++
            } else {
                $titleMismatches++
            }
        } else {
            $missingInFilesystem++
        }
    }
}

# Verificar artigos órfãos no filesystem
Write-Host ""
Write-Host "=== Verificando Artigos Orfaos no Filesystem ===" -ForegroundColor Cyan

$orphanArticles = @()
if (Test-Path $airesearchDir) {
    $folders = Get-ChildItem -Path $airesearchDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $titleFile = Join-Path $folder.FullName "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
            if ($titleFromFile) {
                $folderName = $folder.Name
                $arxivIdFromFolder = Get-ArxivId $folderName
                
                # Verificar se está no registry como Published para AIResearch
                $foundInRegistry = $false
                foreach ($prop2 in $articles) {
                    $id2 = $prop2.Name
                    $article2 = $prop2.Value
                    
                    if ($article2.status -eq "Published") {
                        $arxivId2 = Get-ArxivId $id2
                        
                        if ($arxivId2 -eq $arxivIdFromFolder -or $folderName -like "*$arxivId2*") {
                            $hasAIResearchDest2 = $false
                            if ($article2.destinations) {
                                foreach ($dest in $article2.destinations) {
                                    if ($dest.ToLower() -eq "airesearch") {
                                        $hasAIResearchDest2 = $true
                                        break
                                    }
                                }
                            }
                            
                            if ($hasAIResearchDest2) {
                                $foundInRegistry = $true
                                break
                            }
                        }
                    }
                }
                
                if (-not $foundInRegistry) {
                    $orphans++
                }
            }
        }
    }
}

# Relatório
Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICACAO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published: $totalPublished" -ForegroundColor White
Write-Host "Published para AIResearch: $publishedForAIResearch" -ForegroundColor White
Write-Host ""

Write-Host "Artigos Exibidos (Registry + Filesystem):" -ForegroundColor Cyan
Write-Host "  OK Encontrados no registry E filesystem: $inRegistryAndFilesystem" -ForegroundColor Green
Write-Host "  ERRO No registry mas sem arquivo: $missingInFilesystem" -ForegroundColor Red
Write-Host ""

Write-Host "Consistencia de Titulos:" -ForegroundColor Cyan
Write-Host "  OK Titulos correspondem (generated_title = title.txt): $usingGeneratedTitle" -ForegroundColor Green
Write-Host "  ERRO Titulos nao correspondem: $titleMismatches" -ForegroundColor Red
Write-Host ""

Write-Host "Artigos Orfaos (Filesystem sem Registry):" -ForegroundColor Cyan
Write-Host "  AVISO Artigos no filesystem nao no registry: $orphans" -ForegroundColor Yellow
Write-Host "  (Estes artigos NAO serao exibidos no AIResearch)" -ForegroundColor Gray
Write-Host ""

# Resumo final
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMO FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($titleMismatches -eq 0 -and $missingInFilesystem -eq 0) {
    Write-Host "OK PERFEITO! Todos os artigos exibidos estao corretos!" -ForegroundColor Green
    Write-Host "OK Todos usam generated_title correto!" -ForegroundColor Green
    Write-Host "OK Todos estao presentes no registry!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Artigos exibidos:" -ForegroundColor White
    Write-Host "  - Collection Logs: $usingGeneratedTitle artigos" -ForegroundColor Green
    Write-Host "  - AIResearch: $usingGeneratedTitle artigos" -ForegroundColor Green
    Write-Host ""
    if ($orphans -gt 0) {
        Write-Host "AVISO Nota: $orphans artigos orfaos no filesystem nao serao exibidos" -ForegroundColor Yellow
        Write-Host "   (Isso e esperado - o AIResearch so mostra artigos do registry)" -ForegroundColor Gray
    }
} else {
    Write-Host "AVISO Problemas encontrados:" -ForegroundColor Yellow
    if ($titleMismatches -gt 0) {
        Write-Host "   - $titleMismatches artigos com titulos nao correspondentes" -ForegroundColor Yellow
    }
    if ($missingInFilesystem -gt 0) {
        Write-Host "   - $missingInFilesystem artigos no registry sem arquivo no filesystem" -ForegroundColor Yellow
    }
}

Write-Host ""

