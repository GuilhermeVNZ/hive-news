# Script para testar se featured e hidden são atualizados no registry

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"

Write-Host "=== Teste de Atualização Featured/Hidden ===" -ForegroundColor Cyan
Write-Host ""

# Pegar um artigo Published para teste
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$testArticle = $null
$testArticleId = $null

foreach ($prop in $articles) {
    if ($prop.Value.status -eq "Published") {
        $testArticleId = $prop.Name
        $testArticle = $prop.Value
        break
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
Write-Host "Título: $($testArticle.title.Substring(0, [Math]::Min(60, $testArticle.title.Length)))..." -ForegroundColor Gray
Write-Host "Featured ANTES: $($testArticle.featured)" -ForegroundColor $(if ($testArticle.featured -eq $true) { "Green" } else { "Gray" })
Write-Host "Hidden ANTES: $($testArticle.hidden)" -ForegroundColor $(if ($testArticle.hidden -eq $true) { "Red" } else { "Gray" })
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "INSTRUÇÕES PARA TESTE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "1. Abra o Dashboard em http://localhost:1420/logs" -ForegroundColor White
Write-Host "2. Encontre o artigo: $testArticleId" -ForegroundColor White
Write-Host "3. Marque/desmarque o checkbox 'Featured'" -ForegroundColor White
Write-Host "4. Marque/desmarque o botão 'Hide/Show'" -ForegroundColor White
Write-Host "5. Aguarde alguns segundos" -ForegroundColor White
Write-Host "6. Execute este script novamente para verificar" -ForegroundColor White
Write-Host ""
Write-Host "Pressione Enter após fazer as mudanças no dashboard..." -ForegroundColor Yellow
Read-Host

# Recarregar registry
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
Write-Host "Featured DEPOIS: $($testArticleAfter.featured)" -ForegroundColor $(if ($testArticleAfter.featured -eq $true) { "Green" } else { "Gray" })
Write-Host "Hidden DEPOIS: $($testArticleAfter.hidden)" -ForegroundColor $(if ($testArticleAfter.hidden -eq $true) { "Red" } else { "Gray" })
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESULTADO DO TESTE" -ForegroundColor Cyan
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
    Write-Host "✅ O sistema está funcionando corretamente!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "❌ FALHA! As mudanças NÃO foram refletidas no registry!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Verifique:" -ForegroundColor Yellow
    Write-Host "  - Se o backend está rodando" -ForegroundColor Yellow
    Write-Host "  - Se o backend tem permissão para escrever no registry" -ForegroundColor Yellow
    Write-Host "  - Se ha erros no console do backend" -ForegroundColor Yellow
    Write-Host "  - Se a chamada API foi bem-sucedida (verifique o Network tab no navegador)" -ForegroundColor Yellow
}

Write-Host ""

