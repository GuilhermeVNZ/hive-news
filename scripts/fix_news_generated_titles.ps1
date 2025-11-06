# Script para corrigir generated_title das notícias
$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$registry = Get-Content $registryPath -Raw | ConvertFrom-Json

$fixed = 0
$notFound = 0
$fixedIds = @()

foreach ($id in $registry.articles.PSObject.Properties.Name) {
    $article = $registry.articles.$id
    
    # Verificar se é news Published sem generated_title
    $isNews = $false
    if ($article.source_type -eq "rss" -or $article.source_type -eq "html") {
        $isNews = $true
    } elseif ($article.destinations -and $article.destinations -contains "scienceai") {
        if ($id -notmatch '^\d+\.\d+$') {
            $isNews = $true
        }
    }
    
    if ($isNews -and $article.status -eq "Published" -and (-not $article.generated_title)) {
        if ($article.output_dir) {
            $titlePath = Join-Path $article.output_dir "title.txt"
            
            if (Test-Path $titlePath) {
                $generatedTitle = Get-Content $titlePath -Raw | ForEach-Object { $_.Trim() }
                
                if ($generatedTitle) {
                    $article.generated_title = $generatedTitle
                    $fixed++
                    $fixedIds += $id
                    Write-Host "✅ Fixed: $id - $($generatedTitle.Substring(0, [Math]::Min(50, $generatedTitle.Length)))..." -ForegroundColor Green
                } else {
                    Write-Host "⚠️  Empty title.txt: $id" -ForegroundColor Yellow
                    $notFound++
                }
            } else {
                Write-Host "❌ title.txt not found: $id ($titlePath)" -ForegroundColor Red
                $notFound++
            }
        } else {
            Write-Host "⚠️  No output_dir: $id" -ForegroundColor Yellow
            $notFound++
        }
    }
}

if ($fixed -gt 0) {
    Write-Host "`n💾 Saving registry..." -ForegroundColor Cyan
    $registryJson = $registry | ConvertTo-Json -Depth 100
    [System.IO.File]::WriteAllText($registryPath, $registryJson, [System.Text.Encoding]::UTF8)
    Write-Host "✅ Registry saved! Fixed $fixed news articles." -ForegroundColor Green
} else {
    Write-Host "`n⚠️  No news articles to fix." -ForegroundColor Yellow
}

if ($notFound -gt 0) {
    Write-Host "⚠️  $notFound news articles could not be fixed (missing title.txt or output_dir)" -ForegroundColor Yellow
}

Write-Host "`n=== SUMMARY ===" -ForegroundColor Cyan
Write-Host "Fixed: $fixed" -ForegroundColor Green
Write-Host "Not Found: $notFound" -ForegroundColor Yellow








