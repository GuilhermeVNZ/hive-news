# Script para verificar se todos os artigos Published possuem id e generated_title

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"

Write-Host "=== Verificando Campos Obrigatórios ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$hasId = 0
$hasGeneratedTitle = 0
$hasBoth = 0
$missingId = @()
$missingGeneratedTitle = @()
$missingBoth = @()

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    
    # Verificar se tem id
    $hasIdField = $article.id -and $article.id -ne "" -and $article.id -ne $null
    # Verificar se tem generated_title
    $hasGenTitle = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if ($hasIdField) {
        $hasId++
    }
    
    if ($hasGenTitle) {
        $hasGeneratedTitle++
    }
    
    if ($hasIdField -and $hasGenTitle) {
        $hasBoth++
    } else {
        $titlePreview = if ($article.title) { 
            $article.title.Substring(0, [Math]::Min(60, $article.title.Length)) 
        } else { "N/A" }
        
        $info = [PSCustomObject]@{
            ID = $id
            HasId = $hasIdField
            HasGeneratedTitle = $hasGenTitle
            Title = $titlePreview
            RegistryId = $article.id
            GeneratedTitle = if ($hasGenTitle) { 
                $article.generated_title.Substring(0, [Math]::Min(50, $article.generated_title.Length)) 
            } else { "❌ FALTANDO" }
        }
        
        if (-not $hasIdField -and -not $hasGenTitle) {
            $missingBoth += $info
        } elseif (-not $hasIdField) {
            $missingId += $info
        } elseif (-not $hasGenTitle) {
            $missingGeneratedTitle += $info
        }
    }
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published: $totalPublished" -ForegroundColor White
Write-Host "✅ Artigos com id: $hasId" -ForegroundColor Green
Write-Host "✅ Artigos com generated_title: $hasGeneratedTitle" -ForegroundColor Green
Write-Host "✅ Artigos com ambos: $hasBoth" -ForegroundColor Green
Write-Host ""

if ($missingId.Count -gt 0) {
    Write-Host "❌ Artigos faltando id: $($missingId.Count)" -ForegroundColor Red
}
if ($missingGeneratedTitle.Count -gt 0) {
    Write-Host "⚠️  Artigos faltando generated_title: $($missingGeneratedTitle.Count)" -ForegroundColor Yellow
}
if ($missingBoth.Count -gt 0) {
    Write-Host "❌ Artigos faltando ambos: $($missingBoth.Count)" -ForegroundColor Red
}

if ($missingId.Count -eq 0 -and $missingGeneratedTitle.Count -eq 0 -and $missingBoth.Count -eq 0) {
    Write-Host "✅ PERFEITO! Todos os artigos Published possuem id e generated_title!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "DETALHES DOS ARTIGOS COM PROBLEMAS" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    
    if ($missingBoth.Count -gt 0) {
        Write-Host ""
        Write-Host "Artigos faltando AMBOS os campos:" -ForegroundColor Red
        $missingBoth | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingBoth.Count -gt 20) {
            Write-Host "... e mais $($missingBoth.Count - 20) artigos" -ForegroundColor Gray
        }
    }
    
    if ($missingId.Count -gt 0) {
        Write-Host ""
        Write-Host "Artigos faltando id:" -ForegroundColor Red
        $missingId | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingId.Count -gt 20) {
            Write-Host "... e mais $($missingId.Count - 20) artigos" -ForegroundColor Gray
        }
    }
    
    if ($missingGeneratedTitle.Count -gt 0) {
        Write-Host ""
        Write-Host "Artigos faltando generated_title:" -ForegroundColor Yellow
        $missingGeneratedTitle | Select-Object -First 20 | Format-Table -AutoSize
        if ($missingGeneratedTitle.Count -gt 20) {
            Write-Host "... e mais $($missingGeneratedTitle.Count - 20) artigos" -ForegroundColor Gray
        }
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMO FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
$percentComplete = if ($totalPublished -gt 0) { [Math]::Round(($hasBoth / $totalPublished) * 100, 2) } else { 0 }
Write-Host "Artigos Published completos: $hasBoth / $totalPublished ($percentComplete%)" -ForegroundColor $(if ($percentComplete -eq 100) { "Green" } else { "Yellow" })
Write-Host ""

