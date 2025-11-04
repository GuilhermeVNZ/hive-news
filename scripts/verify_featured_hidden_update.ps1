# Script para verificar se featured e hidden estão sendo atualizados no registry

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"

Write-Host "=== Verificação de Featured e Hidden ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$totalPublished = 0
$hasFeatured = 0
$hasHidden = 0
$featuredTrue = 0
$featuredFalse = 0
$hiddenTrue = 0
$hiddenFalse = 0
$featuredNull = 0
$hiddenNull = 0

foreach ($prop in $articles) {
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    $totalPublished++
    
    # Verificar featured
    if ($article.featured -eq $true) {
        $hasFeatured++
        $featuredTrue++
    } elseif ($article.featured -eq $false) {
        $hasFeatured++
        $featuredFalse++
    } else {
        $featuredNull++
    }
    
    # Verificar hidden
    if ($article.hidden -eq $true) {
        $hasHidden++
        $hiddenTrue++
    } elseif ($article.hidden -eq $false) {
        $hasHidden++
        $hiddenFalse++
    } else {
        $hiddenNull++
    }
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "RESULTADO DA VERIFICAÇÃO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total de artigos Published: $totalPublished" -ForegroundColor White
Write-Host ""

Write-Host "Featured:" -ForegroundColor Cyan
Write-Host "  Total com featured definido: $hasFeatured" -ForegroundColor White
Write-Host "  Featured = true: $featuredTrue" -ForegroundColor Green
Write-Host "  Featured = false: $featuredFalse" -ForegroundColor Gray
Write-Host "  Featured = null: $featuredNull" -ForegroundColor $(if ($featuredNull -gt 0) { "Yellow" } else { "Green" })
Write-Host ""

Write-Host "Hidden:" -ForegroundColor Cyan
Write-Host "  Total com hidden definido: $hasHidden" -ForegroundColor White
Write-Host "  Hidden = true: $hiddenTrue" -ForegroundColor Red
Write-Host "  Hidden = false: $hiddenFalse" -ForegroundColor Gray
Write-Host "  Hidden = null: $hiddenNull" -ForegroundColor $(if ($hiddenNull -gt 0) { "Yellow" } else { "Green" })
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "FLUXO DE ATUALIZAÇÃO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "1. Dashboard faz PUT para:" -ForegroundColor White
Write-Host "   - /api/logs/articles/{id}/featured" -ForegroundColor Gray
Write-Host "   - /api/logs/articles/{id}/hidden" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Backend (logs.rs) processa:" -ForegroundColor White
Write-Host "   - set_featured() -> manager.set_featured()" -ForegroundColor Gray
Write-Host "   - set_hidden() -> manager.set_hidden()" -ForegroundColor Gray
Write-Host ""
Write-Host "3. RegistryManager (article_registry.rs):" -ForegroundColor White
Write-Host "   - set_featured() atualiza meta.featured = Some(featured)" -ForegroundColor Gray
Write-Host "   - set_hidden() atualiza meta.hidden = Some(hidden)" -ForegroundColor Gray
Write-Host "   - save() salva no arquivo JSON atomicamente" -ForegroundColor Gray
Write-Host ""

if ($featuredNull -eq 0 -and $hiddenNull -eq 0) {
    Write-Host "✅ Todos os artigos Published têm featured e hidden definidos!" -ForegroundColor Green
    Write-Host "✅ O sistema está funcionando corretamente!" -ForegroundColor Green
} else {
    Write-Host "⚠️  Alguns artigos têm featured ou hidden como null!" -ForegroundColor Yellow
    Write-Host "   Isso pode indicar que alguns artigos não foram atualizados ainda." -ForegroundColor Gray
}

Write-Host ""

