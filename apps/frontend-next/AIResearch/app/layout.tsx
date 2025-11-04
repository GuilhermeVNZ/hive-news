import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

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
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="pt-BR" suppressHydrationWarning>
      <head>
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
      <body className={inter.className}>{children}</body>
    </html>
  );
}
