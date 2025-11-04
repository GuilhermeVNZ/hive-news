# Script para verificar URLs e seguir redirects

Write-Host "Verificando URLs com problemas e seguindo redirects..." -ForegroundColor Cyan
Write-Host ""

# URLs com redirect (308)
$redirects = @(
    @{id="rss_perplexity"; name="Perplexity AI Blog RSS"; url="https://blog.perplexity.ai/feed"},
    @{id="rss_venturebeat_ai"; name="VentureBeat AI RSS"; url="https://venturebeat.com/category/ai/feed/"},
    @{id="html_mistral_ai"; name="Mistral AI News"; url="https://mistral.ai/news/"},
    @{id="html_character_ai"; name="Character.AI"; url="https://beta.character.ai/"}
)

foreach ($site in $redirects) {
    Write-Host "  Verificando redirect: $($site.name)" -ForegroundColor White
    Write-Host "     URL original: $($site.url)" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $site.url -MaximumRedirection 0 -ErrorAction SilentlyContinue
    } catch {
        if ($_.Exception.Response) {
            $statusCode = [int]$_.Exception.Response.StatusCode
            if ($statusCode -eq 301 -or $statusCode -eq 302 -or $statusCode -eq 307 -or $statusCode -eq 308) {
                $location = $_.Exception.Response.Headers['Location']
                Write-Host "     [REDIRECT] Para: $location" -ForegroundColor Yellow
                
                # Testar URL de destino
                try {
                    $finalResponse = Invoke-WebRequest -Uri $location -Method Head -TimeoutSec 10 -ErrorAction Stop
                    Write-Host "     [OK] URL final acessivel: $location" -ForegroundColor Green
                } catch {
                    Write-Host "     [ERROR] URL final tambem falhou: $($_.Exception.Message)" -ForegroundColor Red
                }
            }
        }
    }
    Write-Host ""
}

# URLs alternativas para testar
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host "TESTANDO URLS ALTERNATIVAS" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host ""

$alternatives = @(
    @{id="rss_nvidia"; name="NVIDIA News RSS"; urls=@(
        "https://nvidianews.nvidia.com/rss.xml",
        "https://blogs.nvidia.com/feed/",
        "https://nvidianews.nvidia.com/rss/all-news.xml"
    )},
    @{id="html_deepseek"; name="DeepSeek News"; urls=@(
        "https://deepseek.ai/blog",
        "https://deepseek.com/blog",
        "https://blog.deepseek.ai"
    )},
    @{id="html_xai"; name="X.ai News"; urls=@(
        "https://x.ai/blog",
        "https://x.ai/updates"
    )}
)

foreach ($site in $alternatives) {
    Write-Host "  Testando alternativas: $($site.name)" -ForegroundColor White
    
    foreach ($url in $site.urls) {
        try {
            $response = Invoke-WebRequest -Uri $url -Method Head -TimeoutSec 10 -ErrorAction Stop
            if ($response.StatusCode -eq 200) {
                Write-Host "     [OK] $url" -ForegroundColor Green
                break
            }
        } catch {
            Write-Host "     [FAIL] $url - $($_.Exception.Message)" -ForegroundColor Gray
        }
    }
    Write-Host ""
}

Write-Host "Verificacao concluida!" -ForegroundColor Green



