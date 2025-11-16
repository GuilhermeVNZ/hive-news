import { useState, useEffect, lazy, Suspense, useMemo, useCallback } from "react";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ArticleCard } from "@/components/ArticleCard";
import { useCategories } from "@/hooks/useCategories";
import { useInfiniteScroll } from "@/hooks/useInfiniteScroll";

// HeroCarousel NÃO pode ser lazy loaded - contém a imagem LCP!
import { HeroCarousel } from "@/components/HeroCarousel";

// Lazy load do Sidebar para melhorar performance inicial
const LazySidebar = lazy(() => import("@/components/Sidebar").then(module => ({ default: module.Sidebar })));

interface Article {
  id: string;
  slug: string;
  title: string;
  category: string;
  excerpt: string;
  content: string;
  date: string;
  author: string;
  readTime: number;
  imageCategories?: string[];
  image?: string; // Image path selected by server (using AIResearch logic)
  featured?: boolean; // Featured status from registry
}

const Index = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [articles, setArticles] = useState<Article[]>([]);
  const [loading, setLoading] = useState(true);
  const [displayedCount, setDisplayedCount] = useState(4); // Inicial: 4 artigos

  // Usar hook compartilhado para categorias (evita duplicação)
  const { categories } = useCategories({
    cacheKey: "scienceai-categories",
    maxCategories: 5,
  });

  // Buscar artigos
  useEffect(() => {
    async function fetchArticles() {
      try {
        // Cache por 60 segundos para artigos (reduz latência de ~640ms)
        const cacheKey = 'scienceai-articles';
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();
        const cacheDuration = 60 * 1000; // 60 segundos
        
        if (cached && (now - cacheTime) < cacheDuration) {
          const articles = JSON.parse(cached).data;
          setArticles(articles);
          setLoading(false);
          return;
        }
        
        const response = await fetch('/api/articles', {
          headers: {
            'Cache-Control': 'max-age=60', // 60 segundos
          },
        });
        const data = await response.json();
        const articles = data.articles || [];
        
        // Salvar no cache
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data: articles,
          timestamp: now,
        }));
        
        // Debug: Log featured articles (apenas em dev)
        if (import.meta.env.DEV) {
          const featuredArticles = articles.filter((a: Article) => a.featured === true);
          console.log(`[ScienceAI Index] Fetched ${articles.length} articles, ${featuredArticles.length} featured:`, 
            featuredArticles.map((a: Article) => ({ id: a.id, title: a.title.substring(0, 50), featured: a.featured })));
        }
        
        setArticles(articles);
      } catch (error) {
        console.error('Error fetching articles:', error);
      } finally {
        setLoading(false);
      }
    }
    
    fetchArticles();
  }, []);

  // Memoizar artigos do carrossel para excluir do feed
  const carouselArticles = useMemo(() => {
    // Mesma lógica do HeroCarousel: máximo 5 artigos, 2 por categoria
    const sortedArticles = [...articles].sort((a, b) => {
      const dateDiff = new Date(b.date).getTime() - new Date(a.date).getTime();
      if (dateDiff !== 0) {
        return dateDiff;
      }
      const aFeatured = a.featured === true;
      const bFeatured = b.featured === true;
      if (aFeatured !== bFeatured) {
        return aFeatured ? -1 : 1;
      }
      return b.id.localeCompare(a.id);
    });
    
    const categoryCounts = new Map<string, number>();
    const limitedArticles: Article[] = [];
    
    for (const article of sortedArticles) {
      const category = article.category.toLowerCase();
      const count = categoryCounts.get(category) || 0;
      
      if (count < 2) {
        categoryCounts.set(category, count + 1);
        limitedArticles.push(article);
        
        if (limitedArticles.length >= 5) {
          break;
        }
      }
    }
    
    return limitedArticles.map(a => a.id);
  }, [articles]);

  // Filtrar e preparar artigos para o feed (excluindo os do carrossel)
  const feedArticles = useMemo(() => {
    let filtered = searchQuery
      ? articles.filter(
          (article) =>
            article.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
            article.excerpt.toLowerCase().includes(searchQuery.toLowerCase()) ||
            article.category.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : articles;

    // Excluir artigos do carrossel
    filtered = filtered.filter(article => !carouselArticles.includes(article.id));

    // Ordenar por data (mais novo primeiro)
    filtered = [...filtered].sort((a, b) => {
      const dateDiff = new Date(b.date).getTime() - new Date(a.date).getTime();
      if (dateDiff !== 0) {
        return dateDiff;
      }
      return b.id.localeCompare(a.id);
    });

    // Agrupar por categoria e limitar a 2 por categoria
    const categoryCounts = new Map<string, number>();
    const categoryLimitedArticles: Article[] = [];

    for (const article of filtered) {
      const category = article.category.toLowerCase();
      const count = categoryCounts.get(category) || 0;
      
      if (count < 2) {
        categoryCounts.set(category, count + 1);
        categoryLimitedArticles.push(article);
      }
    }

    return categoryLimitedArticles;
  }, [articles, searchQuery, carouselArticles]);

  // Limite máximo de artigos para garantir acesso ao footer (20 artigos = ~4-5 telas)
  const MAX_ARTICLES = 20;

  // Artigos a serem exibidos (limitados pelo scroll infinito e máximo)
  const displayedArticles = useMemo(() => {
    const limitedCount = Math.min(displayedCount, MAX_ARTICLES);
    return feedArticles.slice(0, limitedCount);
  }, [feedArticles, displayedCount]);

  // Verificar se há mais artigos para carregar (respeitando limite máximo)
  const hasMore = displayedCount < feedArticles.length && displayedCount < MAX_ARTICLES;

  // Handler para carregar mais artigos (scroll infinito)
  const handleLoadMore = useCallback(() => {
    if (!loading && hasMore && displayedCount < MAX_ARTICLES) {
      // Carregar mais 4 de cada vez, mas não ultrapassar o limite
      setDisplayedCount(prev => Math.min(prev + 4, MAX_ARTICLES));
    }
  }, [loading, hasMore, displayedCount]);

  // Hook de scroll infinito (threshold menor para garantir espaço para footer)
  const { loadMoreRef } = useInfiniteScroll({
    onLoadMore: handleLoadMore,
    hasMore,
    isLoading: loading,
    threshold: 200, // Trigger mais cedo (200px) para não bloquear footer
  });

  return (
    <div className="min-h-screen flex flex-col">
      <Header onSearch={setSearchQuery} />

      <main className="flex-grow">
        <div className="container mx-auto px-4 py-8">
          {loading ? (
            <div className="flex justify-center items-center h-64">
              <p className="text-muted-foreground">Loading articles...</p>
            </div>
          ) : (
            <>
              {/* Hero Carousel - CRÍTICO: Não lazy load pois contém imagem LCP */}
              <HeroCarousel articles={articles} categories={categories} />

              {/* Main Content Grid */}
              <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 mt-12">
                {/* Articles Section */}
                <div className="lg:col-span-2 space-y-12">
                  {searchQuery ? (
                    <section>
                      <h2 className="text-2xl font-bold mb-6">
                        Search Results for "{searchQuery}"
                      </h2>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        {feedArticles.length > 0 ? (
                          feedArticles.map((article, index) => (
                            <ArticleCard 
                              key={article.id} 
                              article={article}
                              priority={index < 4} // Primeiros 4 artigos acima da dobra
                            />
                          ))
                        ) : (
                          <p className="text-muted-foreground col-span-2">
                            No articles found matching your search.
                          </p>
                        )}
                      </div>
                    </section>
                  ) : (
                    <section>
                      <h2 className="text-2xl font-bold mb-6">Latest Articles</h2>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        {displayedArticles.length > 0 ? (
                          <>
                            {displayedArticles.map((article, index) => (
                              <ArticleCard 
                                key={article.id} 
                                article={article}
                                priority={index < 4} // Primeiros 4 artigos acima da dobra
                              />
                            ))}
                            {/* Trigger para scroll infinito */}
                            {hasMore && (
                              <div 
                                ref={loadMoreRef} 
                                className="col-span-2 flex justify-center items-center py-8"
                              >
                                <div className="text-center">
                                  <div className="h-8 w-8 border-4 border-primary border-t-transparent rounded-full animate-spin mx-auto mb-2" />
                                  <p className="text-sm text-muted-foreground">Loading more articles...</p>
                                </div>
                              </div>
                            )}
                            {/* Mensagem de fim (quando todos foram carregados ou limite atingido) */}
                            {(!hasMore || displayedArticles.length >= MAX_ARTICLES) && displayedArticles.length > 4 && (
                              <div className="col-span-2 text-center py-8">
                                <p className="text-sm text-muted-foreground">
                                  {displayedArticles.length >= MAX_ARTICLES 
                                    ? `Showing ${displayedArticles.length} latest articles. Scroll down to see the footer.`
                                    : "You've reached the end!"
                                  }
                                </p>
                              </div>
                            )}
                            {/* Espaçamento extra antes do footer para garantir acesso */}
                            {(!hasMore || displayedArticles.length >= MAX_ARTICLES) && (
                              <div className="col-span-2 h-16" aria-hidden="true" />
                            )}
                          </>
                        ) : (
                          <p className="text-muted-foreground col-span-2">
                            No articles available.
                          </p>
                        )}
                      </div>
                    </section>
                  )}
                </div>

                {/* Sidebar */}
                <div className="lg:col-span-1">
                  <div className="sticky top-24">
                    <Suspense fallback={<div className="h-64 bg-muted animate-pulse rounded-lg" />}>
                      <LazySidebar articles={articles} />
                    </Suspense>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default Index;
