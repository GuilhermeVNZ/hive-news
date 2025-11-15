# Script para verificar destinations das notícias
$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$registry = Get-Content $registryPath -Raw | ConvertFrom-Json

$newsArticles = @()
foreach ($id in $registry.articles.PSObject.Properties.Name) {
    $article = $registry.articles.$id
    
    # Verificar se é news
    $isNews = $false
    if ($article.source_type -eq "rss" -or $article.source_type -eq "html") {
        $isNews = $true
    } elseif ($article.destinations -and $article.destinations -contains "scienceai") {
        if ($id -notmatch '^\d+\.\d+$') {
            $isNews = $true
        }
    }
    
    if ($isNews -and $article.status -eq "Published") {
        $newsArticles += [PSCustomObject]@{
            ID = $id
            Title = $article.title
            Destinations = if ($article.destinations) { ($article.destinations -join ", ") } else { "NONE" }
            HasDestinations = $null -ne $article.destinations -and $article.destinations.Count -gt 0
            HasScienceAI = if ($article.destinations) { $article.destinations -contains "scienceai" } else { $false }
            OutputDir = $article.output_dir
            Featured = $article.featured
            Hidden = $article.hidden
        }
    }
}

Write-Host "`n=== PUBLISHED NEWS - DESTINATIONS CHECK ===" -ForegroundColor Cyan
Write-Host "Total published news: $($newsArticles.Count)`n" -ForegroundColor Yellow

# Separar por destinations
$withDestinations = $newsArticles | Where-Object { $_.HasDestinations }
$withoutDestinations = $newsArticles | Where-Object { -not $_.HasDestinations }
$withScienceAI = $newsArticles | Where-Object { $_.HasScienceAI }
$withoutScienceAI = $newsArticles | Where-Object { -not $_.HasScienceAI }

Write-Host "With destinations: $($withDestinations.Count)" -ForegroundColor Green
Write-Host "Without destinations: $($withoutDestinations.Count)" -ForegroundColor Red
Write-Host "With ScienceAI destination: $($withScienceAI.Count)" -ForegroundColor Green
Write-Host "Without ScienceAI destination: $($withoutScienceAI.Count)" -ForegroundColor Red
Write-Host ""

if ($withoutDestinations.Count -gt 0) {
    Write-Host "⚠️  News articles WITHOUT destinations:" -ForegroundColor Yellow
    $withoutDestinations | Select-Object -First 10 ID, Title, Destinations | Format-Table -AutoSize
}

if ($withoutScienceAI.Count -gt 0) {
    Write-Host "`n⚠️  News articles WITHOUT ScienceAI destination:" -ForegroundColor Yellow
    $withoutScienceAI | Select-Object -First 10 ID, Title, Destinations | Format-Table -AutoSize
}

# Mostrar algumas notícias como exemplo
Write-Host "`n=== SAMPLE NEWS WITH DESTINATIONS ===" -ForegroundColor Cyan
$withScienceAI | Select-Object -First 5 ID, Title, Destinations, Featured, Hidden | Format-Table -AutoSize































