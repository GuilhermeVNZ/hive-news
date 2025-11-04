# Script para verificar se artigos/noticias exibidos no ScienceAI usam generated_title correto
# Considera que noticias (news) têm IDs de hash diferentes de artigos (arXiv)

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"

Write-Host "=== Verificacao Completa ScienceAI ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$publishedForScienceAI = 0
$inRegistryAndFilesystem = 0
$usingGeneratedTitle = 0
$titleMismatches = 0
$missingInFilesystem = 0
$orphans = 0
$newsArticles = 0
$arxivArticles = 0

function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $null
}

function Is-NewsId {
    param([string]$Id)
    # News IDs são números puros (hash) ou não têm formato arXiv
    if ($Id -match "^\d+$") {
        return $true
    }
    if (-not (Get-ArxivId $Id)) {
        return $true
    }
    return $false
}

function Normalize-Text {
    param([string]$Text)
    if ([string]::IsNullOrWhiteSpace($Text)) {
        return ""
    }
    return $Text.Trim().ToLower() -replace '\s+', ' '
}

function Find-ArticleInFilesystem {
    param(
        [string]$Id,
        [string]$BaseDir
    )
    
    $found = $false
    $titleFromFile = $null
    $folderPath = $null
    
    if (-not (Test-Path $BaseDir)) {
        return $false, $null, $null
    }
    
    # Verificar se é news (hash) ou artigo (arXiv)
    $isNews = Is-NewsId $Id
    $arxivId = Get-ArxivId $Id
    
    # Procurar por diferentes formatos de pasta
    $folders = Get-ChildItem -Path $BaseDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $folderName = $folder.Name
        
        # Tentar matching por ID completo
        if ($folderName -like "*$Id*") {
            $titleFile = Join-Path $folder.FullName "title.txt"
            if (Test-Path $titleFile) {
                $found = $true
                $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                $folderPath = $folder.FullName
                break
            }
        }
        
        # Para artigos arXiv, tentar matching por arXiv ID
        if ($arxivId) {
            if ($folderName -like "*$arxivId*") {
                $titleFile = Join-Path $folder.FullName "title.txt"
                if (Test-Path $titleFile) {
                    $found = $true
                    $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                    $folderPath = $folder.FullName
                    break
                }
            }
        }
        
        # Para news, tentar matching por hash (últimos caracteres do nome da pasta)
        if ($isNews) {
            # Formato: YYYY-MM-DD_source_hash
            # Extrair hash do nome da pasta
            $parts = $folderName -split '_'
            if ($parts.Length -ge 3) {
                $hashFromFolder = $parts[-1]
                # Comparar com ID (pode ser hash completo ou parcial)
                if ($hashFromFolder -like "*$Id*" -or $Id -like "*$hashFromFolder*") {
                    $titleFile = Join-Path $folder.FullName "title.txt"
                    if (Test-Path $titleFile) {
                        $found = $true
                        $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                        $folderPath = $folder.FullName
                        break
                    }
                }
            }
        }
    }
    
    return $found, $titleFromFile, $folderPath
}

# Verificar artigos Published para ScienceAI
Write-Host "=== Verificando Artigos Published para ScienceAI ===" -ForegroundColor Cyan

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    
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
        $publishedForScienceAI++
        
        # Classificar como news ou artigo
        if (Is-NewsId $id) {
            $newsArticles++
        } else {
            $arxivArticles++
        }
        
        # Verificar se tem generated_title
        $hasGeneratedTitle = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
        
        if (-not $hasGeneratedTitle) {
            Write-Host "[AVISO] Artigo $id nao tem generated_title" -ForegroundColor Yellow
            continue
        }
        
        # Procurar no filesystem
        $foundInFilesystem, $titleFromFile, $folderPath = Find-ArticleInFilesystem $id $scienceaiDir
        
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
                Write-Host "[MISMATCH] ID: $id" -ForegroundColor Red
                Write-Host "  Registry: $($titleFromRegistry.Substring(0, [Math]::Min(60, $titleFromRegistry.Length)))..." -ForegroundColor Yellow
                Write-Host "  File: $($titleFromFile.Substring(0, [Math]::Min(60, $titleFromFile.Length)))..." -ForegroundColor Yellow
            }
        } else {
            $missingInFilesystem++
            Write-Host "[NAO ENCONTRADO] ID: $id" -ForegroundColor Red
        }
    }
}

# Verificar artigos órfãos no filesystem
Write-Host ""
Write-Host "=== Verificando Artigos Orfaos no Filesystem ===" -ForegroundColor Cyan

$orphanArticles = @()
if (Test-Path $scienceaiDir) {
    $folders = Get-ChildItem -Path $scienceaiDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $titleFile = Join-Path $folder.FullName "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
            if ($titleFromFile) {
                $folderName = $folder.Name
                
                # Extrair possível ID do nome da pasta
                # Formato: YYYY-MM-DD_source_ID ou YYYY-MM-DD_source_hash
                $parts = $folderName -split '_'
                $possibleId = $null
                
                if ($parts.Length -ge 3) {
                    $possibleId = $parts[-1] # Última parte pode ser ID ou hash
                }
                
                # Verificar se está no registry como Published para ScienceAI
                $foundInRegistry = $false
                foreach ($prop2 in $articles) {
                    $id2 = $prop2.Name
                    $article2 = $prop2.Value
                    
                    if ($article2.status -eq "Published") {
                        $hasScienceAIDest2 = $false
                        if ($article2.destinations) {
                            foreach ($dest in $article2.destinations) {
                                if ($dest.ToLower() -eq "scienceai") {
                                    $hasScienceAIDest2 = $true
                                    break
                                }
                            }
                        }
                        
                        if ($hasScienceAIDest2) {
                            # Tentar matching por ID completo
                            if ($folderName -like "*$id2*" -or $id2 -like "*$possibleId*") {
                                # Verificar se título corresponde
                                if ($article2.generated_title) {
                                    $normalizedFs = Normalize-Text $titleFromFile
                                    $normalizedReg = Normalize-Text $article2.generated_title
                                    
                                    if ($normalizedFs -eq $normalizedReg) {
                                        $foundInRegistry = $true
                                        break
                                    }
                                }
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
Write-Host "Published para ScienceAI: $publishedForScienceAI" -ForegroundColor White
Write-Host "  - Noticias (news): $newsArticles" -ForegroundColor Cyan
Write-Host "  - Artigos (arXiv): $arxivArticles" -ForegroundColor Cyan
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
Write-Host "  (AVISO: ScienceAI pode exibir estes artigos mesmo sem registry)" -ForegroundColor Yellow
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
    Write-Host "  - ScienceAI: $usingGeneratedTitle artigos (registry) + $orphans artigos orfaos" -ForegroundColor Green
    Write-Host ""
    if ($orphans -gt 0) {
        Write-Host "AVISO Nota: $orphans artigos orfaos no filesystem podem ser exibidos no ScienceAI" -ForegroundColor Yellow
        Write-Host "   (ScienceAI le diretamente do filesystem, nao filtra por registry)" -ForegroundColor Gray
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

