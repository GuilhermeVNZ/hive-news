import { useState, useEffect, lazy, Suspense } from "react";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ArticleCard } from "@/components/ArticleCard";

// HeroCarousel NÃO pode ser lazy loaded - contém a imagem LCP!
import { HeroCarousel } from "@/components/HeroCarousel";

// Lazy load do Sidebar para melhorar performance inicial
const LazySidebar = lazy(() => import("@/components/Sidebar").then(module => ({ default: module.Sidebar })));

interface Category {
  name: string;
  slug: string;
  icon: string;
  latestDate?: string;
}

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
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);

  // Buscar categorias dinâmicas (top 5 mais recentes)
  useEffect(() => {
    async function fetchCategories() {
      try {
        // Cache por 5 minutos para categorias
        const cacheKey = 'scienceai-categories';
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();
        const cacheDuration = 5 * 60 * 1000; // 5 minutos
        
        if (cached && (now - cacheTime) < cacheDuration) {
          const data = JSON.parse(cached).data;
          const sortedCategories = (data.categories || []).sort((a: Category, b: Category) => {
            if (!a.latestDate || !b.latestDate) return 0;
            return new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime();
          });
          
          // CRÍTICO: Garantir que não há duplicatas por slug
          const seenSlugs = new Set<string>();
          const uniqueCategories = sortedCategories.filter((cat) => {
            const slug = cat.slug.toLowerCase().trim();
            if (seenSlugs.has(slug)) {
              return false;
            }
            seenSlugs.add(slug);
            return true;
          });
          
          setCategories(uniqueCategories.slice(0, 5));
          return;
        }
        
        const response = await fetch('/api/categories', {
          headers: {
            'Cache-Control': 'max-age=300', // 5 minutos
          },
        });
        const data = await response.json();
        
        // Salvar no cache
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data,
          timestamp: now,
        }));
        
        // Ordenar da esquerda para direita (mais recente primeiro)
        const sortedCategories = (data.categories || []).sort((a: Category, b: Category) => {
          if (!a.latestDate || !b.latestDate) return 0;
          return new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime();
        });
        
        // CRÍTICO: Garantir que não há duplicatas por slug
        const seenSlugs = new Set<string>();
        const uniqueCategories = sortedCategories.filter((cat) => {
          const slug = cat.slug.toLowerCase().trim();
          if (seenSlugs.has(slug)) {
            return false;
          }
          seenSlugs.add(slug);
          return true;
        });
        
        setCategories(uniqueCategories.slice(0, 5)); // Máximo 5 categorias
      } catch (error) {
        console.error('Error fetching categories:', error);
        setCategories([]);
      }
    }
    fetchCategories();
  }, []);

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

  // Articles are already sorted by backend (featured first), but ensure they stay that way
  let filteredArticles = searchQuery
    ? articles.filter(
        (article) =>
          article.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          article.excerpt.toLowerCase().includes(searchQuery.toLowerCase()) ||
          article.category.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : articles;
  
  // If no search query, ensure featured articles are at the top
  if (!searchQuery) {
    filteredArticles = [...filteredArticles].sort((a, b) => {
      const dateDiff =
        new Date(b.date).getTime() - new Date(a.date).getTime();
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
  }

  const getArticlesByCategory = (categorySlug: string) => {
    return filteredArticles.filter(
      (article) => article.category.toLowerCase() === categorySlug.toLowerCase()
    );
  };

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
              <HeroCarousel articles={filteredArticles} categories={categories} />

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
                    {filteredArticles.length > 0 ? (
                      filteredArticles.map((article, index) => (
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
                // CRÍTICO: Garantir que não há categorias duplicadas no feed
                (() => {
                  const seenCategorySlugs = new Set<string>();
                  return categories
                    .filter((category) => {
                      const slug = category.slug.toLowerCase().trim();
                      if (seenCategorySlugs.has(slug)) {
                        console.warn(
                          `[Index] ⚠️ Duplicate category filtered out: ${slug}`,
                        );
                        return false;
                      }
                      seenCategorySlugs.add(slug);
                      return true;
                    })
                    .map((category) => {
                      const categoryArticles = getArticlesByCategory(category.slug);
                      if (categoryArticles.length === 0) return null;

                      // Garantir máximo de 4 artigos por categoria na página principal
                      const displayedArticles = categoryArticles.slice(0, 4);

                      return (
                        <section key={category.slug}>
                          <div className="flex items-center justify-between mb-6">
                            <h2 className="text-2xl font-bold">{category.name}</h2>
                            <a
                              href={`/category/${category.slug}`}
                              className="text-primary hover:underline text-sm font-medium"
                            >
                              View all →
                            </a>
                          </div>
                          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            {displayedArticles.map((article, index) => (
                              <ArticleCard 
                                key={article.id} 
                                article={article}
                                // Priorizar carregamento dos primeiros 4 artigos (above the fold)
                                priority={index < 4}
                              />
                            ))}
                          </div>
                        </section>
                      );
                    });
                })()
              )}
            </div>

            {/* Sidebar */}
            <div className="lg:col-span-1">
              <div className="sticky top-24">
                <Suspense fallback={<div className="h-64 bg-muted animate-pulse rounded-lg" />}>
                  <LazySidebar articles={filteredArticles} />
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
