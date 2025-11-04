# Script para adicionar campo generated_title para todos os artigos que nao tem

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"
$scienceaiDir = Join-Path $outputDir "ScienceAI"
$airesearchDir = Join-Path $outputDir "AIResearch"

Write-Host "=== Adicionando Campo generated_title ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$added = 0
$skipped = 0

# Funcao para procurar title.txt no filesystem
function Find-TitleFile {
    param(
        [string]$ArticleId,
        [string]$OutputDir
    )
    
    if (-not (Test-Path $OutputDir)) {
        return $null
    }
    
    # Procurar em todas as pastas
    $folders = Get-ChildItem -Path $OutputDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $folders) {
        $folderName = $folder.Name
        
        # Tentar matching por ID completo
        if ($folderName -like "*$ArticleId*") {
            $titleFile = Join-Path $folder.FullName "title.txt"
            if (Test-Path $titleFile) {
                $title = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                if ($title) {
                    return $title
                }
            }
        }
    }
    
    return $null
}

# Verificar todos os artigos no registry
Write-Host "=== Verificando Artigos no Registry ===" -ForegroundColor Cyan

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Verificar se tem campo generated_title
    $hasGeneratedTitle = $article.PSObject.Properties.Name -contains "generated_title"
    
    if ($hasGeneratedTitle) {
        $skipped++
        continue
    }
    
    # Adicionar campo generated_title
    $newArticle = @{}
    $article.PSObject.Properties | ForEach-Object { $newArticle[$_.Name] = $_.Value }
    
    # Tentar buscar title.txt no filesystem
    $titleFromFile = $null
    
    # Verificar output_dir se disponivel
    if ($article.output_dir -and (Test-Path $article.output_dir)) {
        $titleFile = Join-Path $article.output_dir "title.txt"
        if (Test-Path $titleFile) {
            $titleFromFile = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
        }
    }
    
    # Se nao encontrou no output_dir, procurar em ScienceAI e AIResearch
    if (-not $titleFromFile) {
        $titleFromFile = Find-TitleFile $id $scienceaiDir
        if (-not $titleFromFile) {
            $titleFromFile = Find-TitleFile $id $airesearchDir
        }
    }
    
    # Se encontrou title.txt, usar como generated_title
    if ($titleFromFile) {
        $newArticle["generated_title"] = $titleFromFile
    } else {
        # Se nao encontrou, usar original_title ou title como fallback
        if ($newArticle.ContainsKey("original_title") -and $newArticle["original_title"]) {
            $newArticle["generated_title"] = $newArticle["original_title"]
        } elseif ($newArticle.ContainsKey("title") -and $newArticle["title"]) {
            $newArticle["generated_title"] = $newArticle["title"]
        } else {
            $newArticle["generated_title"] = $null
        }
    }
    
    # Atualizar objeto no registry
    $registry.articles.$id = $newArticle
    $added++
    
    $titleDisplay = if ($newArticle["generated_title"]) { 
        $newArticle["generated_title"].Substring(0, [Math]::Min(50, $newArticle["generated_title"].Length)) 
    } else { 
        "N/A (nao encontrado)" 
    }
    
    Write-Host "[$added] OK Adicionado: $id" -ForegroundColor Green
    Write-Host "   generated_title: $titleDisplay..." -ForegroundColor Gray
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
Write-Host "Adicionados: $added" -ForegroundColor Green
Write-Host "Pulados (ja tem): $skipped" -ForegroundColor Yellow
Write-Host ""

