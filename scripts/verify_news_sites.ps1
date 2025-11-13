# Script para verificar todos os sites de news (exceto hive-hub.ai e airesearch.news)

Write-Host "Verificando sites de news..." -ForegroundColor Cyan
Write-Host ""

$results = @()

# RSS Collectors
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host "RSS FEEDS" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host ""

$rssSites = @(
    @{id="rss_openai"; name="OpenAI Blog RSS"; url="https://openai.com/blog/rss.xml"},
    @{id="rss_google_ai"; name="Google AI RSS"; url="https://blog.research.google/feeds/posts/default"},
    @{id="rss_nvidia"; name="NVIDIA News RSS"; url="https://nvidianews.nvidia.com/rss.xml"},
    @{id="rss_alibaba_damo"; name="Alibaba DAMO RSS"; url="https://damo.alibaba.com/news/rss"},
    @{id="rss_perplexity"; name="Perplexity AI Blog RSS"; url="https://blog.perplexity.ai/feed"},
    @{id="rss_huggingface"; name="Hugging Face Blog RSS"; url="https://huggingface.co/blog/feed.xml"},
    @{id="rss_elevenlabs"; name="ElevenLabs Blog RSS"; url="https://blog.elevenlabs.io/feed"},
    @{id="rss_microsoft_ai"; name="Microsoft AI Blog RSS"; url="https://blogs.microsoft.com/ai/feed/"},
    @{id="rss_ibm_research"; name="IBM Research AI RSS"; url="https://research.ibm.com/blog/feed"},
    @{id="rss_salesforce_ai"; name="Salesforce AI Blog RSS"; url="https://www.salesforce.com/news/feed/"},
    @{id="rss_techcrunch_ai"; name="TechCrunch AI RSS"; url="https://techcrunch.com/tag/artificial-intelligence/feed/"},
    @{id="rss_venturebeat_ai"; name="VentureBeat AI RSS"; url="https://venturebeat.com/category/ai/feed/"},
    @{id="rss_the_verge_ai"; name="The Verge AI RSS"; url="https://www.theverge.com/rss/group/ai/index.xml"},
    @{id="rss_wired_ai"; name="Wired AI RSS"; url="https://www.wired.com/feed/category/science/ai/latest/rss"},
    @{id="rss_mit_tech_review_ai"; name="MIT Technology Review AI RSS"; url="https://news.mit.edu/topic/artificial-intelligence-rss.xml"},
    @{id="rss_nature_ai"; name="Nature AI RSS"; url="https://www.nature.com/subjects/artificial-intelligence.rss"},
    @{id="rss_science_ai"; name="Science AI RSS"; url="https://www.science.org/topic/artificial-intelligence/rss"}
)

foreach ($site in $rssSites) {
    Write-Host "  Verificando: $($site.name)" -ForegroundColor White
    Write-Host "     URL: $($site.url)" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $site.url -Method Head -TimeoutSec 10 -ErrorAction Stop
        $statusCode = $response.StatusCode
        $contentType = $response.Headers['Content-Type']
        
        if ($statusCode -eq 200) {
            if ($contentType -like "*xml*" -or $contentType -like "*rss*" -or $contentType -like "*atom*" -or $contentType -like "*text*") {
                Write-Host "     [OK] Status: OK ($statusCode) - Content-Type: $contentType" -ForegroundColor Green
                $results += @{
                    id = $site.id
                    name = $site.name
                    url = $site.url
                    status = "OK"
                    message = "RSS feed acessivel"
                }
            } else {
                Write-Host "     [WARNING] Status: OK ($statusCode) mas Content-Type inesperado: $contentType" -ForegroundColor Yellow
                $results += @{
                    id = $site.id
                    name = $site.name
                    url = $site.url
                    status = "WARNING"
                    message = "Content-Type inesperado: $contentType"
                }
            }
        } else {
            Write-Host "     [ERROR] Status: $statusCode" -ForegroundColor Red
            $results += @{
                id = $site.id
                name = $site.name
                url = $site.url
                status = "ERROR"
                message = "Status code: $statusCode"
            }
        }
    } catch {
        Write-Host "     [ERROR] Erro: $($_.Exception.Message)" -ForegroundColor Red
        $results += @{
            id = $site.id
            name = $site.name
            url = $site.url
            status = "ERROR"
            message = $_.Exception.Message
        }
    }
    Write-Host ""
}

# HTML Collectors
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host "HTML SITES" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host ""

$htmlSites = @(
    @{id="html_anthropic"; name="Anthropic News"; url="https://www.anthropic.com/news"},
    @{id="html_meta_ai"; name="Meta AI Blog"; url="https://ai.meta.com/blog/"},
    @{id="html_deepseek"; name="DeepSeek Blog"; url="https://deepseek.ai/blog"},
    @{id="html_alibaba_damo"; name="Alibaba Alizila News"; url="https://www.alizila.com/"},
    @{id="html_xai"; name="X.ai News"; url="https://x.ai/news"},
    @{id="html_mistral_ai"; name="Mistral AI News"; url="https://mistral.ai/news/"},
    @{id="html_cohere"; name="Cohere AI Blog"; url="https://txt.cohere.com/"},
    @{id="html_stability_ai"; name="Stability AI News"; url="https://stability.ai/news"},
    @{id="html_character_ai"; name="Character.AI"; url="https://character.ai/"},
    @{id="html_inflection_ai"; name="Inflection AI (Pi)"; url="https://inflection.ai/"},
    @{id="html_apple_ml"; name="Apple Machine Learning Journal"; url="https://machinelearning.apple.com/"},
    @{id="html_intel_ai"; name="Intel AI Blog"; url="https://www.intel.com/content/www/us/en/artificial-intelligence/posts.html"},
    @{id="html_amd_ai"; name="AMD AI / Machine Learning"; url="https://community.amd.com/t5/ai-and-ml/bg-p/ai-ml"},
    @{id="html_stanford_hai"; name="Stanford HAI News"; url="https://hai.stanford.edu/news"},
    @{id="html_berkeley_ai"; name="Berkeley AI Research Blog"; url="https://bair.berkeley.edu/blog"},
    @{id="html_deepmind_blog"; name="DeepMind Blog"; url="https://deepmind.google/discover/blog/"},
    @{id="html_menlo_ventures"; name="Menlo Ventures AI"; url="https://menlovc.com/focus-areas/ai/"}
)

foreach ($site in $htmlSites) {
    Write-Host "  Verificando: $($site.name)" -ForegroundColor White
    Write-Host "     URL: $($site.url)" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $site.url -Method Head -TimeoutSec 10 -ErrorAction Stop
        $statusCode = $response.StatusCode
        $contentType = $response.Headers['Content-Type']
        
        if ($statusCode -eq 200) {
            if ($contentType -like "*html*" -or $contentType -like "*text*") {
                Write-Host "     [OK] Status: OK ($statusCode) - Content-Type: $contentType" -ForegroundColor Green
                $results += @{
                    id = $site.id
                    name = $site.name
                    url = $site.url
                    status = "OK"
                    message = "Site acessivel"
                }
            } else {
                Write-Host "     [WARNING] Status: OK ($statusCode) mas Content-Type inesperado: $contentType" -ForegroundColor Yellow
                $results += @{
                    id = $site.id
                    name = $site.name
                    url = $site.url
                    status = "WARNING"
                    message = "Content-Type inesperado: $contentType"
                }
            }
        } else {
            Write-Host "     [ERROR] Status: $statusCode" -ForegroundColor Red
            $results += @{
                id = $site.id
                name = $site.name
                url = $site.url
                status = "ERROR"
                message = "Status code: $statusCode"
            }
        }
    } catch {
        Write-Host "     [ERROR] Erro: $($_.Exception.Message)" -ForegroundColor Red
        $results += @{
            id = $site.id
            name = $site.name
            url = $site.url
            status = "ERROR"
            message = $_.Exception.Message
        }
    }
    Write-Host ""
}

# Resumo
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host "RESUMO" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Yellow
Write-Host ""

$okCount = ($results | Where-Object { $_.status -eq "OK" }).Count
$warningCount = ($results | Where-Object { $_.status -eq "WARNING" }).Count
$errorCount = ($results | Where-Object { $_.status -eq "ERROR" }).Count

Write-Host "  [OK] OK: $okCount" -ForegroundColor Green
Write-Host "  [WARNING] WARNING: $warningCount" -ForegroundColor Yellow
Write-Host "  [ERROR] ERROR: $errorCount" -ForegroundColor Red
Write-Host ""

if ($errorCount -gt 0) {
    Write-Host "  Sites com ERRO:" -ForegroundColor Red
    foreach ($result in ($results | Where-Object { $_.status -eq "ERROR" })) {
        Write-Host "     - $($result.name): $($result.message)" -ForegroundColor Red
    }
    Write-Host ""
}

if ($warningCount -gt 0) {
    Write-Host "  Sites com WARNING:" -ForegroundColor Yellow
    foreach ($result in ($results | Where-Object { $_.status -eq "WARNING" })) {
        Write-Host "     - $($result.name): $($result.message)" -ForegroundColor Yellow
    }
    Write-Host ""
}

Write-Host "Verificacao concluida!" -ForegroundColor Green
