# News Sites Status Report

## Summary
- **Total Sites**: 31 news collectors configured
- **RSS Feeds**: 11 sites
- **HTML Scrapers**: 20 sites

## RSS Collectors (Usually More Reliable)

| Site | URL | Status |
|------|-----|--------|
| OpenAI | `https://openai.com/blog/rss.xml` | ✅ Should work |
| Google AI | `https://blog.research.google/feeds/posts/default` | ✅ Should work |
| NVIDIA | `https://nvidianews.nvidia.com/rss.xml` | ✅ Should work |
| Alibaba DAMO | `https://damo.alibaba.com/news/rss` | ⚠️ Needs testing |
| Hugging Face | `https://huggingface.co/blog/feed.xml` | ✅ Should work |
| ElevenLabs | `https://blog.elevenlabs.io/feed` | ✅ Should work |
| Microsoft AI | `https://blogs.microsoft.com/ai/feed/` | ✅ Should work |
| IBM Research | `https://research.ibm.com/blog/feed` | ✅ Should work |
| Salesforce | `https://www.salesforce.com/news/feed/` | ✅ Should work |
| TechCrunch AI | `https://techcrunch.com/tag/artificial-intelligence/feed/` | ✅ Should work |
| Qualcomm | `https://www.qualcomm.com/news/rss/allnews.xml` | ✅ Should work |

## HTML Scrapers (Potential Blockers)

### High Risk (Likely to Block)

| Site | URL | Known Issue |
|------|-----|-------------|
| Anthropic | `https://www.anthropic.com/news` | ⚠️ May have bot protection |
| Perplexity | `https://www.perplexity.ai/discover/tech` | ⚠️ May require JS rendering |
| Stability AI | `https://stability.ai/news` | ⚠️ May have Cloudflare |
| Character.AI | `https://blog.character.ai/` | ⚠️ May have bot protection |

### Medium Risk (May Need Headers/JS)

| Site | URL | Notes |
|------|-----|-------|
| Meta AI | `https://ai.meta.com/blog/` | May need proper headers |
| DeepSeek | `https://deepseek.ai/blog` | May need JS rendering |
| X.AI | `https://x.ai/news` | May have rate limiting |
| Mistral AI | `https://mistral.ai/news/` | May need JS |
| Cohere | `https://txt.cohere.com/` | May need proper headers |
| Inflection AI | `https://inflection.ai/blog/enterprise` | May need JS |

### Low Risk (Standard Scraping)

| Site | URL | Notes |
|------|-----|-------|
| Alibaba (Alizila) | `https://www.alizila.com/` | General news site |
| Apple ML Highlights | `https://machinelearning.apple.com/highlights` | May be static |
| Apple ML Research | `https://machinelearning.apple.com/research` | May be static |
| Intel AI | `https://www.intel.com/...` | Complex URL with filters |
| AMD AI | `https://www.amd.com/...` | Complex URL with sorting |
| Stanford HAI | `https://hai.stanford.edu/news?filterBy=news` | Academic site |
| Berkeley AI | `https://bair.berkeley.edu/blog/archive/` | Academic site |
| DeepMind Blog | `https://deepmind.google/discover/blog/` | May need JS |

## Current Anti-Bot Measures in Place

✅ **Already Implemented:**
- Chrome User-Agent spoofing
- Accept headers (HTML, XHTML, XML)
- Accept-Language (en-US)
- Accept-Encoding (gzip, deflate, br)
- Sec-Fetch headers (site, mode, user, dest)
- Referer: Google.com
- Cookie store enabled
- Redirect following (max 5)
- 60s timeout

## Recommended Actions

### For RSS Sites
1. Test each RSS feed individually
2. If blocked: add exponential backoff retry
3. If 403/401: check if API key needed
4. If 429: implement rate limiting

### For HTML Sites (Needs JS Rendering)
Consider these approaches:
1. **Playwright Integration** (already available in Docker)
   - Use for sites that require JS
   - Slower but more reliable
2. **Session Management**
   - Store cookies between requests
   - Maintain session state
3. **Proxy Rotation** (future)
   - Use proxy services if blocking persists

### Priority Testing Order
1. **High-value RSS feeds first** (OpenAI, Google, NVIDIA)
2. **HTML sites with known content** (Meta, DeepMind, Anthropic)
3. **Complex filtered URLs** (Intel, AMD, Stanford)

## Next Steps

1. Run test script to check each site
2. Document which sites return 403/429/blocked
3. Implement site-specific workarounds:
   - Add delays between requests
   - Use Playwright for JS-heavy sites
   - Add retries with exponential backoff
4. Create site-specific configs for problematic ones

---

Generated: $(date)
Last Update: Manual configuration review


