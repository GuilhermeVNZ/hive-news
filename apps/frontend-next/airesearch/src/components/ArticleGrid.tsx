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
  searchQuery?: string;
}

const ArticleGrid = ({
  initialArticles,
  initialHasMore = false,
  initialTotal = 0,
  searchQuery,
}: ArticleGridProps) => {
  const [articles, setArticles] = useState<Article[]>(initialArticles);
  const [hasMore, setHasMore] = useState(initialHasMore);
  const [total, setTotal] = useState(initialTotal);
  const [loading, setLoading] = useState(false);
  const [displayedCount, setDisplayedCount] = useState(6); // Mostrar apenas 6 artigos inicialmente

  // Função para recarregar artigos com nova busca
  const reloadArticles = async (query?: string) => {
    console.log('[ArticleGrid] Reloading articles with query:', query);
    setLoading(true);
    try {
      const { articles: newArticles, hasMore: newHasMore, total: newTotal } = await getArticles(
        undefined, // Sem filtro de categoria
        6, // Carregar 6 artigos iniciais
        0, // Offset 0 para nova busca
        query // Query de busca
      );
      
      console.log('[ArticleGrid] Loaded articles:', newArticles.length, 'total:', newTotal);
      setArticles(newArticles);
      setHasMore(newHasMore);
      setTotal(newTotal);
      setDisplayedCount(6);
    } catch (error) {
      console.error("Failed to reload articles:", error);
    } finally {
      setLoading(false);
    }
  };

  // Reset quando busca muda - recarregar do servidor
  useEffect(() => {
    console.log('[ArticleGrid] Search query changed:', searchQuery);
    // Se searchQuery mudou, recarregar artigos
    if (searchQuery !== undefined) {
      reloadArticles(searchQuery);
    } else {
      // Fallback para artigos iniciais se não há query
      console.log('[ArticleGrid] No query, using initial articles');
      setArticles(initialArticles);
      setHasMore(initialHasMore);
      setTotal(initialTotal);
      setDisplayedCount(6);
    }
  }, [searchQuery]);

  // Reset apenas para artigos iniciais quando eles mudam (primeira carga)
  useEffect(() => {
    if (!searchQuery) {
      setArticles(initialArticles);
      setHasMore(initialHasMore);
      setTotal(initialTotal);
      setDisplayedCount(6);
    }
  }, [searchQuery, initialArticles, initialHasMore, initialTotal]);

  const loadMore = async () => {
    if (loading) return;
    
    setLoading(true);
    try {
      // Se ainda há artigos no servidor, carregar mais da API
      if (hasMore && displayedCount >= articles.length) {
        const offset = articles.length;
        const { articles: newArticles, hasMore: newHasMore, total: newTotal } = await getArticles(
          undefined, // Sem filtro de categoria
          50, // Carregar mais 50 artigos
          offset,
          searchQuery // Incluir query de busca
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

  // Os artigos já vêm filtrados do backend, então não precisamos filtrar aqui
  const filteredArticles = useMemo(() => {
    return [...articles].sort((a, b) => {
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
  }, [articles]);

  const displayedArticles = useMemo(
    () => filteredArticles.slice(0, displayedCount),
    [filteredArticles, displayedCount],
  );

  // Verificar se há mais artigos: do servidor OU localmente filtrados
  const hasMoreLocal = filteredArticles.length > displayedCount;

  return (
    <section className="container mx-auto px-4 py-3 section-below-fold bg-background" id="articles">
      <div className="mb-10">
        <h2 className="text-3xl md:text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
          {searchQuery && searchQuery.trim() 
            ? `Search Results` 
            : "Featured Articles"}
        </h2>
        <p className="text-foreground/80 text-lg font-medium">
          {searchQuery && searchQuery.trim()
            ? `${filteredArticles.length} ${
                filteredArticles.length === 1
                  ? "article found"
                  : "articles found"
              } for "${searchQuery}"`
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
