# Script para verificar se todos os artigos e news não rejeitados possuem original_title e generated_title

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
Write-Host "Lendo registry: $registryPath" -ForegroundColor Cyan

$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$total = 0
$nonRejected = 0
$missingBoth = @()
$missingOriginal = @()
$missingGenerated = @()
$hasBoth = 0

foreach ($prop in $articles) {
    $article = $prop.Value
    $total++
    
    # Pular artigos rejeitados
    if ($article.status -eq "Rejected") {
        continue
    }
    
    $nonRejected++
    
    # Verificar se possui original_title
    $hasOriginal = $article.original_title -and $article.original_title -ne "" -and $article.original_title -ne $null
    # Verificar se possui generated_title
    $hasGenerated = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if ($hasOriginal -and $hasGenerated) {
        $hasBoth++
    } else {
        $titlePreview = if ($article.title) { 
            $article.title.Substring(0, [Math]::Min(60, $article.title.Length)) 
        } else { "N/A" }
        
        $info = [PSCustomObject]@{
            ID = $article.id
            Status = $article.status
            Title = $titlePreview
            HasOriginal = $hasOriginal
            HasGenerated = $hasGenerated
            OriginalTitle = if ($article.original_title) { 
                $article.original_title.Substring(0, [Math]::Min(50, $article.original_title.Length)) 
            } else { "❌ FALTANDO" }
            GeneratedTitle = if ($article.generated_title) { 
                $article.generated_title.Substring(0, [Math]::Min(50, $article.generated_title.Length)) 
            } else { "❌ FALTANDO" }
        }
        
        if (-not $hasOriginal -and -not $hasGenerated) {
            $missingBoth += $info
        } elseif (-not $hasOriginal) {
            $missingOriginal += $info
        } elseif (-not $hasGenerated) {
            $missingGenerated += $info
        }
    }
}

Write-Host "`n========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos no registry: $total" -ForegroundColor White
Write-Host "Artigos não rejeitados: $nonRejected" -ForegroundColor White
Write-Host "✅ Artigos com ambos os títulos: $hasBoth" -ForegroundColor Green
Write-Host ""

if ($missingBoth.Count -gt 0) {
    Write-Host "❌ Artigos faltando AMBOS os títulos: $($missingBoth.Count)" -ForegroundColor Red
}
if ($missingOriginal.Count -gt 0) {
    Write-Host "⚠️  Artigos faltando original_title: $($missingOriginal.Count)" -ForegroundColor Yellow
}
if ($missingGenerated.Count -gt 0) {
    Write-Host "⚠️  Artigos faltando generated_title: $($missingGenerated.Count)" -ForegroundColor Yellow
}

if ($missingBoth.Count -eq 0 -and $missingOriginal.Count -eq 0 -and $missingGenerated.Count -eq 0) {
    Write-Host "`n✅ PERFEITO! Todos os artigos não rejeitados possuem ambos os títulos!" -ForegroundColor Green
} else {
    Write-Host "`n========================================" -ForegroundColor Yellow
    Write-Host "DETALHES DOS ARTIGOS COM PROBLEMAS" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    
    if ($missingBoth.Count -gt 0) {
        Write-Host "`nArtigos faltando AMBOS os títulos:" -ForegroundColor Red
        $missingBoth | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingBoth.Count -gt 20) {
            Write-Host "... e mais $($missingBoth.Count - 20) artigos" -ForegroundColor Gray
        }
    }
    
    if ($missingOriginal.Count -gt 0) {
        Write-Host "`nArtigos faltando original_title:" -ForegroundColor Yellow
        $missingOriginal | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingOriginal.Count -gt 20) {
            Write-Host "... e mais $($missingOriginal.Count - 20) artigos" -ForegroundColor Gray
        }
    }
    
    if ($missingGenerated.Count -gt 0) {
        Write-Host "`nArtigos faltando generated_title:" -ForegroundColor Yellow
        $missingGenerated | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingGenerated.Count -gt 20) {
            Write-Host "... e mais $($missingGenerated.Count - 20) artigos" -ForegroundColor Gray
        }
    }
}

