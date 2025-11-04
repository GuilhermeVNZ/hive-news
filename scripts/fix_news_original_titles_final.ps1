# Script para corrigir original_title das noticias
# Para noticias: original_title = titulo da fonte (RSS/web), generated_title = title.txt (DeepSeek)

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"
$rawDir = "G:\Hive-Hub\News-main\downloads\raw"

Write-Host "=== Corrigindo original_title das Noticias ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$updated = 0
$skipped = 0
$notFound = 0

# Criar mapa de JSONs originais para busca rapida
Write-Host "=== Carregando JSONs Originais ===" -ForegroundColor Cyan
$originalJsonMap = @{}

if (Test-Path $rawDir) {
    $dateFolders = Get-ChildItem -Path $rawDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($dateFolder in $dateFolders) {
        $jsonFiles = Get-ChildItem -Path $dateFolder.FullName -Filter "*.json" -ErrorAction SilentlyContinue
        
        foreach ($jsonFile in $jsonFiles) {
            try {
                $json = Get-Content $jsonFile.FullName -Raw -Encoding UTF8 | ConvertFrom-Json
                if ($json.id) {
                    $originalJsonMap[$json.id] = @{
                        title = $json.title
                        original_title = if ($json.original_title) { $json.original_title } else { $json.title }
                        url = $json.url
                    }
                }
            } catch {
                # Ignorar erros de parsing
            }
        }
    }
}

Write-Host "JSONs originais carregados: $($originalJsonMap.Count)" -ForegroundColor Gray
Write-Host ""

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
    
    # 1. generated_title: deve vir do title.txt na pasta de output (DeepSeek)
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
    
    # 2. original_title: deve vir do arquivo JSON original (RSS/web scraping)
    $originalTitleFromJson = $null
    
    if ($originalJsonMap.ContainsKey($id)) {
        $originalJsonMap[$id].original_title
        $originalTitleFromJson = $originalJsonMap[$id].original_title
    } elseif ($originalJsonMap.ContainsKey($id)) {
        $originalTitleFromJson = $originalJsonMap[$id].title
    }
    
    # Se nao encontrou no mapa, manter o original_title atual se for diferente do generated_title
    if (-not $originalTitleFromJson) {
        # Se original_title atual e diferente do generated_title, manter
        if ($newArticle["original_title"] -and $newArticle["generated_title"] -and 
            $newArticle["original_title"] -ne $newArticle["generated_title"]) {
            # Manter original_title atual
            $originalTitleFromJson = $newArticle["original_title"]
        } else {
            # Se nao tem original_title ou e igual ao generated_title, usar title como fallback
            if ($newArticle["title"] -and $newArticle["title"] -ne $newArticle["generated_title"]) {
                $originalTitleFromJson = $newArticle["title"]
            } else {
                # Se tudo e igual, usar o generated_title como original (caso extremo)
                $originalTitleFromJson = $newArticle["generated_title"]
            }
        }
    }
    
    if ($originalTitleFromJson) {
        # original_title deve ser o titulo original da fonte
        if (-not $newArticle["original_title"] -or $newArticle["original_title"] -ne $originalTitleFromJson) {
            $newArticle["original_title"] = $originalTitleFromJson
            $needsUpdate = $true
        }
    } else {
        $notFound++
    }
    
    # Atualizar objeto no registry
    if ($needsUpdate) {
        $registry.articles.$id = $newArticle
        $updated++
        
        $originalDisplay = if ($newArticle["original_title"]) { 
            $newArticle["original_title"].Substring(0, [Math]::Min(50, $newArticle["original_title"].Length)) 
        } else { 
            "N/A" 
        }
        $generatedDisplay = if ($newArticle["generated_title"]) { 
            $newArticle["generated_title"].Substring(0, [Math]::Min(50, $newArticle["generated_title"].Length)) 
        } else { 
            "N/A" 
        }
        
        Write-Host "[$updated] OK Atualizado: $id" -ForegroundColor Green
        Write-Host "   Original (fonte): $originalDisplay..." -ForegroundColor Gray
        Write-Host "   Generated (DeepSeek): $generatedDisplay..." -ForegroundColor Gray
        
        if ($originalTitleFromJson -eq $generatedDisplay) {
            Write-Host "   AVISO: Original e Generated sao iguais (pode ser correto)" -ForegroundColor Yellow
        }
    } else {
        $skipped++
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
Write-Host "Verificados: $($updated + $skipped)" -ForegroundColor White
Write-Host "Atualizados: $updated" -ForegroundColor Green
Write-Host "Pulados (ja corretos): $skipped" -ForegroundColor Yellow
Write-Host "JSON original nao encontrado: $notFound" -ForegroundColor Yellow
Write-Host ""
Write-Host "LOGICA CORRIGIDA:" -ForegroundColor Yellow
Write-Host "  - original_title: Titulo original da fonte (RSS/web scraping)" -ForegroundColor White
Write-Host "  - generated_title: Titulo gerado pelo DeepSeek (do title.txt)" -ForegroundColor White
Write-Host ""

