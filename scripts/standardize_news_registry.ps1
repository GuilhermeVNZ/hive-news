# Script para padronizar noticias no registry com todos os campos necessarios

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
$errors = 0

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

function Get-DateFromFolder {
    param([string]$FolderName)
    $parts = $FolderName -split '_'
    if ($parts.Length -ge 1) {
        $dateStr = $parts[0]
        if ($dateStr -match '^\d{4}-\d{2}-\d{2}$') {
            return $dateStr
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
    if ($article.source_type -eq "news") {
        $isNews = $true
    } elseif ($id -notmatch "^\d{4}\.\d{4,6}" -and $id -notmatch "^PMC") {
        # Se nao e arXiv nem PMC, provavelmente e noticia
        $isNews = $true
    }
    
    if (-not $isNews) {
        continue
    }
    
    $updated++
    
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
    
    # Padronizar campos
    $needsUpdate = $false
    
    # Garantir que todos os campos existam antes de usar
    if (-not $article.PSObject.Properties.Name -contains "original_title") {
        $article | Add-Member -NotePropertyName "original_title" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "generated_title") {
        $article | Add-Member -NotePropertyName "generated_title" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "source_type") {
        $article | Add-Member -NotePropertyName "source_type" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "category") {
        $article | Add-Member -NotePropertyName "category" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "summary") {
        $article | Add-Member -NotePropertyName "summary" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "content_text") {
        $article | Add-Member -NotePropertyName "content_text" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "slug") {
        $article | Add-Member -NotePropertyName "slug" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "published_date") {
        $article | Add-Member -NotePropertyName "published_date" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "author") {
        $article | Add-Member -NotePropertyName "author" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "image_url") {
        $article | Add-Member -NotePropertyName "image_url" -NotePropertyValue $null -Force
    }
    if (-not $article.PSObject.Properties.Name -contains "content_html") {
        $article | Add-Member -NotePropertyName "content_html" -NotePropertyValue $null -Force
    }
    
    # original_title: usar o que esta no registry ou title se nao tiver
    if (-not $article.original_title -or $article.original_title -eq "") {
        if ($article.url -and $article.url -ne "https://example.com/news/$id") {
            # Se tem URL real, tentar usar title do registry
            $article.original_title = $article.title
        } else {
            # Se nao tem URL real, usar generated_title se disponivel
            if ($article.generated_title) {
                $article.original_title = $article.generated_title
            } else {
                $article.original_title = $article.title
            }
        }
        $needsUpdate = $true
    }
    
    # generated_title: usar title.txt se disponivel
    if ($titleFromFile) {
        if (-not $article.generated_title -or $article.generated_title -ne $titleFromFile) {
            $article.generated_title = $titleFromFile
            $needsUpdate = $true
        }
    }
    
    # source_type: garantir que e "news"
    if (-not $article.source_type -or $article.source_type -ne "news") {
        $article.source_type = "news"
        $needsUpdate = $true
    }
    
    # category: usar source category
    if ($sourceFromFile) {
        if (-not $article.category -or $article.category -ne $sourceFromFile) {
            $article.category = $sourceFromFile
            $needsUpdate = $true
        }
    } elseif ($sourceCategoryFromFile) {
        if (-not $article.category -or $article.category -ne $sourceCategoryFromFile) {
            $article.category = $sourceCategoryFromFile
            $needsUpdate = $true
        }
    }
    
    # url: se e placeholder, tentar manter ou melhorar
    if ($article.url -eq "https://example.com/news/$id" -or -not $article.url) {
        # Manter placeholder por enquanto (sera preenchido quando tiver URL real)
        # Nao atualizar para evitar perder URLs reais
    }
    
    # summary: usar subtitle se disponivel
    if ($subtitleFromFile) {
        if (-not $article.summary -or $article.summary -ne $subtitleFromFile) {
            $article.summary = $subtitleFromFile
            $needsUpdate = $true
        }
    }
    
    # content_text: usar article.md se disponivel
    if ($articleContentFromFile) {
        if (-not $article.content_text -or $article.content_text -ne $articleContentFromFile) {
            $article.content_text = $articleContentFromFile
            $needsUpdate = $true
        }
    }
    
    # slug: usar slug.txt se disponivel
    if ($slugFromFile) {
        if (-not $article.slug -or $article.slug -ne $slugFromFile) {
            $article.slug = $slugFromFile
            $needsUpdate = $true
        }
    }
    
    # published_date: usar published_at se disponivel
    if ($article.published_at -and -not $article.published_date) {
        $article.published_date = $article.published_at
        $needsUpdate = $true
    }
    
    if ($needsUpdate) {
        $titleDisplay = if ($article.generated_title) { 
            $article.generated_title.Substring(0, [Math]::Min(50, $article.generated_title.Length)) 
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

