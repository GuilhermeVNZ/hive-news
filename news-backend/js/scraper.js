// js/scraper.js - Playwright-based JavaScript renderer for dynamic content
const { chromium } = require('playwright');
const url = process.argv[2]; // Recebe URL via comando Rust

(async () => {
  let browser;
  try {
    browser = await chromium.launch({ 
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox'] // Para compatibilidade VPS
    });
    
    const page = await browser.newPage();

    // Define headers para parecer navegador real
    await page.setExtraHTTPHeaders({
      'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
      'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
      'Accept-Language': 'en-US,en;q=0.9',
      'Accept-Encoding': 'gzip, deflate, br',
      'Connection': 'keep-alive',
      'Upgrade-Insecure-Requests': '1'
    });

    // Acessa a página com timeouts generosos
    // Tenta networkidle primeiro, se falhar tenta domcontentloaded
    try {
      await page.goto(url, { 
        waitUntil: 'networkidle',
        timeout: 60000 
      });
    } catch (error) {
      // Se networkidle falhar, tenta domcontentloaded (mais rápido)
      console.error('Network idle timeout, trying domcontentloaded:', error.message);
      await page.goto(url, { 
        waitUntil: 'domcontentloaded',
        timeout: 60000 
      });
    }

    // Aguarda conteúdo real aparecer (ajustável caso a caso)
    await page.waitForTimeout(5000);

    // Tenta esperar por seletores comuns de artigos
    const commonSelectors = [
      'article',
      'a[href*="/news/"]',
      'a[href*="/blog/"]',
      '.article-card',
      '.postCard',
      '[class*="post"]',
      '[class*="article"]'
    ];

    let found = false;
    for (const selector of commonSelectors) {
      try {
        await page.waitForSelector(selector, { timeout: 2000 });
        found = true;
        break; // Pelo menos um seletor funcionou
      } catch (e) {
        // Continuar tentando outros seletores
      }
    }

    if (!found) {
      // Se não encontrou nenhum seletor específico, espera um pouco mais
      await page.waitForTimeout(2000);
    }

    // Pega HTML final renderizado
    const html = await page.content();

    // Envia HTML via STDOUT para o Rust
    console.log(html);
    
    await browser.close();
    
    // Exit code 0 = sucesso
    process.exit(0);
  } catch (error) {
    console.error('Error in scraper.js:', error.message);
    if (browser) {
      await browser.close();
    }
    // Exit code 1 = erro
    process.exit(1);
  }
})();


