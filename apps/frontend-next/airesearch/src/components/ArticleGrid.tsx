"use client";

import { useEffect, useMemo, useState } from "react";
import dynamic from "next/dynamic";
import ArticleCard from "./ArticleCard";
import { Button } from "@/components/ui/button";
import { getArticles } from "@/lib/articles";
import type { Article } from "@/types/article";

// Islands Hydration: ArticleCard abaixo da dobra carrega sem SSR
// Reduz JavaScript enviado para blocos estáticos
// Visual idêntico, mas com menos JS para hidratação
const LazyArticleCard = dynamic(() => import("./ArticleCard"), {
  ssr: false, // Não renderizar no servidor - apenas client-side
  loading: () => <div className="h-64 bg-muted animate-pulse rounded-lg" />,
});

interface ArticleGridProps {
  initialArticles: Article[];
  initialHasMore?: boolean;
  initialTotal?: number;
  selectedCategory?: string;
  searchQuery?: string;
}

const ArticleGrid = ({
  initialArticles,
  initialHasMore = false,
  initialTotal = 0,
  selectedCategory,
  searchQuery,
}: ArticleGridProps) => {
  const [articles, setArticles] = useState<Article[]>(initialArticles);
  const [hasMore, setHasMore] = useState(initialHasMore);
  const [total, setTotal] = useState(initialTotal);
  const [loading, setLoading] = useState(false);
  const [displayedCount, setDisplayedCount] = useState(6); // Mostrar apenas 6 artigos inicialmente

  // Reset quando categoria ou busca mudam
  useEffect(() => {
    setArticles(initialArticles);
    setHasMore(initialHasMore);
    setTotal(initialTotal);
    setDisplayedCount(6); // Reset para 6 artigos
  }, [selectedCategory, searchQuery, initialArticles, initialHasMore, initialTotal]);

  const loadMore = async () => {
    if (loading) return;
    
    setLoading(true);
    try {
      // Se ainda há artigos no servidor, carregar mais da API
      if (hasMore && displayedCount >= articles.length) {
        const offset = articles.length;
        const { articles: newArticles, hasMore: newHasMore, total: newTotal } = await getArticles(
          selectedCategory && selectedCategory.toLowerCase() !== "all" ? selectedCategory : undefined,
          50, // Carregar mais 50 artigos
          offset
        );
        
        setArticles(prev => [...prev, ...newArticles]);
        setHasMore(newHasMore);
        setTotal(newTotal);
      }
      // Mostrar mais 6 artigos dos que já estão carregados
      setDisplayedCount(prev => prev + 6);
    } catch (error) {
      console.error("Failed to load more articles:", error);
    } finally {
      setLoading(false);
    }
  };

  const normalize = (value: string) =>
    value
      .toLowerCase()
      .replace(/[^\w\s]/g, " ")
      .replace(/\s+/g, " ")
      .trim();

  // Normalizar categoria: converter espaços para underscores e vice-versa
  const normalizeCategory = (category: string): string => {
    return category
      .toLowerCase()
      .trim()
      .replace(/\s+/g, "_") // Converter espaços para underscores
      .replace(/-/g, "_"); // Converter hífens para underscores
  };

  // Normalizar categoria selecionada
  const normalizedCategory = selectedCategory ? normalizeCategory(selectedCategory) : null;

  const searchWords = useMemo(() => {
    if (!searchQuery || !searchQuery.trim()) {
      return [];
    }
    return normalize(searchQuery).split(" ");
  }, [searchQuery]);

  const filteredByCategory = useMemo(() => {
    if (!normalizedCategory) {
      return articles;
    }

    return articles.filter((article) => {
      // Verificar se imageCategories contém a categoria normalizada
      if (Array.isArray(article.imageCategories) && article.imageCategories.length > 0) {
        // Normalizar todas as categorias do artigo e verificar se alguma corresponde
        const normalizedArticleCategories = article.imageCategories.map(cat => normalizeCategory(cat));
        if (normalizedArticleCategories.includes(normalizedCategory)) {
          return true;
        }
      }
      
      // Fallback: verificar também o campo category do artigo
      if (article.category) {
        const normalizedArticleCategory = normalizeCategory(article.category);
        if (normalizedArticleCategory === normalizedCategory) {
          return true;
        }
      }
      
      return false;
    });
  }, [articles, normalizedCategory]);

  const prioritizedArticles = useMemo(() => {
    if (normalizedCategory || searchWords.length > 0) {
      return filteredByCategory;
    }

    return [...filteredByCategory].sort((a, b) => {
      const dateDiff =
        new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
      if (dateDiff !== 0) {
        return dateDiff;
      }

      const aFeatured = a.featured === true;
      const bFeatured = b.featured === true;
      if (aFeatured !== bFeatured) {
        return aFeatured ? -1 : 1;
      }

      return String(b.id).localeCompare(String(a.id));
    });
  }, [filteredByCategory, normalizedCategory, searchWords.length]);

  const filteredArticles = useMemo(() => {
    if (searchWords.length === 0) {
      return prioritizedArticles;
    }

    return prioritizedArticles.filter((article) => {
      const topics = Array.isArray(article.imageCategories)
        ? article.imageCategories.join(" ")
        : "";
      const haystack = normalize(
        `${article.title} ${article.id} ${article.excerpt} ${article.category ?? ""} ${topics}`,
      );
      return searchWords.every((word) => haystack.includes(word));
    });
  }, [prioritizedArticles, searchWords]);

  const displayedArticles = useMemo(
    () => filteredArticles.slice(0, displayedCount),
    [filteredArticles, displayedCount],
  );

  // Verificar se há mais artigos: do servidor OU localmente filtrados
  const hasMoreLocal = filteredArticles.length > displayedCount;

  return (
    <section className="container mx-auto px-4 py-16 section-below-fold bg-background" id="articles">
      <div className="mb-10">
        <h2 className="text-3xl md:text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
          {selectedCategory
            ? `Articles in ${selectedCategory}`
            : "Featured Articles"}
        </h2>
        <p className="text-foreground/80 text-lg font-medium">
          {selectedCategory
            ? `${filteredArticles.length} ${
                filteredArticles.length === 1
                  ? "article found"
                  : "articles found"
              }${total > filteredArticles.length ? ` (${total} total)` : ""}`
            : `Explore ${total > 0 ? `${total} ` : ""}articles on the latest research and developments in AI`}
        </p>
      </div>
      {filteredArticles.length === 0 ? (
        <div className="text-center py-16">
          <p className="text-foreground/70 text-lg">No articles found</p>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {displayedArticles.map((article, index) => (
              <div
                key={article.id}
                className="animate-fade-in-up animate-optimized"
                style={{
                  animationDelay: `${index * 150}ms`,
                  animationFillMode: "both",
                }}
              >
                {/* Islands Hydration:
                    - Primeiros 6 cards (above the fold): SSR completo para SEO e FCP
                    - Demais cards (below the fold): Client-side apenas (islands)
                    Visual idêntico, menos JS para hidratação */}
                {index < 6 ? (
                  <ArticleCard {...article} />
                ) : (
                  <LazyArticleCard {...article} />
                )}
              </div>
            ))}
          </div>

          {(hasMore || hasMoreLocal) && (
            <div className="flex justify-center mt-12">
              <Button
                onClick={loadMore}
                className="px-8 py-6 text-lg"
                variant="outline"
                disabled={loading}
              >
                {loading 
                  ? "Loading..." 
                  : hasMoreLocal
                  ? `Show More Articles (${filteredArticles.length - displayedCount} remaining)`
                  : hasMore
                  ? `Load More Articles (${total - articles.length} remaining)`
                  : "Show More Articles"}
              </Button>
            </div>
          )}
        </>
      )}
    </section>
  );
};

export default ArticleGrid;
