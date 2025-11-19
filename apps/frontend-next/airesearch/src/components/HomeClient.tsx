"use client";

import { useEffect, useState } from "react";
import dynamic from "next/dynamic";
import { useRouter } from "next/navigation";
import Header from "@/components/Header";
import Hero from "@/components/Hero";
import ArticleGrid from "@/components/ArticleGrid";
import type { Article } from "@/types/article";

// Islands Hydration: Footer carrega sem SSR (não é crítico para SEO)
// Reduz JavaScript enviado para blocos estáticos abaixo da dobra
const Footer = dynamic(() => import("@/components/Footer"), {
  ssr: false, // Não renderizar no servidor - apenas client-side
});

interface HomeClientProps {
  initialArticles: Article[];
  initialHasMore?: boolean;
  initialTotal?: number;
  initialCategory?: string;
  initialQuery?: string;
}

export default function HomeClient({
  initialArticles,
  initialHasMore = false,
  initialTotal = 0,
  initialCategory = "",
  initialQuery = "",
}: HomeClientProps) {
  const router = useRouter();
  const [selectedCategory, setSelectedCategory] =
    useState<string>(initialCategory);
  const [searchQuery, setSearchQuery] = useState<string>(initialQuery);
  const [committedQuery, setCommittedQuery] = useState<string>(initialQuery);
  
  // Prefetch inteligente de rotas quando o mouse está sobre cards de artigos
  // Prefetch HTML/JSON da rota antes do clique para navegação quase instantânea
  useEffect(() => {
    const handleMouseEnter = (e: MouseEvent) => {
      // Verificar se e.target é um Element válido antes de usar closest
      const target = e.target;
      if (!target || !(target instanceof Element)) {
        return;
      }
      
      const link = target.closest('a');
      if (link && link.href) {
        try {
          const url = new URL(link.href);
          if (url.pathname.startsWith('/article/')) {
            // Prefetch da página completa (HTML + dados)
            router.prefetch(url.pathname);
            // Também prefetch dos dados da API se necessário
            // O Next.js já faz isso automaticamente com prefetch
          }
        } catch (err) {
          // Ignorar URLs inválidas
        }
      }
    };
    
    // Usar mouseenter para prefetch proativo
    document.addEventListener('mouseenter', handleMouseEnter, true);
    return () => document.removeEventListener('mouseenter', handleMouseEnter, true);
  }, [router]);

  useEffect(() => {
    setSelectedCategory(initialCategory);
  }, [initialCategory]);

  useEffect(() => {
    setSearchQuery(initialQuery);
    setCommittedQuery(initialQuery);
  }, [initialQuery]);

  useEffect(() => {
    if (!initialCategory) {
      return;
    }

    const timeout = setTimeout(() => {
      const articlesSection = document.getElementById("articles");
      if (articlesSection) {
        articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 100);

    return () => clearTimeout(timeout);
  }, [initialCategory]);

  const handleCategorySelect = (category: string) => {
    setSelectedCategory((current) => (category === current ? "" : category));

    setTimeout(() => {
      const articlesSection = document.getElementById("articles");
      if (articlesSection) {
        articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 100);
  };

  const handleSubmitSearch = () => {
    setCommittedQuery(searchQuery);

    setTimeout(() => {
      const articlesSection = document.getElementById("articles");
      if (articlesSection) {
        articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 50);
  };

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        <Hero
          selectedCategory={selectedCategory}
          onCategorySelect={handleCategorySelect}
          searchQuery={searchQuery}
          onSearchQueryChange={setSearchQuery}
          onSubmitSearch={handleSubmitSearch}
        />
        <ArticleGrid
          initialArticles={initialArticles}
          initialHasMore={initialHasMore}
          initialTotal={initialTotal}
          selectedCategory={selectedCategory}
          searchQuery={committedQuery}
        />
      </main>
      <Footer />
    </div>
  );
}
