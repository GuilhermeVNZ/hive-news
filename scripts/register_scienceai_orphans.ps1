# Script para registrar noticias orfaos do ScienceAI no registry

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"

Write-Host "=== Registrando Noticias Orfaos do ScienceAI ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

# Criar mapa de IDs existentes para verificação rápida
$existingIds = @{}
foreach ($prop in $articles) {
    $existingIds[$prop.Name] = $true
}

$registered = 0
$skipped = 0
$errors = 0

# Função para extrair ID do nome da pasta
function Get-ArticleIdFromFolder {
    param([string]$FolderName)
    
    # Formato: YYYY-MM-DD_source_category_article_id
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 3) {
        # Última parte é o ID (hash)
        $id = $parts[-1]
        return $id
    }
    return $null
}

# Função para detectar source category do nome da pasta
function Get-SourceCategoryFromFolder {
    param([string]$FolderName)
    
    # Formato: YYYY-MM-DD_source_category_article_id
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 3) {
        # Segunda parte é o source category
        $sourceCategory = $parts[1]
        return $sourceCategory
    }
    return "unknown"
}

# Função para extrair data do nome da pasta
function Get-DateFromFolder {
    param([string]$FolderName)
    
    # Formato: YYYY-MM-DD_source_category_article_id
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 1) {
        $dateStr = $parts[0]
        if ($dateStr -match '^\d{4}-\d{2}-\d{2}$') {
            return $dateStr
        }
    }
    return $null
}

# Função para gerar hash da URL (similar ao Rust)
function Get-HashFromString {
    param([string]$Text)
    
    # Usar .NET GetHashCode() - não é exatamente igual ao Rust DefaultHasher, mas serve como aproximação
    # Para matching exato, precisaríamos do hash real usado no Rust
    $hash = $Text.GetHashCode()
    if ($hash -lt 0) {
        $hash = [Math]::Abs($hash)
    }
    return $hash.ToString()
}

# Verificar artigos órfãos no filesystem
Write-Host "=== Verificando Artigos Orfaos no Filesystem ===" -ForegroundColor Cyan

if (-not (Test-Path $scienceaiDir)) {
    Write-Host "ERRO: Diretorio ScienceAI nao encontrado: $scienceaiDir" -ForegroundColor Red
    exit 1
}

$folders = Get-ChildItem -Path $scienceaiDir -Directory -ErrorAction SilentlyContinue

foreach ($folder in $folders) {
    $folderName = $folder.Name
    $titleFile = Join-Path $folder.FullName "title.txt"
    $articleFile = Join-Path $folder.FullName "article.md"
    $sourceFile = Join-Path $folder.FullName "source.txt"
    $subtitleFile = Join-Path $folder.FullName "subtitle.txt"
    
    if (-not (Test-Path $titleFile) -or -not (Test-Path $articleFile)) {
        Write-Host "[SKIP] $folderName - arquivos essenciais nao encontrados" -ForegroundColor Gray
        $skipped++
        continue
    }
    
    # Extrair ID do nome da pasta
    $articleId = Get-ArticleIdFromFolder $folderName
    
    if (-not $articleId) {
        Write-Host "[SKIP] $folderName - nao foi possivel extrair ID" -ForegroundColor Gray
        $skipped++
        continue
    }
    
    # Verificar se já está no registry
    if ($existingIds.ContainsKey($articleId)) {
        Write-Host "[SKIP] $folderName - ja existe no registry (ID: $articleId)" -ForegroundColor Gray
        $skipped++
        continue
    }
    
    # Ler informações do filesystem
    $title = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
    $articleContent = (Get-Content $articleFile -Raw -Encoding UTF8).Trim()
    $sourceCategory = if (Test-Path $sourceFile) { 
        (Get-Content $sourceFile -Raw -Encoding UTF8).Trim() 
    } else { 
        Get-SourceCategoryFromFolder $folderName 
    }
    $subtitle = if (Test-Path $subtitleFile) { 
        (Get-Content $subtitleFile -Raw -Encoding UTF8).Trim() 
    } else { 
        "" 
    }
    
    # Extrair data da pasta
    $collectionDate = Get-DateFromFolder $folderName
    $publishedAt = if ($collectionDate) {
        try {
            $date = [DateTime]::ParseExact($collectionDate, "yyyy-MM-dd", $null)
            $date.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        } catch {
            (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        }
    } else {
        $folderInfo = Get-Item $folder.FullName
        $folderInfo.CreationTime.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    }
    
    # Construir URL baseada no source category (pode ser melhorado)
    $url = "https://example.com/news/$articleId"
    
    # Criar entrada no registry
    $articleMetadata = @{
        id = $articleId
        title = $title  # Mantido para compatibilidade
        original_title = $title  # Título original (mesmo que gerado, já que vem do title.txt)
        generated_title = $title  # Título gerado (do title.txt)
        arxiv_url = ""
        pdf_url = $url
        url = $url
        status = "Published"
        published_at = $publishedAt
        collected_at = $publishedAt
        output_dir = $folder.FullName
        destinations = @("scienceai")
        source_type = "news"
        category = $sourceCategory
        summary = $subtitle
        author = $null
        published_date = $publishedAt
        image_url = $null
        featured = $false
        hidden = $false
        content_html = $null
        content_text = $articleContent
        slug = $null
    }
    
    # Adicionar ao registry
    if (-not $registry.articles) {
        $registry | Add-Member -NotePropertyName "articles" -NotePropertyValue @{} -Force
    }
    
    $registry.articles | Add-Member -NotePropertyName $articleId -NotePropertyValue $articleMetadata -Force
    
    $registered++
    Write-Host "[$registered] OK Registrado: $folderName" -ForegroundColor Green
    Write-Host "   ID: $articleId" -ForegroundColor Gray
    Write-Host "   Titulo: $($title.Substring(0, [Math]::Min(60, $title.Length)))..." -ForegroundColor Gray
    Write-Host "   Source: $sourceCategory" -ForegroundColor Gray
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
Write-Host "Registrados: $registered" -ForegroundColor Green
Write-Host "Pulados (ja existiam): $skipped" -ForegroundColor Yellow
Write-Host "Erros: $errors" -ForegroundColor Red
Write-Host ""

if ($registered -gt 0) {
    Write-Host "OK Noticias orfaos registradas com sucesso!" -ForegroundColor Green
    Write-Host "Total de artigos no registry agora: $($registry.articles.PSObject.Properties.Count)" -ForegroundColor White
} else {
    Write-Host "AVISO Nenhuma noticia orfa encontrada ou todas ja estao registradas" -ForegroundColor Yellow
}

Write-Host ""

