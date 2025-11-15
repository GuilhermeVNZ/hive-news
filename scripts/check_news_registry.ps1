# Script para verificar notícias no registry
$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$registry = Get-Content $registryPath | ConvertFrom-Json

$newsArticles = @()
foreach ($id in $registry.articles.PSObject.Properties.Name) {
    $article = $registry.articles.$id
    
    # Verificar se é news (source_type rss/html OU destinations contém scienceai E ID não é arXiv format)
    $isNews = $false
    if ($article.source_type -eq "rss" -or $article.source_type -eq "html") {
        $isNews = $true
    } elseif ($article.destinations -and $article.destinations -contains "scienceai") {
        if ($id -notmatch '^\d+\.\d+$') {
            $isNews = $true
        }
    }
    
    if ($isNews) {
        $newsArticles += [PSCustomObject]@{
            ID = $id
            Status = $article.status
            Title = $article.title
            OriginalTitle = $article.original_title
            GeneratedTitle = $article.generated_title
            SourceType = $article.source_type
            Destinations = ($article.destinations -join ", ")
            Featured = $article.featured
            Hidden = $article.hidden
            OutputDir = $article.output_dir
            PublishedAt = $article.published_at
            CollectedAt = $article.collected_at
        }
    }
}

Write-Host "`n=== NEWS ARTICLES IN REGISTRY ===" -ForegroundColor Cyan
Write-Host "Total news articles: $($newsArticles.Count)`n" -ForegroundColor Yellow

# Separar por status
$published = $newsArticles | Where-Object { $_.Status -eq "Published" }
$collected = $newsArticles | Where-Object { $_.Status -eq "Collected" }
$filtered = $newsArticles | Where-Object { $_.Status -eq "Filtered" }
$rejected = $newsArticles | Where-Object { $_.Status -eq "Rejected" }

Write-Host "Published: $($published.Count)" -ForegroundColor Green
Write-Host "Collected: $($collected.Count)" -ForegroundColor Yellow
Write-Host "Filtered: $($filtered.Count)" -ForegroundColor Yellow
Write-Host "Rejected: $($rejected.Count)" -ForegroundColor Red
Write-Host ""

# Verificar campos obrigatórios para Published
Write-Host "`n=== PUBLISHED NEWS - FIELD CHECK ===" -ForegroundColor Cyan
$missingFields = @()

foreach ($article in $published) {
    $missing = @()
    
    if (-not $article.Featured -and $article.Featured -ne $false) {
        $missing += "featured"
    }
    
    if (-not $article.Hidden -and $article.Hidden -ne $false) {
        $missing += "hidden"
    }
    
    if (-not $article.OutputDir) {
        $missing += "output_dir"
    }
    
    if (-not $article.Destinations) {
        $missing += "destinations"
    }
    
    if (-not $article.GeneratedTitle) {
        $missing += "generated_title"
    }
    
    if ($missing.Count -gt 0) {
        $missingFields += [PSCustomObject]@{
            ID = $article.ID
            MissingFields = ($missing -join ", ")
            Featured = $article.Featured
            Hidden = $article.Hidden
            OutputDir = $article.OutputDir
            Destinations = $article.Destinations
        }
    }
}

if ($missingFields.Count -eq 0) {
    Write-Host "✅ All published news articles have required fields (featured, hidden, output_dir, destinations)" -ForegroundColor Green
} else {
    Write-Host "⚠️  Found $($missingFields.Count) published news articles with missing fields:" -ForegroundColor Yellow
    $missingFields | Format-Table -AutoSize
}

# Verificar se output_dir existe
Write-Host "`n=== OUTPUT DIR CHECK ===" -ForegroundColor Cyan
$missingDirs = @()

foreach ($article in $published) {
    if ($article.OutputDir) {
        $outputPath = $article.OutputDir
        if (-not (Test-Path $outputPath)) {
            $missingDirs += [PSCustomObject]@{
                ID = $article.ID
                OutputDir = $outputPath
                Exists = $false
            }
        }
    }
}

if ($missingDirs.Count -eq 0) {
    Write-Host "✅ All published news articles have valid output directories" -ForegroundColor Green
} else {
    Write-Host "⚠️  Found $($missingDirs.Count) published news articles with missing output directories:" -ForegroundColor Yellow
    $missingDirs | Format-Table -AutoSize
}

# Mostrar algumas notícias publicadas como exemplo
Write-Host "`n=== SAMPLE PUBLISHED NEWS ===" -ForegroundColor Cyan
$published | Select-Object -First 5 ID, Status, Title, Featured, Hidden, Destinations, OutputDir | Format-Table -AutoSize


































