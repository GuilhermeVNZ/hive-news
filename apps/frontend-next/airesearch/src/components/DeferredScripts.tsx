"use client";

import { useEffect } from "react";

/**
 * Componente que carrega scripts de tracking e ferramentas analíticas
 * APÓS a página ser renderizada, melhorando métricas de velocidade
 * Similar ao comportamento do WP Meteor
 */
export function DeferredScripts() {
  useEffect(() => {
    // Aguardar página estar totalmente renderizada e interativa
    const loadDeferredScripts = () => {
      // Carregar scripts apenas após o conteúdo estar visível
      if (document.readyState === "complete") {
        // Aguardar um pequeno delay para garantir que o LCP já foi registrado
        setTimeout(() => {
          // Aqui você pode adicionar scripts de tracking como:
          // - Google Analytics 4
          // - Google Tag Manager
          // - Facebook Pixel
          // - Hotjar
          // - Outros scripts de analytics
          // Exemplo de como carregar Google Analytics após renderização:
          // if (process.env.NEXT_PUBLIC_GA_ID) {
          //   const script = document.createElement('script');
          //   script.src = `https://www.googletagmanager.com/gtag/js?id=${process.env.NEXT_PUBLIC_GA_ID}`;
          //   script.async = true;
          //   script.defer = true;
          //   document.head.appendChild(script);
          // }
          // Carregar outros scripts de terceiros aqui
        }, 2000); // Aguardar 2 segundos após renderização
      }
    };

    // Executar quando a página estiver totalmente carregada
    if (document.readyState === "complete") {
      loadDeferredScripts();
      return undefined;
    } else {
      window.addEventListener("load", loadDeferredScripts);
      return () => window.removeEventListener("load", loadDeferredScripts);
    }
  }, []);

  return null;
}
