# Script para verificar se todos os artigos exibidos usam generated_title correto

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$airesearchDir = Join-Path $outputDir "AIResearch"
$scienceaiDir = Join-Path $outputDir "ScienceAI"

Write-Host "=== Verificação de Consistência de Títulos ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host "Output: $outputDir" -ForegroundColor Gray
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$inRegistry = 0
$inFilesystem = 0
$titleMatches = 0
$titleMismatches = 0
$missingInFilesystem = 0
$missingGeneratedTitle = 0
$missingInRegistry = @()
$mismatches = @()
$missingFiles = @()

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

# Verificar artigos no registry
Write-Host "=== Verificando Artigos no Registry ===" -ForegroundColor Cyan
foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    $inRegistry++
    
    # Verificar se tem generated_title
    $hasGeneratedTitle = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if (-not $hasGeneratedTitle) {
        $missingGeneratedTitle++
        Write-Host "[AVISO] Artigo $id não tem generated_title" -ForegroundColor Yellow
        continue
    }
    
    # Verificar destinations para determinar qual site
    $siteDirs = @()
    if ($article.destinations) {
        foreach ($dest in $article.destinations) {
            $destLower = $dest.ToLower()
            if ($destLower -eq "airesearch") {
                $siteDirs += $airesearchDir
            } elseif ($destLower -eq "scienceai") {
                $siteDirs += $scienceaiDir
            }
        }
    } else {
        # Se não tem destinations, verificar pelo output_dir
        if ($article.output_dir) {
            $outputDirPath = $article.output_dir -replace '\\', '/'
            if ($outputDirPath -like "*AIResearch*") {
                $siteDirs += $airesearchDir
            } elseif ($outputDirPath -like "*ScienceAI*") {
                $siteDirs += $scienceaiDir
            }
        }
    }
    
    # Se não encontrou site, tentar ambos
    if ($siteDirs.Count -eq 0) {
        $siteDirs = @($airesearchDir, $scienceaiDir)
    }
    
    # Procurar artigo no filesystem
    $arxivId = Get-ArxivId $id
    $foundInFilesystem = $false
    $titleFromFile = $null
    $articlePath = $null
    
    foreach ($siteDir in $siteDirs) {
        if (Test-Path $siteDir) {
            $folders = Get-ChildItem -Path $siteDir -Directory -ErrorAction SilentlyContinue | Where-Object { 
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
            
            if ($foundInFilesystem) {
                break
            }
        }
    }
    
    if ($foundInFilesystem) {
        $inFilesystem++
        
        # Comparar título do arquivo com generated_title do registry
        $titleFromRegistry = $article.generated_title
        $normalizedFile = Normalize-Text $titleFromFile
        $normalizedRegistry = Normalize-Text $titleFromRegistry
        
        if ($normalizedFile -eq $normalizedRegistry) {
            $titleMatches++
        } else {
            $titleMismatches++
            $mismatches += [PSCustomObject]@{
                ID = $id
                GeneratedTitle = $titleFromRegistry.Substring(0, [Math]::Min(60, $titleFromRegistry.Length))
                TitleFromFile = $titleFromFile.Substring(0, [Math]::Min(60, $titleFromFile.Length))
                Path = $articlePath
            }
        }
    } else {
        $missingInFilesystem++
        $missingFiles += [PSCustomObject]@{
            ID = $id
            GeneratedTitle = $article.generated_title.Substring(0, [Math]::Min(60, $article.generated_title.Length))
            Destinations = if ($article.destinations) { ($article.destinations -join ", ") } else { "N/A" }
            OutputDir = $article.output_dir
        }
    }
}

# Verificar artigos no filesystem que não estão no registry
Write-Host ""
Write-Host "=== Verificando Artigos no Filesystem ===" -ForegroundColor Cyan

function Get-ArticlesFromFilesystem {
    param([string]$SiteDir, [string]$SiteName)
    
    $articlesFromFs = @()
    
    if (-not (Test-Path $SiteDir)) {
        return $articlesFromFs
    }
    
    $folders = Get-ChildItem -Path $SiteDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $titleFile = Join-Path $folder.FullName "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
            if ($titleFromFile) {
                $articlesFromFs += [PSCustomObject]@{
                    FolderName = $folder.Name
                    Title = $titleFromFile
                    Path = $folder.FullName
                    Site = $SiteName
                }
            }
        }
    }
    
    return $articlesFromFs
}

$airesearchArticles = Get-ArticlesFromFilesystem $airesearchDir "AIResearch"
$scienceaiArticles = Get-ArticlesFromFilesystem $scienceaiDir "ScienceAI"
$allFsArticles = $airesearchArticles + $scienceaiArticles

Write-Host "Artigos encontrados no filesystem:" -ForegroundColor White
Write-Host "  AIResearch: $($airesearchArticles.Count)" -ForegroundColor White
Write-Host "  ScienceAI: $($scienceaiArticles.Count)" -ForegroundColor White
Write-Host "  Total: $($allFsArticles.Count)" -ForegroundColor White
Write-Host ""

# Verificar se artigos do filesystem estão no registry
foreach ($fsArticle in $allFsArticles) {
    $found = $false
    
    # Tentar encontrar pelo folder name (pode conter arXiv ID)
    $folderName = $fsArticle.FolderName
    $arxivIdFromFolder = Get-ArxivId $folderName
    
    foreach ($prop in $articles) {
        $id = $prop.Name
        $article = $prop.Value
        
        if ($article.status -eq "Published") {
            $arxivId = Get-ArxivId $id
            
            # Comparar por arXiv ID ou folder name
            if ($folderName -like "*$arxivId*" -or $folderName -like "*$id*" -or $id -eq $arxivIdFromFolder) {
                # Verificar se o título corresponde
                if ($article.generated_title) {
                    $normalizedFs = Normalize-Text $fsArticle.Title
                    $normalizedReg = Normalize-Text $article.generated_title
                    
                    if ($normalizedFs -eq $normalizedReg) {
                        $found = $true
                        break
                    }
                }
            }
        }
    }
    
    if (-not $found) {
        $missingInRegistry += [PSCustomObject]@{
            FolderName = $fsArticle.FolderName
            Title = $fsArticle.Title.Substring(0, [Math]::Min(60, $fsArticle.Title.Length))
            Path = $fsArticle.Path
            Site = $fsArticle.Site
        }
    }
}

# Relatório
Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published no registry: $totalPublished" -ForegroundColor White
Write-Host ""

Write-Host "Artigos no Registry:" -ForegroundColor Cyan
Write-Host "  ✅ Com generated_title: $($totalPublished - $missingGeneratedTitle)" -ForegroundColor Green
Write-Host "  ⚠️  Sem generated_title: $missingGeneratedTitle" -ForegroundColor Yellow
Write-Host ""

Write-Host "Artigos no Filesystem:" -ForegroundColor Cyan
Write-Host "  ✅ Encontrados: $inFilesystem" -ForegroundColor Green
Write-Host "  ❌ Não encontrados: $missingInFilesystem" -ForegroundColor Red
Write-Host ""

Write-Host "Consistência de Títulos:" -ForegroundColor Cyan
Write-Host "  ✅ Títulos correspondem: $titleMatches" -ForegroundColor Green
Write-Host "  ❌ Títulos não correspondem: $titleMismatches" -ForegroundColor Red
Write-Host ""

Write-Host "Artigos no Filesystem não no Registry:" -ForegroundColor Cyan
Write-Host "  ❌ Artigos órfãos: $($missingInRegistry.Count)" -ForegroundColor Red
Write-Host ""

# Detalhes dos problemas
if ($missingGeneratedTitle -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "ARTIGOS SEM generated_title" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "$missingGeneratedTitle artigos Published não têm generated_title no registry" -ForegroundColor Yellow
    Write-Host ""
}

if ($missingInFilesystem -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "ARTIGOS NO REGISTRY SEM ARQUIVOS" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    $missingFiles | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingInFilesystem -gt 10) {
        Write-Host "... e mais $($missingInFilesystem - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($titleMismatches -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "TÍTULOS NÃO CORRESPONDEM" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "Estes artigos têm generated_title no registry diferente do title.txt no filesystem" -ForegroundColor Yellow
    Write-Host ""
    $mismatches | Select-Object -First 10 | Format-Table -AutoSize
    if ($titleMismatches -gt 10) {
        Write-Host "... e mais $($titleMismatches - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($missingInRegistry.Count -gt 0) {
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "ARTIGOS NO FILESYSTEM NÃO NO REGISTRY" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "Estes artigos existem no filesystem mas não estão no registry como Published" -ForegroundColor Yellow
    Write-Host ""
    $missingInRegistry | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingInRegistry.Count -gt 10) {
        Write-Host "... e mais $($missingInRegistry.Count - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

# Resumo final
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMO FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$percentComplete = if ($totalPublished -gt 0) { 
    [Math]::Round((($titleMatches + $missingInFilesystem) / $totalPublished) * 100, 2) 
} else { 0 }

Write-Host "Artigos consistentes: $titleMatches / $totalPublished ($percentComplete%)" -ForegroundColor $(if ($percentComplete -eq 100) { "Green" } else { "Yellow" })

if ($titleMismatches -eq 0 -and $missingInFilesystem -eq 0 -and $missingGeneratedTitle -eq 0 -and $missingInRegistry.Count -eq 0) {
    Write-Host ""
    Write-Host "✅ PERFEITO! Todos os artigos estão consistentes!" -ForegroundColor Green
    Write-Host "✅ Todos usam generated_title correto!" -ForegroundColor Green
    Write-Host "✅ Todos estão presentes no registry!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "⚠️  Problemas encontrados:" -ForegroundColor Yellow
    if ($titleMismatches -gt 0) {
        Write-Host "   - $titleMismatches artigos com títulos não correspondentes" -ForegroundColor Yellow
    }
    if ($missingInFilesystem -gt 0) {
        Write-Host "   - $missingInFilesystem artigos no registry sem arquivos" -ForegroundColor Yellow
    }
    if ($missingGeneratedTitle -gt 0) {
        Write-Host "   - $missingGeneratedTitle artigos sem generated_title" -ForegroundColor Yellow
    }
    if ($missingInRegistry.Count -gt 0) {
        Write-Host "   - $($missingInRegistry.Count) artigos no filesystem não no registry" -ForegroundColor Yellow
    }
}

Write-Host ""

