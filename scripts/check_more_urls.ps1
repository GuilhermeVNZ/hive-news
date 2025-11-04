# Script para verificar mais URLs alternativas

Write-Host "Verificando mais URLs alternativas..." -ForegroundColor Cyan
Write-Host ""

$tests = @(
    @{name="Perplexity RSS"; url="https://www.perplexity.ai/feed"},
    @{name="ElevenLabs RSS"; url="https://elevenlabs.io/blog/feed"},
    @{name="IBM Research RSS"; url="https://research.ibm.com/blog/feed/rss"},
    @{name="Wired AI RSS"; url="https://www.wired.com/feed/tag/artificial-intelligence/rss"},
    @{name="MIT News AI RSS"; url="https://news.mit.edu/rss/topic/artificial-intelligence"},
    @{name="Nature AI RSS"; url="https://www.nature.com/subjects/artificial-intelligence/feed/rss"},
    @{name="Character.AI"; url="https://character.ai/"},
    @{name="Mistral Blog"; url="https://mistral.ai/blog"},
    @{name="Berkeley BAIR"; url="https://bair.berkeley.edu/blog"},
    @{name="Salesforce RSS"; url="https://www.salesforce.com/news/rss"}
)

foreach ($test in $tests) {
    Write-Host "  Testando: $($test.name)" -ForegroundColor White
    Write-Host "     URL: $($test.url)" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $test.url -Method Head -TimeoutSec 10 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            $contentType = $response.Headers['Content-Type']
            Write-Host "     [OK] Status: $($response.StatusCode) - Content-Type: $contentType" -ForegroundColor Green
        }
    } catch {
        if ($_.Exception.Response) {
            $statusCode = [int]$_.Exception.Response.StatusCode
            if ($statusCode -eq 301 -or $statusCode -eq 302 -or $statusCode -eq 307 -or $statusCode -eq 308) {
                $location = $_.Exception.Response.Headers['Location']
                Write-Host "     [REDIRECT] Para: $location" -ForegroundColor Yellow
            } else {
                Write-Host "     [ERROR] Status: $statusCode - $($_.Exception.Message)" -ForegroundColor Red
            }
        } else {
            Write-Host "     [ERROR] $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    Write-Host ""
}

Write-Host "Verificacao concluida!" -ForegroundColor Green



