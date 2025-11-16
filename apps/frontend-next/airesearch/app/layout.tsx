import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { DeferredScripts } from "@/components/DeferredScripts";

// Otimização de fontes: preload e display swap para melhor FCP
const inter = Inter({ 
  subsets: ["latin"],
  display: 'swap', // Evita FOIT (Flash of Invisible Text)
  preload: true,
  variable: '--font-inter',
});

export const metadata: Metadata = {
  title: {
    default: "AIResearch - Notícias Científicas sobre IA",
    template: "%s | AIResearch"
  },
  description: "Notícias científicas sobre Inteligência Artificial, Machine Learning e Deep Learning. Análises, descobertas e breakthroughs em IA.",
  keywords: ["Inteligência Artificial", "IA", "Machine Learning", "Deep Learning", "AI Research", "Notícias Científicas", "Ciência"],
  authors: [{ name: "AIResearch Team" }],
  creator: "AIResearch",
  publisher: "AIResearch",
  metadataBase: new URL('https://airesearch.com'),
  openGraph: {
    type: 'website',
    locale: 'pt_BR',
    url: 'https://airesearch.com',
    siteName: 'AIResearch',
    title: 'AIResearch - Notícias Científicas sobre IA',
    description: 'Notícias científicas sobre Inteligência Artificial, Machine Learning e Deep Learning.',
    images: [
      {
        url: '/favicon.png',
        width: 1200,
        height: 630,
        alt: 'AIResearch - Notícias Científicas sobre IA',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'AIResearch - Notícias Científicas sobre IA',
    description: 'Notícias científicas sobre Inteligência Artificial, Machine Learning e Deep Learning.',
    images: ['/favicon.png'],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  icons: {
    icon: [
      { url: '/favicon.png', sizes: 'any', type: 'image/png' },
      { url: '/favicon.png', sizes: '32x32', type: 'image/png' },
      { url: '/favicon.png', sizes: '16x16', type: 'image/png' },
    ],
    apple: '/favicon.png',
    shortcut: '/favicon.png',
  },
  alternates: {
    canonical: 'https://airesearch.com',
  },
  // Otimizações de performance
  other: {
    'x-content-type-options': 'nosniff',
    'x-frame-options': 'SAMEORIGIN',
    'x-xss-protection': '1; mode=block',
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="pt-BR" suppressHydrationWarning>
      <head>
        {/* Resource Hints para performance máxima */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link rel="dns-prefetch" href="https://fonts.googleapis.com" />
        
        {/* Preload de recursos críticos */}
        <link rel="preload" href="/favicon.png" as="image" type="image/png" fetchPriority="high" />
        
        {/* DNS Prefetch e Preconnect para domínios externos */}
        <link rel="dns-prefetch" href="https://arxiv.org" />
        <link rel="preconnect" href="https://arxiv.org" crossOrigin="anonymous" />
        
        {/* Critical CSS inline - reduz bloqueio de renderização (620ms -> ~200ms) */}
        <style dangerouslySetInnerHTML={{
          __html: `
            /* Layout inicial crítico - apenas acima da dobra */
            body{margin:0;font-family:system-ui,-apple-system,sans-serif;background:#fff;color:#1a1a1a}
            #__next,#root{min-height:100vh;display:flex;flex-direction:column}
            main{flex:1}
            /* Prevenir FOUC */
            html{visibility:visible;opacity:1}
            /* Header crítico */
            header{position:relative;z-index:50}
            /* Hero section crítica */
            section[class*="Hero"],section[class*="hero"]{position:relative;overflow:hidden}
            /* Footer - altura fixa para evitar CLS */
            footer{min-height:auto;contain-intrinsic-size:300px}
          `
        }} />
        
        {/* Prefetch de rotas prováveis */}
        
        {/* Structured Data para SEO */}
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "WebSite",
              "name": "AIResearch",
              "description": "Notícias científicas sobre Inteligência Artificial",
              "url": "https://airesearch.com",
              "potentialAction": {
                "@type": "SearchAction",
                "target": "https://airesearch.com/search?q={search_term_string}",
                "query-input": "required name=search_term_string"
              }
            })
          }}
        />
      </head>
      <body className={`${inter.className} ${inter.variable}`}>
        {children}
        <DeferredScripts />
      </body>
    </html>
  );
}
