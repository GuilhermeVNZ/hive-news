"use client";

import { useEffect, useState } from "react";
import dynamic from "next/dynamic";
import { useRouter } from "next/navigation";
import Header from "@/components/Header";
import Hero from "@/components/Hero";
import ArticleGrid from "@/components/ArticleGrid";
import type { Article } from "@/types/article";

// Lazy load do Footer para melhorar performance inicial
// O Footer não é crítico para o FCP
const Footer = dynamic(() => import("@/components/Footer"), {
  ssr: true, // Manter SSR para SEO
});

interface HomeClientProps {
  initialArticles: Article[];
  initialCategory?: string;
  initialQuery?: string;
}

export default function HomeClient({
  initialArticles,
  initialCategory = "",
  initialQuery = "",
}: HomeClientProps) {
  const router = useRouter();
  const [selectedCategory, setSelectedCategory] =
    useState<string>(initialCategory);
  const [searchQuery, setSearchQuery] = useState<string>(initialQuery);
  const [committedQuery, setCommittedQuery] = useState<string>(initialQuery);
  
  // Prefetch de rotas prováveis quando o mouse está sobre links
  useEffect(() => {
    const handleMouseEnter = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      const link = target.closest('a');
      if (link && link.href) {
        const url = new URL(link.href);
        if (url.pathname.startsWith('/article/')) {
          router.prefetch(url.pathname);
        }
      }
    };
    
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
          articles={initialArticles}
          selectedCategory={selectedCategory}
          searchQuery={committedQuery}
        />
      </main>
      <Footer />
    </div>
  );
}
