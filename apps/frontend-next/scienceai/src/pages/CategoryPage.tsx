import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ArticleCard } from "@/components/ArticleCard";
import { Sidebar } from "@/components/Sidebar";

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
}

// Mapeamento de slugs para nomes (fallback quando categoria não está nas top 5)
const categoryNames: Record<string, string> = {
  'nvidia': 'NVIDIA',
  'openai': 'OpenAI',
  'google': 'Google',
  'anthropic': 'Anthropic',
  'deepseek': 'DeepSeek',
  'meta': 'Meta',
  'x': 'X',
  'mistral': 'Mistral',
  'alibaba': 'Alibaba',
  'microsoft': 'Microsoft',
  'hivehub': 'HiveHub',
  'perplexity': 'Perplexity',
  'huggingface': 'Hugging Face',
  'stability': 'Stability AI',
  'elevenlabs': 'ElevenLabs',
  'character': 'Character.AI',
  'inflection': 'Inflection AI',
  'ibm': 'IBM Research',
  'apple': 'Apple ML',
  'intel': 'Intel AI',
  'amd': 'AMD AI',
  'salesforce': 'Salesforce AI',
  'stanford': 'Stanford AI',
  'berkeley': 'Berkeley AI',
  'deepmind': 'DeepMind',
  'techcrunch': 'TechCrunch',
  'venturebeat': 'VentureBeat',
  'verge': 'The Verge',
  'wired': 'Wired',
  'mit': 'MIT Tech Review',
  'nature': 'Nature',
  'science': 'Science',
  'menlo': 'Menlo Ventures',
  'unknown': 'Technology',
  'technology': 'Technology',
};

const CategoryPage = () => {
  const { category } = useParams();
  const [articles, setArticles] = useState<Article[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Buscar categorias dinâmicas para obter nome correto (com cache)
  useEffect(() => {
    async function fetchCategories() {
      try {
        const cacheKey = 'scienceai-category-page-categories';
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();
        const cacheDuration = 5 * 60 * 1000; // 5 minutos
        
        if (cached && (now - cacheTime) < cacheDuration) {
          const data = JSON.parse(cached).data;
          setCategories(data.categories || []);
          return;
        }
        
        const response = await fetch('/api/categories', {
          headers: { 'Cache-Control': 'max-age=300' },
        });
        const data = await response.json();
        
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data,
          timestamp: now,
        }));
        
        setCategories(data.categories || []);
      } catch (error) {
        console.error('Error fetching categories:', error);
      }
    }
    fetchCategories();
  }, []);
  
  const categoryInfo = categories.find((c) => c.slug === category);
  const categoryName = categoryInfo?.name || categoryNames[category?.toLowerCase() || ''] || category?.charAt(0).toUpperCase() + category?.slice(1) || 'Category';
  
  useEffect(() => {
    async function fetchArticles() {
      try {
        // Cache por 60 segundos
        const cacheKey = 'scienceai-category-page-articles';
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
          headers: { 'Cache-Control': 'max-age=60' },
        });
        const data = await response.json();
        const articles = data.articles || [];
        
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data: articles,
          timestamp: now,
        }));
        
        setArticles(articles);
      } catch (error) {
        console.error('Error fetching articles:', error);
      } finally {
        setLoading(false);
      }
    }
    
    fetchArticles();
  }, []);
  
  const categoryArticles = articles.filter(
    (article) => article.category.toLowerCase() === category?.toLowerCase()
  );

  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="flex-grow">
        <div className="container mx-auto px-4 py-12">
          <div className="mb-12">
            <h1 className="text-4xl font-bold mb-4">
              {categoryName}
            </h1>
            <p className="text-muted-foreground text-lg">
              Latest articles in {categoryName}
            </p>
          </div>

          {loading ? (
            <div className="flex justify-center items-center h-64">
              <p className="text-muted-foreground">Loading articles...</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
              <div className="lg:col-span-2">
                {categoryArticles.length > 0 ? (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {categoryArticles.map((article) => (
                      <ArticleCard key={article.id} article={article} />
                    ))}
                  </div>
                ) : (
                  <p className="text-muted-foreground">
                    No articles found in this category.
                  </p>
                )}
              </div>

              <div className="lg:col-span-1">
                <div className="sticky top-24">
                  <Sidebar articles={articles} />
                </div>
              </div>
            </div>
          )}
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default CategoryPage;
