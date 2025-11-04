# Script para verificar artigos Published e comparar títulos

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"

Write-Host "=== Verificando Artigos Published ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host "Output: $outputDir" -ForegroundColor Gray
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$hasBothTitles = 0
$missingOriginal = 0
$missingGenerated = 0
$missingGeneratedButHasFile = @()
$missingGeneratedNoFile = @()
$sameTitles = @()
$differentTitles = 0
$details = @()

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    
    $hasOriginal = $article.original_title -and $article.original_title -ne "" -and $article.original_title -ne $null
    $hasGenerated = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if ($hasOriginal -and $hasGenerated) {
        $hasBothTitles++
        
        # Verificar se são diferentes
        $originalNorm = ($article.original_title -replace '\s+', ' ').Trim()
        $generatedNorm = ($article.generated_title -replace '\s+', ' ').Trim()
        
        if ($originalNorm -eq $generatedNorm) {
            $sameTitles += [PSCustomObject]@{
                ID = $id
                OriginalTitle = $article.original_title.Substring(0, [Math]::Min(60, $article.original_title.Length))
                GeneratedTitle = $article.generated_title.Substring(0, [Math]::Min(60, $article.generated_title.Length))
            }
        } else {
            $differentTitles++
        }
    } elseif (-not $hasOriginal) {
        $missingOriginal++
        $details += [PSCustomObject]@{
            ID = $id
            Status = $article.status
            Issue = "Missing original_title"
            Title = if ($article.title) { $article.title.Substring(0, [Math]::Min(60, $article.title.Length)) } else { "N/A" }
            OutputDir = $article.output_dir
        }
    } elseif (-not $hasGenerated) {
        $missingGenerated++
        
        # Verificar se existe title.txt no filesystem
        $hasTitleFile = $false
        $titleFileContent = $null
        
        if ($article.output_dir) {
            $outputDirPath = $article.output_dir -replace '\\', '/'
            $titleFilePath = Join-Path $outputDirPath "title.txt"
            
            if (Test-Path $titleFilePath) {
                $hasTitleFile = $true
                $titleFileContent = (Get-Content $titleFilePath -Raw -Encoding UTF8).Trim()
            } else {
                # Tentar buscar em AIResearch ou ScienceAI
                $arxivId = if ($id -match "(\d{4}\.\d{4,6})") { $matches[1] } else { $id }
                foreach ($siteDir in @("AIResearch", "ScienceAI")) {
                    $sitePath = Join-Path $outputDir $siteDir
                    if (Test-Path $sitePath) {
                        $folders = Get-ChildItem -Path $sitePath -Directory -ErrorAction SilentlyContinue | Where-Object { 
                            $_.Name -like "*$arxivId*" 
                        }
                        foreach ($folder in $folders) {
                            $titleFile = Join-Path $folder.FullName "title.txt"
                            if (Test-Path $titleFile) {
                                $hasTitleFile = $true
                                $titleFileContent = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                                break
                            }
                        }
                        if ($hasTitleFile) { break }
                    }
                }
            }
        }
        
        if ($hasTitleFile) {
            $missingGeneratedButHasFile += [PSCustomObject]@{
                ID = $id
                OutputDir = $article.output_dir
                TitleFileContent = $titleFileContent.Substring(0, [Math]::Min(60, $titleFileContent.Length))
                OriginalTitle = if ($article.original_title) { $article.original_title.Substring(0, [Math]::Min(60, $article.original_title.Length)) } else { "N/A" }
            }
        } else {
            $missingGeneratedNoFile += [PSCustomObject]@{
                ID = $id
                OutputDir = $article.output_dir
                OriginalTitle = if ($article.original_title) { $article.original_title.Substring(0, [Math]::Min(60, $article.original_title.Length)) } else { "N/A" }
            }
        }
    }
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published: $totalPublished" -ForegroundColor White
Write-Host "✅ Artigos com ambos os títulos: $hasBothTitles" -ForegroundColor Green
Write-Host ""

if ($missingOriginal -gt 0) {
    Write-Host "❌ Artigos faltando original_title: $missingOriginal" -ForegroundColor Red
}
if ($missingGenerated -gt 0) {
    Write-Host "⚠️  Artigos faltando generated_title: $missingGenerated" -ForegroundColor Yellow
    Write-Host "   - Com title.txt no filesystem: $($missingGeneratedButHasFile.Count)" -ForegroundColor Cyan
    Write-Host "   - Sem title.txt no filesystem: $($missingGeneratedNoFile.Count)" -ForegroundColor Red
}
Write-Host ""

if ($hasBothTitles -gt 0) {
    Write-Host "Comparação de títulos:" -ForegroundColor Cyan
    Write-Host "✅ Títulos diferentes: $differentTitles" -ForegroundColor Green
    if ($sameTitles.Count -gt 0) {
        Write-Host "⚠️  Títulos iguais (original = generated): $($sameTitles.Count)" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Detalhes dos problemas
if ($missingOriginal -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "ARTIGOS FALTANDO original_title:" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    $details | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingOriginal -gt 10) {
        Write-Host "... e mais $($missingOriginal - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($missingGeneratedButHasFile.Count -gt 0) {
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "ARTIGOS FALTANDO generated_title MAS COM title.txt:" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "Estes artigos têm title.txt no filesystem mas não têm generated_title no registry." -ForegroundColor Gray
    Write-Host "É necessário executar o backfill novamente ou atualizar o registry." -ForegroundColor Gray
    Write-Host ""
    $missingGeneratedButHasFile | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingGeneratedButHasFile.Count -gt 10) {
        Write-Host "... e mais $($missingGeneratedButHasFile.Count - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($missingGeneratedNoFile.Count -gt 0) {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "ARTIGOS FALTANDO generated_title E SEM title.txt:" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "Estes artigos estão marcados como Published mas não têm title.txt no filesystem." -ForegroundColor Gray
    Write-Host ""
    $missingGeneratedNoFile | Select-Object -First 10 | Format-Table -AutoSize
    if ($missingGeneratedNoFile.Count -gt 10) {
        Write-Host "... e mais $($missingGeneratedNoFile.Count - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($sameTitles.Count -gt 0) {
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "ARTIGOS COM TÍTULOS IGUAIS (original = generated):" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "Estes artigos têm original_title igual ao generated_title." -ForegroundColor Gray
    Write-Host "Isso pode indicar que o DeepSeek não gerou um novo título ou houve um problema." -ForegroundColor Gray
    Write-Host ""
    $sameTitles | Select-Object -First 10 | Format-Table -AutoSize
    if ($sameTitles.Count -gt 10) {
        Write-Host "... e mais $($sameTitles.Count - 10) artigos" -ForegroundColor Gray
    }
    Write-Host ""
}

# Resumo final
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMO FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
$percentComplete = if ($totalPublished -gt 0) { [Math]::Round(($hasBothTitles / $totalPublished) * 100, 2) } else { 0 }
Write-Host "Artigos Published completos: $hasBothTitles / $totalPublished ($percentComplete%)" -ForegroundColor $(if ($percentComplete -eq 100) { "Green" } else { "Yellow" })
Write-Host ""

