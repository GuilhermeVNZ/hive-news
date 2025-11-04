# Script para verificar se artigos exibidos usam generated_title correto

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$airesearchDir = Join-Path $outputDir "AIResearch"

Write-Host "=== Verificação de Artigos Exibidos ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host "Output: $outputDir" -ForegroundColor Gray
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
$publishedAIResearch = @()

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
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
    
    # Se não tem destinations, verificar pelo output_dir
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
            Write-Host "[AVISO] Artigo $id não tem generated_title" -ForegroundColor Yellow
            continue
        }
        
        # Procurar no filesystem
        $arxivId = Get-ArxivId $id
        $foundInFilesystem = $false
        $titleFromFile = $null
        $articlePath = $null
        
        if (Test-Path $airesearchDir) {
            $folders = Get-ChildItem -Path $airesearchDir -Directory -ErrorAction SilentlyContinue | Where-Object { 
                $_.Name -like "*$arxivId*" -or $_.Name -like "*$id*"
            }
            
            foreach ($folder in $folders) {
                $titleFile = Join-Path $folder.FullName "title.txt"
                if (Test-Path $titleFile) {
                    $foundInFilesystem = $true
                    $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                    $articlePath = $folder.FullName
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
                $publishedAIResearch += [PSCustomObject]@{
                    ID = $id
                    GeneratedTitle = $titleFromRegistry
                    TitleFromFile = $titleFromFile
                    Status = "✅ OK"
                }
            } else {
                $titleMismatches++
                $publishedAIResearch += [PSCustomObject]@{
                    ID = $id
                    GeneratedTitle = $titleFromRegistry
                    TitleFromFile = $titleFromFile
                    Status = "❌ MISMATCH"
                }
            }
        } else {
            $missingInFilesystem++
            $publishedAIResearch += [PSCustomObject]@{
                ID = $id
                GeneratedTitle = $article.generated_title
                TitleFromFile = "NÃO ENCONTRADO"
                Status = "❌ SEM ARQUIVO"
            }
        }
    }
}

# Verificar artigos órfãos no filesystem
Write-Host ""
Write-Host "=== Verificando Artigos Órfãos no Filesystem ===" -ForegroundColor Cyan

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
                foreach ($prop in $articles) {
                    $id = $prop.Name
                    $article = $prop.Value
                    
                    if ($article.status -eq "Published") {
                        $arxivId = Get-ArxivId $id
                        
                        # Verificar se arXiv ID corresponde
                        if ($arxivId -eq $arxivIdFromFolder -or $folderName -like "*$arxivId*") {
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
                            
                            if ($hasAIResearchDest) {
                                $foundInRegistry = $true
                                break
                            }
                        }
                    }
                }
                
                if (-not $foundInRegistry) {
                    $orphans++
                    $orphanArticles += [PSCustomObject]@{
                        FolderName = $folderName
                        Title = $titleFromFile.Substring(0, [Math]::Min(60, $titleFromFile.Length))
                        Path = $folder.FullName
                    }
                }
            }
        }
    }
}

# Relatório
Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published: $totalPublished" -ForegroundColor White
Write-Host "Published para AIResearch: $publishedForAIResearch" -ForegroundColor White
Write-Host ""

Write-Host "Artigos Exibidos (Registry + Filesystem):" -ForegroundColor Cyan
Write-Host "  ✅ Encontrados no registry E filesystem: $inRegistryAndFilesystem" -ForegroundColor Green
Write-Host "  ❌ No registry mas sem arquivo: $missingInFilesystem" -ForegroundColor Red
Write-Host ""

Write-Host "Consistência de Títulos:" -ForegroundColor Cyan
Write-Host "  ✅ Títulos correspondem (generated_title = title.txt): $usingGeneratedTitle" -ForegroundColor Green
Write-Host "  ❌ Títulos não correspondem: $titleMismatches" -ForegroundColor Red
Write-Host ""

Write-Host "Artigos Órfãos (Filesystem sem Registry):" -ForegroundColor Cyan
Write-Host "  ⚠️  Artigos no filesystem não no registry: $orphans" -ForegroundColor Yellow
Write-Host "  (Estes artigos NÃO serão exibidos no AIResearch)" -ForegroundColor Gray
Write-Host ""

# Detalhes dos problemas
if ($titleMismatches -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "TÍTULOS NÃO CORRESPONDEM" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    $publishedAIResearch | Where-Object { $_.Status -eq "❌ MISMATCH" } | Select-Object -First 10 | Format-Table -AutoSize
    if ($titleMismatches -gt 10) {
        Write-Host "... e mais $($titleMismatches - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($missingInFilesystem -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "ARTIGOS NO REGISTRY SEM ARQUIVO" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    $publishedAIResearch | Where-Object { $_.Status -eq "❌ SEM ARQUIVO" } | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingInFilesystem -gt 10) {
        Write-Host "... e mais $($missingInFilesystem - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($orphans -gt 0) {
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "ARTIGOS ÓRFÃOS (NÃO SERÃO EXIBIDOS)" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "Estes artigos existem no filesystem mas não estão no registry como Published para AIResearch" -ForegroundColor Gray
    Write-Host "Eles NÃO serão exibidos no AIResearch porque o frontend só mostra artigos que estão no registry" -ForegroundColor Gray
    Write-Host ""
    $orphanArticles | Select-Object -First 10 | Format-Table -AutoSize
    if ($orphans -gt 10) {
        Write-Host "... e mais $($orphans - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

# Resumo final
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMO FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($titleMismatches -eq 0 -and $missingInFilesystem -eq 0) {
    Write-Host "✅ PERFEITO! Todos os artigos exibidos estão corretos!" -ForegroundColor Green
    Write-Host "✅ Todos usam generated_title correto!" -ForegroundColor Green
    Write-Host "✅ Todos estão presentes no registry!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Artigos exibidos:" -ForegroundColor White
    Write-Host "  - Collection Logs: $usingGeneratedTitle artigos" -ForegroundColor Green
    Write-Host "  - AIResearch: $usingGeneratedTitle artigos" -ForegroundColor Green
    Write-Host ""
    if ($orphans -gt 0) {
        Write-Host "⚠️  Nota: $orphans artigos órfãos no filesystem não serão exibidos" -ForegroundColor Yellow
        Write-Host "   (Isso é esperado - o AIResearch só mostra artigos do registry)" -ForegroundColor Gray
    }
} else {
    Write-Host "⚠️  Problemas encontrados:" -ForegroundColor Yellow
    if ($titleMismatches -gt 0) {
        Write-Host "   - $titleMismatches artigos com títulos não correspondentes" -ForegroundColor Yellow
    }
    if ($missingInFilesystem -gt 0) {
        Write-Host "   - $missingInFilesystem artigos no registry sem arquivo no filesystem" -ForegroundColor Yellow
    }
}

Write-Host ""

