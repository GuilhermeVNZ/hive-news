# Verificação de Sites de News - Relatório

**Data**: 2025-11-02  
**Status**: 18 OK / 16 ERROS

## Sites com ERRO - Requer Correção

### RSS Feeds com ERRO

1. **NVIDIA News RSS** - `https://nvidianews.nvidia.com/rss/all-news.xml`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar URL correta do RSS da NVIDIA
   - Possível URL: `https://nvidianews.nvidia.com/rss.xml` ou `https://blogs.nvidia.com/feed/`

2. **Perplexity AI Blog RSS** - `https://blog.perplexity.ai/feed`
   - Status: 308 (Redirect permanente)
   - Sugestão: Seguir redirect e atualizar URL

3. **ElevenLabs Blog RSS** - `https://blog.elevenlabs.io/feed`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar se ElevenLabs tem RSS feed ou mudou URL

4. **IBM Research AI RSS** - `https://research.ibm.com/blog/feed`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar URL correta do RSS do IBM Research

5. **Salesforce AI Blog RSS** - `https://www.salesforce.com/news/feed/`
   - Status: 500 (Erro interno do servidor)
   - Sugestão: Pode ser temporário, verificar novamente

6. **VentureBeat AI RSS** - `https://venturebeat.com/category/ai/feed/`
   - Status: 308 (Redirect permanente)
   - Sugestão: Seguir redirect e atualizar URL

7. **Wired AI RSS** - `https://www.wired.com/feed/category/science/ai/latest/rss`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar estrutura de URL do Wired RSS

8. **MIT Technology Review AI RSS** - `https://news.mit.edu/topic/artificial-intelligence-rss.xml`
   - Status: 404 (Não encontrado)
   - Sugestão: URL pode estar incorreta, verificar estrutura do MIT News RSS

9. **Nature AI RSS** - `https://www.nature.com/subjects/artificial-intelligence.rss`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar URL correta do RSS da Nature

10. **Science AI RSS** - `https://www.science.org/topic/artificial-intelligence/rss`
    - Status: 403 (Proibido)
    - Sugestão: Pode requerer autenticação ou ter política anti-scraping

### HTML Sites com ERRO

1. **DeepSeek News** - `https://deepseek.com/news`
   - Status: 403 (Proibido)
   - Sugestão: Site pode ter proteção anti-bot. Tentar `https://deepseek.ai/blog` ou verificar se requer headers específicos

2. **X.ai News** - `https://x.ai/news`
   - Status: 403 (Proibido)
   - Sugestão: Site pode ter proteção anti-bot. Verificar se há página de blog alternativa

3. **Mistral AI News** - `https://mistral.ai/news/`
   - Status: 308 (Redirect permanente)
   - Sugestão: Seguir redirect e atualizar URL

4. **Character.AI** - `https://beta.character.ai/`
   - Status: 308 (Redirect permanente)
   - Sugestão: Seguir redirect (provavelmente para `https://character.ai/`)

5. **Intel AI Blog** - `https://www.intel.com/content/www/us/en/artificial-intelligence/posts.html`
   - Status: 403 (Proibido)
   - Sugestão: Site pode ter proteção anti-bot ou estrutura HTML diferente

6. **Berkeley AI Research News** - `https://bair.berkeley.edu/news`
   - Status: 404 (Não encontrado)
   - Sugestão: Verificar se o BAIR tem página de news ou mudou estrutura

## Sites OK (18 sites)

### RSS Feeds OK
- ✅ OpenAI Blog RSS
- ✅ Google AI RSS
- ✅ Alibaba DAMO RSS
- ✅ Hugging Face Blog RSS
- ✅ Microsoft AI Blog RSS
- ✅ TechCrunch AI RSS
- ✅ The Verge AI RSS

### HTML Sites OK
- ✅ Anthropic News
- ✅ Meta AI Blog
- ✅ Alibaba Alizila News
- ✅ Cohere AI Blog
- ✅ Stability AI News
- ✅ Inflection AI (Pi)
- ✅ Apple Machine Learning Journal
- ✅ AMD AI / Machine Learning
- ✅ Stanford HAI News
- ✅ DeepMind Blog
- ✅ Menlo Ventures AI

## Ações Recomendadas

1. **Para sites com 308 (Redirect)**: Atualizar URLs para o destino final do redirect
2. **Para sites com 403 (Proibido)**: Avaliar se requerem headers de user-agent ou são protegidos contra scraping
3. **Para sites com 404 (Não encontrado)**: Verificar manualmente se a URL mudou ou se o site ainda oferece feed/página de news
4. **Para sites com 500 (Erro interno)**: Testar novamente após algumas horas - pode ser temporário



