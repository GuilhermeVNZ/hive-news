# Script para corrigir artigos Published que têm title.txt mas não têm generated_title no registry

$registryPath = "G:\Hive-Hub\News-main\articles_registry.json"
$outputDir = "G:\Hive-Hub\News-main\output"

Write-Host "=== Corrigindo generated_title para artigos Published ===" -ForegroundColor Cyan
Write-Host ""

# Carregar registry
$registry = Get-Content -Path $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties

$updated = 0
$notFound = 0
$alreadyHas = 0

function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $Id
}

foreach ($prop in $articles) {
    $id = $prop.Name
    $article = $prop.Value
    
    # Apenas artigos Published
    if ($article.status -ne "Published") {
        continue
    }
    
    # Verificar se já tem generated_title
    $hasGenerated = $article.generated_title -and $article.generated_title -ne "" -and $article.generated_title -ne $null
    
    if ($hasGenerated) {
        $alreadyHas++
        continue
    }
    
    # Procurar title.txt
    $titleFileContent = $null
    $foundPath = $null
    
    # Primeiro tentar pelo output_dir
    if ($article.output_dir) {
        $outputDirPath = $article.output_dir -replace '\\', '/'
        $titleFilePath = Join-Path $outputDirPath "title.txt"
        
        if (Test-Path $titleFilePath) {
            $titleFileContent = (Get-Content $titleFilePath -Raw -Encoding UTF8).Trim()
            $foundPath = $titleFilePath
        }
    }
    
    # Se não encontrou, buscar em AIResearch ou ScienceAI
    if (-not $titleFileContent) {
        $arxivId = Get-ArxivId $id
        
        foreach ($siteDir in @("AIResearch", "ScienceAI")) {
            $sitePath = Join-Path $outputDir $siteDir
            if (Test-Path $sitePath) {
                $folders = Get-ChildItem -Path $sitePath -Directory -ErrorAction SilentlyContinue | Where-Object { 
                    $_.Name -like "*$arxivId*" -or $_.Name -like "*$id*"
                }
                
                foreach ($folder in $folders) {
                    $titleFile = Join-Path $folder.FullName "title.txt"
                    if (Test-Path $titleFile) {
                        $titleFileContent = (Get-Content $titleFile -Raw -Encoding UTF8).Trim()
                        $foundPath = $titleFile
                        break
                    }
                }
                
                if ($titleFileContent) {
                    break
                }
            }
        }
    }
    
    if ($titleFileContent) {
        # Atualizar registry
        if ($article.PSObject.Properties.Name -contains "generated_title") {
            $article.generated_title = $titleFileContent
        } else {
            $article | Add-Member -NotePropertyName "generated_title" -NotePropertyValue $titleFileContent -Force
        }
        $updated++
        Write-Host "[$updated] ✅ ID: $id" -ForegroundColor Green
        Write-Host "   Arquivo: $foundPath" -ForegroundColor Gray
        Write-Host "   Título: $($titleFileContent.Substring(0, [Math]::Min(60, $titleFileContent.Length)))..." -ForegroundColor Gray
    } else {
        $notFound++
        Write-Host "[$notFound] ❌ ID: $id - title.txt não encontrado" -ForegroundColor Red
        Write-Host "   OutputDir: $($article.output_dir)" -ForegroundColor Gray
    }
}

# Salvar registry atualizado
Write-Host ""
Write-Host "=== Salvando registry atualizado ===" -ForegroundColor Cyan
try {
    $json = $registry | ConvertTo-Json -Depth 100
    $json | Set-Content $registryPath -Encoding UTF8
    Write-Host "✅ Registry salvo com sucesso!" -ForegroundColor Green
} catch {
    Write-Host "❌ ERRO ao salvar registry: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Resumo
Write-Host ""
Write-Host "=== Resumo ===" -ForegroundColor Cyan
Write-Host "Atualizados: $updated" -ForegroundColor Green
Write-Host "Já tinham generated_title: $alreadyHas" -ForegroundColor Yellow
Write-Host "title.txt não encontrado: $notFound" -ForegroundColor Red
Write-Host ""

