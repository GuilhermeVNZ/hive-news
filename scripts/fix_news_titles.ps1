# Script para corrigir original_title e generated_title das noticias
# Para noticias: original_title = titulo da fonte (RSS/web), generated_title = title.txt (DeepSeek)

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"
$rawDir = "G:\Hive-Hub\News-main\downloads\raw"

Write-Host "=== Corrigindo Titulos das Noticias ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$updated = 0
$skipped = 0
$notFound = 0

# Funcao para procurar arquivo JSON original na pasta raw
function Find-OriginalJson {
    param(
        [string]$ArticleId,
        [string]$RawDir
    )
    
    if (-not (Test-Path $RawDir)) {
        return $null
    }
    
    # Procurar em todas as subpastas de data
    $dateFolders = Get-ChildItem -Path $RawDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($dateFolder in $dateFolders) {
        $jsonFile = Join-Path $dateFolder.FullName "$ArticleId.json"
        if (Test-Path $jsonFile) {
            return $jsonFile
        }
    }
    
    return $null
}

# Verificar todas as noticias no registry
Write-Host "=== Verificando Noticias no Registry ===" -ForegroundColor Cyan

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Identificar noticias (source_type = "news" ou ID nao e arXiv nem PMC)
    $isNews = $false
    if ($article.PSObject.Properties.Name -contains "source_type" -and $article.source_type -eq "news") {
        $isNews = $true
    } elseif ($id -notmatch "^\d{4}\.\d{4,6}" -and $id -notmatch "^PMC") {
        # Se nao e arXiv nem PMC, provavelmente e noticia
        $isNews = $true
    }
    
    if (-not $isNews) {
        continue
    }
    
    $needsUpdate = $false
    $newArticle = @{}
    $article.PSObject.Properties | ForEach-Object { $newArticle[$_.Name] = $_.Value }
    
    # Garantir que campos existam
    if (-not $newArticle.ContainsKey("original_title")) {
        $newArticle["original_title"] = $null
    }
    if (-not $newArticle.ContainsKey("generated_title")) {
        $newArticle["generated_title"] = $null
    }
    
    # 1. generated_title: deve vir do title.txt na pasta de output
    $titleFromFile = $null
    if ($article.output_dir -and (Test-Path $article.output_dir)) {
        $titleFile = Join-Path $article.output_dir "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
        }
    }
    
    if ($titleFromFile) {
        # generated_title deve ser o title.txt (titulo gerado pelo DeepSeek)
        if (-not $newArticle["generated_title"] -or $newArticle["generated_title"] -ne $titleFromFile) {
            $newArticle["generated_title"] = $titleFromFile
            $needsUpdate = $true
        }
    }
    
    # 2. original_title: deve vir do arquivo JSON original na pasta raw
    $originalTitleFromJson = $null
    $originalJsonFile = Find-OriginalJson $id $rawDir
    
    if ($originalJsonFile -and (Test-Path $originalJsonFile)) {
        try {
            $originalJson = Get-Content $originalJsonFile -Raw -Encoding UTF8 | ConvertFrom-Json
            
            # Tentar obter original_title do JSON
            if ($originalJson.original_title) {
                $originalTitleFromJson = $originalJson.original_title
            } elseif ($originalJson.title) {
                $originalTitleFromJson = $originalJson.title
            }
        } catch {
            Write-Host "[AVISO] Erro ao ler JSON original para $id : $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }
    
    # Se nao encontrou no JSON, tentar usar o title do registry (pode ser o original)
    if (-not $originalTitleFromJson) {
        # Se o title do registry e diferente do generated_title, provavelmente e o original
        if ($newArticle["title"] -and $newArticle["generated_title"] -and $newArticle["title"] -ne $newArticle["generated_title"]) {
            $originalTitleFromJson = $newArticle["title"]
        } elseif ($newArticle["title"]) {
            # Se nao tem generated_title ainda, usar title como fallback
            $originalTitleFromJson = $newArticle["title"]
        }
    }
    
    if ($originalTitleFromJson) {
        # original_title deve ser o titulo original da fonte
        if (-not $newArticle["original_title"] -or $newArticle["original_title"] -ne $originalTitleFromJson) {
            $newArticle["original_title"] = $originalTitleFromJson
            $needsUpdate = $true
        }
    }
    
    # Atualizar objeto no registry
    if ($needsUpdate) {
        $registry.articles.$id = $newArticle
        $updated++
        
        $originalDisplay = if ($newArticle["original_title"]) { 
            $newArticle["original_title"].Substring(0, [Math]::Min(40, $newArticle["original_title"].Length)) 
        } else { 
            "N/A" 
        }
        $generatedDisplay = if ($newArticle["generated_title"]) { 
            $newArticle["generated_title"].Substring(0, [Math]::Min(40, $newArticle["generated_title"].Length)) 
        } else { 
            "N/A" 
        }
        
        Write-Host "[$updated] OK Atualizado: $id" -ForegroundColor Green
        Write-Host "   Original: $originalDisplay..." -ForegroundColor Gray
        Write-Host "   Generated: $generatedDisplay..." -ForegroundColor Gray
    } else {
        $skipped++
        Write-Host "[$updated] SKIP (ja correto): $id" -ForegroundColor Gray
    }
}

# Salvar registry atualizado
Write-Host ""
Write-Host "=== Salvando Registry Atualizado ===" -ForegroundColor Cyan
try {
    $json = $registry | ConvertTo-Json -Depth 100
    $json | Set-Content $registryPath -Encoding UTF8
    Write-Host "OK Registry salvo com sucesso!" -ForegroundColor Green
} catch {
    Write-Host "ERRO ao salvar registry: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Resumo
Write-Host ""
Write-Host "=== Resumo ===" -ForegroundColor Cyan
Write-Host "Verificados: $updated" -ForegroundColor White
Write-Host "Atualizados: $updated" -ForegroundColor Green
Write-Host "Pulados (ja corretos): $skipped" -ForegroundColor Yellow
Write-Host ""
Write-Host "LOGICA CORRIGIDA:" -ForegroundColor Yellow
Write-Host "  - original_title: Titulo original da fonte (RSS/web scraping)" -ForegroundColor White
Write-Host "  - generated_title: Titulo gerado pelo DeepSeek (do title.txt)" -ForegroundColor White
Write-Host ""

