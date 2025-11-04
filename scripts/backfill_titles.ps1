# Script para preencher retroativamente original_title e generated_title
# para artigos antigos no registry

param(
    [string]$RegistryPath = "G:\Hive-Hub\News-main\articles_registry.json",
    [string]$OutputDir = "G:\Hive-Hub\News-main\output"
)

Write-Host "=== Preenchendo títulos retroativamente ===" -ForegroundColor Cyan
Write-Host "Registry: $RegistryPath" -ForegroundColor Gray
Write-Host "Output: $OutputDir" -ForegroundColor Gray
Write-Host ""

# Carregar registry
if (-not (Test-Path $RegistryPath)) {
    Write-Host "ERRO: Registry não encontrado em $RegistryPath" -ForegroundColor Red
    exit 1
}

$registry = Get-Content $RegistryPath -Raw | ConvertFrom-Json
$articles = $registry.articles.PSObject.Properties
$total = $articles.Count
$updated = 0
$skipped = 0
$errors = 0

Write-Host "Total de artigos no registry: $total" -ForegroundColor Yellow
Write-Host ""

# Função para extrair arXiv ID
function Get-ArxivId {
    param([string]$Id)
    if ($Id -match "(\d{4}\.\d{4,6})") {
        return $matches[1]
    }
    return $Id
}

# Função para normalizar título
function Normalize-Title {
    param([string]$Title)
    if ([string]::IsNullOrWhiteSpace($Title)) {
        return $null
    }
    return $Title.Trim()
}

foreach ($prop in $articles) {
    $id = $prop.Name
    $meta = $prop.Value
    
    # Extrair arXiv ID
    $arxivId = Get-ArxivId $id
    
    # Verificar se precisa atualizar
    $needsUpdate = $false
    $originalTitle = $null
    $generatedTitle = $null
    
    # 1. Preencher original_title se não existir
    if ([string]::IsNullOrWhiteSpace($meta.original_title)) {
        # Usar o título atual do registry como original
        $originalTitle = Normalize-Title $meta.title
        if ($originalTitle) {
            $needsUpdate = $true
        }
    } else {
        $originalTitle = Normalize-Title $meta.original_title
    }
    
    # 2. Preencher generated_title se não existir e o artigo estiver publicado
    if ($meta.status -eq "Published" -and [string]::IsNullOrWhiteSpace($meta.generated_title)) {
        # Procurar title.txt no filesystem
        $found = $false
        foreach ($siteDir in @("AIResearch", "ScienceAI")) {
            $fullSiteDir = Join-Path $OutputDir $siteDir
            if (Test-Path $fullSiteDir) {
                $folders = Get-ChildItem -Path $fullSiteDir -Directory | Where-Object { 
                    $_.Name -like "*$arxivId*" 
                }
                foreach ($folder in $folders) {
                    $titleFile = Join-Path $folder.FullName "title.txt"
                    if (Test-Path $titleFile) {
                        $title = Normalize-Title (Get-Content $titleFile -Raw)
                        if ($title) {
                            $generatedTitle = $title
                            $found = $true
                            break
                        }
                    }
                }
                if ($found) {
                    break
                }
            }
        }
        
        if ($generatedTitle) {
            $needsUpdate = $true
        }
    }
    
    # Atualizar se necessário
    if ($needsUpdate) {
        try {
            if ($originalTitle -and ([string]::IsNullOrWhiteSpace($meta.original_title) -or -not $meta.PSObject.Properties.Name -contains "original_title")) {
                if ($meta.PSObject.Properties.Name -contains "original_title") {
                    $meta.original_title = $originalTitle
                } else {
                    $meta | Add-Member -NotePropertyName "original_title" -NotePropertyValue $originalTitle -Force
                }
            }
            if ($generatedTitle -and ([string]::IsNullOrWhiteSpace($meta.generated_title) -or -not $meta.PSObject.Properties.Name -contains "generated_title")) {
                if ($meta.PSObject.Properties.Name -contains "generated_title") {
                    $meta.generated_title = $generatedTitle
                } else {
                    $meta | Add-Member -NotePropertyName "generated_title" -NotePropertyValue $generatedTitle -Force
                }
            }
            $updated++
            Write-Host "[$updated/$total] ID: $id" -ForegroundColor Green
            if ($originalTitle) {
                Write-Host "  Original: $originalTitle" -ForegroundColor Gray
            }
            if ($generatedTitle) {
                Write-Host "  Generated: $generatedTitle" -ForegroundColor Gray
            }
        } catch {
            $errors++
            Write-Host "[ERRO] ID: $id - $($_.Exception.Message)" -ForegroundColor Red
        }
    } else {
        $skipped++
    }
}

# Salvar registry atualizado
Write-Host ""
Write-Host "=== Salvando registry atualizado ===" -ForegroundColor Cyan
try {
    $json = $registry | ConvertTo-Json -Depth 100
    $json | Set-Content $RegistryPath -Encoding UTF8
    Write-Host "✅ Registry salvo com sucesso!" -ForegroundColor Green
} catch {
    Write-Host "❌ ERRO ao salvar registry: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Resumo
Write-Host ""
Write-Host "=== Resumo ===" -ForegroundColor Cyan
Write-Host "Total: $total" -ForegroundColor White
Write-Host "Atualizados: $updated" -ForegroundColor Green
Write-Host "Ignorados: $skipped" -ForegroundColor Yellow
Write-Host "Erros: $errors" -ForegroundColor $(if ($errors -gt 0) { "Red" } else { "Green" })
Write-Host ""

