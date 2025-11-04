# Script para testar se featured e hidden estão sendo atualizados no registry

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$testArticleId = "2510.25013"  # ID de exemplo para teste

Write-Host "=== Teste de Featured e Hidden ===" -ForegroundColor Cyan
Write-Host "Registry: $registryPath" -ForegroundColor Gray
Write-Host "Artigo de teste: $testArticleId" -ForegroundColor Gray
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

# Encontrar o artigo de teste
$testArticle = $null
foreach ($prop in $articles) {
    if ($prop.Name -eq $testArticleId) {
        $testArticle = $prop.Value
        break
    }
}

if (-not $testArticle) {
    Write-Host "❌ Artigo de teste '$testArticleId' não encontrado!" -ForegroundColor Red
    Write-Host "Use o primeiro artigo Published encontrado..." -ForegroundColor Yellow
    
    # Procurar primeiro artigo Published
    foreach ($prop in $articles) {
        if ($prop.Value.status -eq "Published") {
            $testArticleId = $prop.Name
            $testArticle = $prop.Value
            Write-Host "Usando artigo: $testArticleId" -ForegroundColor Green
            break
        }
    }
}

if (-not $testArticle) {
    Write-Host "❌ Nenhum artigo Published encontrado!" -ForegroundColor Red
    exit 1
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "ESTADO ATUAL DO ARTIGO" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "ID: $testArticleId" -ForegroundColor White
Write-Host "Status: $($testArticle.status)" -ForegroundColor White
Write-Host "Featured: $($testArticle.featured)" -ForegroundColor $(if ($testArticle.featured -eq $true) { "Green" } else { "Gray" })
Write-Host "Hidden: $($testArticle.hidden)" -ForegroundColor $(if ($testArticle.hidden -eq $true) { "Red" } else { "Gray" })
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "INSTRUÇÕES PARA TESTE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "1. Abra o Dashboard em http://localhost:1420" -ForegroundColor White
Write-Host "2. Vá para a página de Logs" -ForegroundColor White
Write-Host "3. Encontre o artigo: $testArticleId" -ForegroundColor White
Write-Host "4. Marque/desmarque o Featured" -ForegroundColor White
Write-Host "5. Marque/desmarque o Hide/Show" -ForegroundColor White
Write-Host "6. Execute este script novamente para verificar as mudanças" -ForegroundColor White
Write-Host ""
Write-Host "Pressione Enter após fazer as mudanças no dashboard..." -ForegroundColor Yellow
Read-Host

# Recarregar registry após mudanças
Write-Host ""
Write-Host "Recarregando registry..." -ForegroundColor Cyan
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$testArticleAfter = $null
foreach ($prop in $articles) {
    if ($prop.Name -eq $testArticleId) {
        $testArticleAfter = $prop.Value
        break
    }
}

if (-not $testArticleAfter) {
    Write-Host "❌ Artigo não encontrado após recarregar!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "ESTADO APÓS AS MUDANÇAS" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "ID: $testArticleId" -ForegroundColor White
Write-Host "Status: $($testArticleAfter.status)" -ForegroundColor White
Write-Host "Featured: $($testArticleAfter.featured)" -ForegroundColor $(if ($testArticleAfter.featured -eq $true) { "Green" } else { "Gray" })
Write-Host "Hidden: $($testArticleAfter.hidden)" -ForegroundColor $(if ($testArticleAfter.hidden -eq $true) { "Red" } else { "Gray" })
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARAÇÃO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
$featuredChanged = $testArticle.featured -ne $testArticleAfter.featured
$hiddenChanged = $testArticle.hidden -ne $testArticleAfter.hidden

if ($featuredChanged) {
    Write-Host "✅ Featured mudou: $($testArticle.featured) -> $($testArticleAfter.featured)" -ForegroundColor Green
} else {
    Write-Host "⚠️  Featured não mudou: $($testArticle.featured)" -ForegroundColor Yellow
}

if ($hiddenChanged) {
    Write-Host "✅ Hidden mudou: $($testArticle.hidden) -> $($testArticleAfter.hidden)" -ForegroundColor Green
} else {
    Write-Host "⚠️  Hidden não mudou: $($testArticle.hidden)" -ForegroundColor Yellow
}

if ($featuredChanged -or $hiddenChanged) {
    Write-Host ""
    Write-Host "✅ SUCESSO! As mudanças foram refletidas no registry!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "❌ FALHA! As mudanças NÃO foram refletidas no registry!" -ForegroundColor Red
    Write-Host "Verifique:" -ForegroundColor Yellow
    Write-Host "  - Se o backend está rodando" -ForegroundColor Yellow
    Write-Host "  - Se o backend tem permissão para escrever no registry" -ForegroundColor Yellow
    Write-Host "  - Se ha erros no console do backend" -ForegroundColor Yellow
}

Write-Host ""

