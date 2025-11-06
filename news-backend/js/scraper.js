// js/scraper.js - Playwright-based JavaScript renderer for dynamic content
// Handles cookie banners, popups, and JavaScript-heavy sites
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

    // Aguarda um pouco para popups e modais aparecerem
    await page.waitForTimeout(2000);

    // Tenta aceitar cookies automaticamente (múltiplos seletores comuns)
    const cookieSelectors = [
      'button[id*="accept"]',
      'button[class*="accept"]',
      'button[id*="cookie"]',
      'button[class*="cookie"]',
      'button:has-text("Accept")',
      'button:has-text("Accept All")',
      'button:has-text("I Accept")',
      'button:has-text("Agree")',
      'button:has-text("OK")',
      'a[class*="accept"]',
      'a[id*="accept"]',
      '[data-testid*="accept"]',
      '[id*="cookie-banner"] button',
      '[class*="cookie-banner"] button',
      '[id*="cookie-consent"] button',
      '[class*="cookie-consent"] button',
      '#onetrust-accept-btn-handler', // OneTrust
      '#CybotCookiebotDialogBodyLevelButtonLevelOptinAllowAll', // Cookiebot
      '.cookie-banner button:first-child',
      '.cookie-notice button:first-child',
    ];

    let cookieAccepted = false;
    for (const selector of cookieSelectors) {
      try {
        const button = await page.$(selector);
        if (button) {
          const isVisible = await button.isVisible();
          if (isVisible) {
            await button.click();
            cookieAccepted = true;
            console.error('Cookie banner accepted using selector:', selector);
            await page.waitForTimeout(1000); // Aguarda animação fechar
            break;
          }
        }
      } catch (e) {
        // Continuar tentando outros seletores
      }
    }

    // Se não encontrou botão de cookie, tenta buscar por texto
    if (!cookieAccepted) {
      try {
        const acceptButtons = await page.$$eval('button, a', elements => 
          elements.filter(el => {
            const text = el.textContent.toLowerCase().trim();
            return text.includes('accept') || 
                   text.includes('agree') || 
                   text.includes('consent') ||
                   text.includes('ok') ||
                   text.includes('allow all');
          })
        );
        
        if (acceptButtons.length > 0) {
          await acceptButtons[0].click();
          cookieAccepted = true;
          console.error('Cookie banner accepted using text search');
          await page.waitForTimeout(1000);
        }
      } catch (e) {
        // Ignorar erros
      }
    }

    // Tenta fechar outros popups/modais comuns
    const popupSelectors = [
      'button[aria-label="Close"]',
      'button[class*="close"]',
      'button[id*="close"]',
      '[class*="modal"] button[class*="close"]',
      '[class*="popup"] button[class*="close"]',
      '[id*="modal"] button[class*="close"]',
      '[id*="popup"] button[class*="close"]',
      '.close-button',
      '#close-button',
    ];

    for (const selector of popupSelectors) {
      try {
        const button = await page.$(selector);
        if (button) {
          const isVisible = await button.isVisible();
          if (isVisible) {
            await button.click();
            console.error('Popup closed using selector:', selector);
            await page.waitForTimeout(500);
            break;
          }
        }
      } catch (e) {
        // Continuar tentando outros seletores
      }
    }

    // Aguarda conteúdo real aparecer após interações
    await page.waitForTimeout(3000);

    // Tenta esperar por seletores comuns de artigos
    const commonSelectors = [
      'article',
      'a[href*="/news/"]',
      'a[href*="/blog/"]',
      '.article-card',
      '.postCard',
      '[class*="post"]',
      '[class*="article"]',
      'main',
      '[role="main"]',
    ];

    let found = false;
    for (const selector of commonSelectors) {
      try {
        await page.waitForSelector(selector, { timeout: 3000 });
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


