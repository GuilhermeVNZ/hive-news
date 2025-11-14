# Test script for news sites accessibility
# Run: .\test-sites.ps1

$ErrorActionPreference = "Continue"

# Define sites to test
$rssSites = @(
    @{Name="OpenAI"; Url="https://openai.com/blog/rss.xml"},
    @{Name="Google AI"; Url="https://blog.research.google/feeds/posts/default"},
    @{Name="NVIDIA"; Url="https://nvidianews.nvidia.com/rss.xml"},
    @{Name="Alibaba DAMO"; Url="https://damo.alibaba.com/news/rss"},
    @{Name="Hugging Face"; Url="https://huggingface.co/blog/feed.xml"},
    @{Name="ElevenLabs"; Url="https://blog.elevenlabs.io/feed"},
    @{Name="Microsoft AI"; Url="https://blogs.microsoft.com/ai/feed/"},
    @{Name="IBM Research"; Url="https://research.ibm.com/blog/feed"},
    @{Name="Salesforce"; Url="https://www.salesforce.com/news/feed/"},
    @{Name="TechCrunch AI"; Url="https://techcrunch.com/tag/artificial-intelligence/feed/"},
    @{Name="Qualcomm"; Url="https://www.qualcomm.com/news/rss/allnews.xml"}
)

$htmlSites = @(
    @{Name="Anthropic"; Url="https://www.anthropic.com/news"},
    @{Name="Meta AI"; Url="https://ai.meta.com/blog/"},
    @{Name="DeepSeek"; Url="https://deepseek.ai/blog"},
    @{Name="X.AI"; Url="https://x.ai/news"},
    @{Name="Mistral AI"; Url="https://mistral.ai/news/"},
    @{Name="Cohere"; Url="https://txt.cohere.com/"},
    @{Name="Perplexity"; Url="https://www.perplexity.ai/discover/tech"},
    @{Name="Stability AI"; Url="https://stability.ai/news"},
    @{Name="Character.AI"; Url="https://blog.character.ai/"},
    @{Name="Inflection AI"; Url="https://inflection.ai/blog/enterprise"},
    @{Name="Apple ML Highlights"; Url="https://machinelearning.apple.com/highlights"},
    @{Name="Apple ML Research"; Url="https://machinelearning.apple.com/research"},
    @{Name="Stanford HAI"; Url="https://hai.stanford.edu/news?filterBy=news"},
    @{Name="Berkeley AI"; Url="https://bair.berkeley.edu/blog/archive/"},
    @{Name="DeepMind Blog"; Url="https://deepmind.google/discover/blog/"}
)

# Results tracking
$results = @{
    Success = @()
    Failed = @()
    Blocked = @()
    Timeout = @()
}

# Headers to simulate browser
$headers = @{
    "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    "Accept" = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
    "Accept-Language" = "en-US,en;q=0.9"
    "Referer" = "https://www.google.com/"
}

function Test-Site {
    param (
        [string]$Name,
        [string]$Url,
        [string]$Type
    )
    
    Write-Host "`n🔍 Testing: $Name" -ForegroundColor Cyan
    Write-Host "   URL: $Url" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $Url -Headers $headers -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        $statusCode = $response.StatusCode
        $contentLength = $response.Content.Length
        
        Write-Host "   ✅ SUCCESS - Status: $statusCode, Size: $contentLength bytes" -ForegroundColor Green
        $results.Success += @{Name=$Name; Url=$Url; Type=$Type; Status=$statusCode; Size=$contentLength}
        return $true
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        $message = $_.Exception.Message
        
        if ($statusCode -eq 403 -or $statusCode -eq 401) {
            Write-Host "   🚫 BLOCKED - Status: $statusCode (Auth/Bot protection)" -ForegroundColor Red
            $results.Blocked += @{Name=$Name; Url=$Url; Type=$Type; Status=$statusCode}
        }
        elseif ($statusCode -eq 429) {
            Write-Host "   ⚠️  RATE LIMITED - Status: $statusCode" -ForegroundColor Yellow
            $results.Failed += @{Name=$Name; Url=$Url; Type=$Type; Status=$statusCode; Reason="Rate limit"}
        }
        elseif ($message -like "*timeout*") {
            Write-Host "   ⏱️  TIMEOUT - Request took too long" -ForegroundColor Yellow
            $results.Timeout += @{Name=$Name; Url=$Url; Type=$Type; Reason="Timeout"}
        }
        else {
            Write-Host "   ❌ FAILED - $message" -ForegroundColor Red
            $results.Failed += @{Name=$Name; Url=$Url; Type=$Type; Reason=$message}
        }
        return $false
    }
}

# Test all RSS sites
Write-Host "`n========================================" -ForegroundColor Magenta
Write-Host "   TESTING RSS FEEDS" -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta

foreach ($site in $rssSites) {
    Test-Site -Name $site.Name -Url $site.Url -Type "RSS"
    Start-Sleep -Milliseconds 500  # Polite delay
}

# Test all HTML sites
Write-Host "`n========================================" -ForegroundColor Magenta
Write-Host "   TESTING HTML SITES" -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta

foreach ($site in $htmlSites) {
    Test-Site -Name $site.Name -Url $site.Url -Type "HTML"
    Start-Sleep -Milliseconds 500  # Polite delay
}

# Generate summary report
Write-Host "`n========================================" -ForegroundColor Magenta
Write-Host "   SUMMARY REPORT" -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta

$totalSites = $rssSites.Count + $htmlSites.Count
$successCount = $results.Success.Count
$failedCount = $results.Failed.Count
$blockedCount = $results.Blocked.Count
$timeoutCount = $results.Timeout.Count

Write-Host "`n📊 Statistics:" -ForegroundColor Cyan
Write-Host "   Total Sites: $totalSites" -ForegroundColor White
Write-Host "   ✅ Success: $successCount ($([math]::Round($successCount/$totalSites*100, 1))%)" -ForegroundColor Green
Write-Host "   🚫 Blocked: $blockedCount ($([math]::Round($blockedCount/$totalSites*100, 1))%)" -ForegroundColor Red
Write-Host "   ❌ Failed: $failedCount ($([math]::Round($failedCount/$totalSites*100, 1))%)" -ForegroundColor Red
Write-Host "   ⏱️  Timeout: $timeoutCount ($([math]::Round($timeoutCount/$totalSites*100, 1))%)" -ForegroundColor Yellow

# Detailed problematic sites
if ($results.Blocked.Count -gt 0) {
    Write-Host "`n🚫 BLOCKED SITES (Require Workarounds):" -ForegroundColor Red
    foreach ($site in $results.Blocked) {
        Write-Host "   - $($site.Name) ($($site.Type)) - Status: $($site.Status)" -ForegroundColor Red
    }
}

if ($results.Failed.Count -gt 0) {
    Write-Host "`n❌ FAILED SITES (Check Configuration):" -ForegroundColor Red
    foreach ($site in $results.Failed) {
        Write-Host "   - $($site.Name) ($($site.Type)) - Reason: $($site.Reason)" -ForegroundColor Red
    }
}

if ($results.Timeout.Count -gt 0) {
    Write-Host "`n⏱️  TIMEOUT SITES (May Need JS Rendering):" -ForegroundColor Yellow
    foreach ($site in $results.Timeout) {
        Write-Host "   - $($site.Name) ($($site.Type))" -ForegroundColor Yellow
    }
}

# Recommendations
Write-Host "`n💡 RECOMMENDATIONS:" -ForegroundColor Cyan

if ($blockedCount -gt 0) {
    Write-Host "   1. Implement Playwright rendering for blocked HTML sites" -ForegroundColor White
    Write-Host "   2. Add random User-Agent rotation" -ForegroundColor White
    Write-Host "   3. Increase delays between requests (2-5 seconds)" -ForegroundColor White
}

if ($timeoutCount -gt 0) {
    Write-Host "   4. Use JavaScript rendering for timeout sites" -ForegroundColor White
    Write-Host "   5. Increase timeout to 30-60 seconds for heavy pages" -ForegroundColor White
}

if ($failedCount -gt 0) {
    Write-Host "   6. Review failed sites for API key requirements" -ForegroundColor White
    Write-Host "   7. Check if URLs have changed/moved" -ForegroundColor White
}

Write-Host "`n📝 Save this output to: test-results-$(Get-Date -Format 'yyyy-MM-dd-HHmm').txt" -ForegroundColor Gray
Write-Host "`nDone! Check SITES_STATUS.md for detailed action plan." -ForegroundColor Green



