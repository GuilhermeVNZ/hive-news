import { useEffect } from 'react';

/**
 * Componente que carrega scripts de tracking e ferramentas analíticas
 * APÓS a página ser renderizada, melhorando métricas de velocidade
 * Similar ao comportamento do WP Meteor - adia carregamento 2-5x mais rápido
 */
export function ScriptLoader() {
  useEffect(() => {
    const loadDeferredScripts = () => {
      // Carregar scripts apenas após o conteúdo estar visível e interativo
      // Usar requestIdleCallback para não bloquear renderização (similar ao WP Meteor)
      const loadScripts = () => {
        // Aqui você pode adicionar scripts de tracking como:
        // - Google Analytics 4
        // - Google Tag Manager
        // - Facebook Pixel
        // - Hotjar
        // - Outros scripts de analytics
        
        // Exemplo de como carregar Google Analytics após renderização:
        // if (import.meta.env.VITE_GA_ID) {
        //   const script = document.createElement('script');
        //   script.src = `https://www.googletagmanager.com/gtag/js?id=${import.meta.env.VITE_GA_ID}`;
        //   script.async = true;
        //   script.defer = true;
        //   document.head.appendChild(script);
        //   
        //   window.dataLayer = window.dataLayer || [];
        //   function gtag(...args: any[]) { window.dataLayer.push(args); }
        //   gtag('js', new Date());
        //   gtag('config', import.meta.env.VITE_GA_ID);
        // }
        
        // Carregar outros scripts de terceiros aqui
        console.log('[ScriptLoader] Deferred scripts loaded (WP Meteor equivalent)');
      };

      // Aguardar página estar totalmente renderizada
      if (document.readyState === 'complete') {
        // Usar requestIdleCallback se disponível (melhor performance)
        // Fallback para setTimeout se não suportado
        if (window.requestIdleCallback) {
          window.requestIdleCallback(loadScripts, { timeout: 2000 });
        } else {
          // Fallback para navegadores mais antigos
          setTimeout(loadScripts, 2000);
        }
      }
    };

    // Aguardar página estar totalmente carregada
    if (document.readyState === 'complete') {
      loadDeferredScripts();
      return undefined;
    } else {
      window.addEventListener('load', loadDeferredScripts);
      return () => window.removeEventListener('load', loadDeferredScripts);
    }
  }, []);

  return null;
}
