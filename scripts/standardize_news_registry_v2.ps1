# Script para padronizar noticias no registry com todos os campos necessarios (versao corrigida)

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"

Write-Host "=== Padronizando Noticias no Registry ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$updated = 0
$skipped = 0

# Funcoes auxiliares
function Get-ArticleIdFromFolder {
    param([string]$FolderName)
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 3) {
        return $parts[-1]
    }
    return $null
}

function Get-SourceCategoryFromFolder {
    param([string]$FolderName)
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 3) {
        return $parts[1]
    }
    return "unknown"
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
    
    $updated++
    $needsUpdate = $false
    
    # Ler informacoes do filesystem se disponivel
    $titleFromFile = $null
    $subtitleFromFile = $null
    $articleContentFromFile = $null
    $sourceCategoryFromFile = $null
    $slugFromFile = $null
    $sourceFromFile = $null
    
    if ($article.output_dir -and (Test-Path $article.output_dir)) {
        $outputDirPath = $article.output_dir
        
        # Ler title.txt
        $titleFile = Join-Path $outputDirPath "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
        }
        
        # Ler subtitle.txt
        $subtitleFile = Join-Path $outputDirPath "subtitle.txt"
        if (Test-Path $subtitleFile) {
            $subtitleFromFile = (Get-Content $subtitleFile -Raw -Encoding UTF8).Trim()
        }
        
        # Ler article.md
        $articleFile = Join-Path $outputDirPath "article.md"
        if (Test-Path $articleFile) {
            $articleContentFromFile = (Get-Content $articleFile -Raw -Encoding UTF8).Trim()
        }
        
        # Ler source.txt
        $sourceFile = Join-Path $outputDirPath "source.txt"
        if (Test-Path $sourceFile) {
            $sourceFromFile = (Get-Content $sourceFile -Raw -Encoding UTF8).Trim()
        }
        
        # Ler image_categories.txt (primeira linha e source category)
        $categoriesFile = Join-Path $outputDirPath "image_categories.txt"
        if (Test-Path $categoriesFile) {
            $categories = Get-Content $categoriesFile -Raw -Encoding UTF8
            $categoriesLines = $categories -split "`n" | Where-Object { $_.Trim() }
            if ($categoriesLines.Count -gt 0) {
                $sourceCategoryFromFile = $categoriesLines[0].Trim().ToLower()
            }
        }
        
        # Ler slug.txt
        $slugFile = Join-Path $outputDirPath "slug.txt"
        if (Test-Path $slugFile) {
            $slugFromFile = (Get-Content $slugFile -Raw -Encoding UTF8).Trim()
        }
        
        # Se nao tem source.txt, extrair do nome da pasta
        if (-not $sourceFromFile) {
            $folderName = Split-Path $outputDirPath -Leaf
            $sourceFromFile = Get-SourceCategoryFromFolder $folderName
        }
    }
    
    # Criar novo objeto com todos os campos padronizados
    $newArticle = @{}
    $article.PSObject.Properties | ForEach-Object { $newArticle[$_.Name] = $_.Value }
    
    # Padronizar campos
    # original_title: usar o que esta no registry ou title se nao tiver
    if (-not $newArticle.ContainsKey("original_title") -or -not $newArticle["original_title"] -or $newArticle["original_title"] -eq "") {
        if ($newArticle["url"] -and $newArticle["url"] -ne "https://example.com/news/$id") {
            $newArticle["original_title"] = $newArticle["title"]
        } else {
            if ($newArticle.ContainsKey("generated_title") -and $newArticle["generated_title"]) {
                $newArticle["original_title"] = $newArticle["generated_title"]
            } else {
                $newArticle["original_title"] = $newArticle["title"]
            }
        }
        $needsUpdate = $true
    }
    
    # generated_title: usar title.txt se disponivel
    if ($titleFromFile) {
        if (-not $newArticle.ContainsKey("generated_title") -or $newArticle["generated_title"] -ne $titleFromFile) {
            $newArticle["generated_title"] = $titleFromFile
            $needsUpdate = $true
        }
    }
    
    # source_type: garantir que e "news"
    if (-not $newArticle.ContainsKey("source_type") -or $newArticle["source_type"] -ne "news") {
        $newArticle["source_type"] = "news"
        $needsUpdate = $true
    }
    
    # category: usar source category
    if ($sourceFromFile) {
        if (-not $newArticle.ContainsKey("category") -or $newArticle["category"] -ne $sourceFromFile) {
            $newArticle["category"] = $sourceFromFile
            $needsUpdate = $true
        }
    } elseif ($sourceCategoryFromFile) {
        if (-not $newArticle.ContainsKey("category") -or $newArticle["category"] -ne $sourceCategoryFromFile) {
            $newArticle["category"] = $sourceCategoryFromFile
            $needsUpdate = $true
        }
    }
    
    # summary: usar subtitle se disponivel
    if ($subtitleFromFile) {
        if (-not $newArticle.ContainsKey("summary") -or $newArticle["summary"] -ne $subtitleFromFile) {
            $newArticle["summary"] = $subtitleFromFile
            $needsUpdate = $true
        }
    }
    
    # content_text: usar article.md se disponivel
    if ($articleContentFromFile) {
        if (-not $newArticle.ContainsKey("content_text") -or $newArticle["content_text"] -ne $articleContentFromFile) {
            $newArticle["content_text"] = $articleContentFromFile
            $needsUpdate = $true
        }
    }
    
    # slug: usar slug.txt se disponivel
    if ($slugFromFile) {
        if (-not $newArticle.ContainsKey("slug") -or $newArticle["slug"] -ne $slugFromFile) {
            $newArticle["slug"] = $slugFromFile
            $needsUpdate = $true
        }
    }
    
    # published_date: usar published_at se disponivel
    if ($newArticle.ContainsKey("published_at") -and $newArticle["published_at"] -and -not $newArticle.ContainsKey("published_date")) {
        $newArticle["published_date"] = $newArticle["published_at"]
        $needsUpdate = $true
    }
    
    # Garantir que todos os campos opcionais existam (mesmo que null)
    $requiredFields = @("author", "image_url", "content_html", "url")
    foreach ($field in $requiredFields) {
        if (-not $newArticle.ContainsKey($field)) {
            $newArticle[$field] = $null
            $needsUpdate = $true
        }
    }
    
    # Atualizar objeto no registry
    if ($needsUpdate) {
        $registry.articles.$id = $newArticle
        $titleDisplay = if ($newArticle["generated_title"]) { 
            $newArticle["generated_title"].Substring(0, [Math]::Min(50, $newArticle["generated_title"].Length)) 
        } else { 
            "N/A" 
        }
        Write-Host "[$updated] OK Atualizado: $id" -ForegroundColor Green
        Write-Host "   Titulo: $titleDisplay..." -ForegroundColor Gray
    } else {
        $skipped++
        Write-Host "[$updated] SKIP (ja padronizado): $id" -ForegroundColor Gray
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
Write-Host "Atualizados: $($updated - $skipped)" -ForegroundColor Green
Write-Host "Pulados (ja padronizados): $skipped" -ForegroundColor Yellow
Write-Host ""

