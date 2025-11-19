import { useState, useEffect } from "react";
import { useParams, Link } from "react-router-dom";
import { Calendar, Clock, Share2, Twitter, Linkedin, MessageCircle, Send } from "lucide-react";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ArticleCard } from "@/components/ArticleCard";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { selectArticleImage } from "@/lib/imageUtils";

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
  image?: string; // Image for feed (second category, non-repeating)
  imageCarousel?: string; // Image for carousel (first category, deterministic)
  imageArticle?: string; // Image for article detail (first category, deterministic)
  linkedinPost?: string; // LinkedIn post content from linkedin.txt
  xPost?: string; // X/Twitter post content from x.txt
}

// Map category slugs to display names (14 allowed categories only)
// Format: First letter uppercase, rest lowercase, with spaces
const categoryLabels: Record<string, string> = {
  ai: 'AI',
  coding: 'Coding',
  crypto: 'Crypto',
  data: 'Data',
  ethics: 'Ethics',
  games: 'Games',
  hardware: 'Hardware',
  legal: 'Legal',
  network: 'Network',
  quantum_computing: 'Quantum computing',
  robotics: 'Robotics',
  science: 'Science',
  security: 'Security',
  sound: 'Sound',
};

const ArticleDetail = () => {
  const { id: slug } = useParams();
  const [article, setArticle] = useState<Article | null>(null);
  const [loading, setLoading] = useState(true);
  const [relatedArticles, setRelatedArticles] = useState<Article[]>([]);

  useEffect(() => {
    async function fetchArticle() {
      if (!slug) return;
      
      try {
        // Cache por 5 minutos para artigos individuais
        const cacheKey = `scienceai-article-${slug}`;
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();
        const cacheDuration = 5 * 60 * 1000; // 5 minutos
        
        if (cached && (now - cacheTime) < cacheDuration) {
          const cachedData = JSON.parse(cached).data;
          setArticle(cachedData.article);
          setRelatedArticles(cachedData.related);
          setLoading(false);
          return;
        }
        
        const response = await fetch('/api/articles', {
          headers: {
            'Cache-Control': 'max-age=300', // 5 minutos
          },
        });
        if (!response.ok) {
          throw new Error('Failed to fetch articles');
        }

        const data = await response.json();
        const articles: Article[] = data.articles || [];

        const selected = articles.find((item: Article) => item.slug === slug);
        if (selected) {
          const related = articles
            .filter((a: Article) => a.category === selected.category && a.id !== selected.id)
            .slice(0, 3);
          
          // Salvar no cache
          sessionStorage.setItem(cacheKey, JSON.stringify({
            data: { article: selected, related },
            timestamp: now,
          }));
          
          setArticle(selected);
          setRelatedArticles(related);
        } else {
          setArticle(null);
        }
      } catch (error) {
        console.error('Error fetching article:', error);
        setArticle(null);
      } finally {
        setLoading(false);
      }
    }
    
    fetchArticle();
  }, [slug]);

  if (loading) {
    return (
      <div className="min-h-screen flex flex-col">
        <Header />
        <main className="flex-grow flex items-center justify-center">
          <div className="text-center">
            <p className="text-foreground/70">Loading article...</p>
          </div>
        </main>
        <Footer />
      </div>
    );
  }

  if (!article) {
    return (
      <div className="min-h-screen flex flex-col">
        <Header />
        <main className="flex-grow flex items-center justify-center">
          <div className="text-center">
            <h1 className="text-4xl font-bold mb-4">Article Not Found</h1>
            <Link to="/" className="text-primary hover:underline">
              Return to Home
            </Link>
          </div>
        </main>
        <Footer />
      </div>
    );
  }

  const handleShare = (platform: string) => {
    const url = window.location.href;
    const imageUrl = article.imageArticle || article.image || selectArticleImage(article.imageCategories || [], article.id);
    const fullImageUrl = imageUrl.startsWith('http') ? imageUrl : `${window.location.origin}${imageUrl}`;
    
    if (platform === "copy") {
      navigator.clipboard.writeText(url);
      toast.success("Link copied to clipboard!");
    } else if (platform === "twitter" || platform === "x") {
      // X/Twitter: conteúdo do arquivo x.txt (sem link no texto, o parâmetro url já adiciona automaticamente)
      const shareText = article.xPost || article.title;
      window.open(
        `https://twitter.com/intent/tweet?text=${encodeURIComponent(shareText)}&url=${encodeURIComponent(url)}`,
        "_blank"
      );
    } else if (platform === "linkedin") {
      // LinkedIn: conteúdo do arquivo linkedin.txt + link + preview da imagem
      const shareText = article.linkedinPost 
        ? `${article.linkedinPost}\n\n${url}`
        : article.title;
      // LinkedIn usa share-offsite que permite preview automático da imagem via Open Graph
      window.open(
        `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
        "_blank"
      );
    } else if (platform === "whatsapp") {
      // WhatsApp: título + link + preview da imagem
      const shareText = `${article.title}\n\n${url}`;
      window.open(
        `https://api.whatsapp.com/send?text=${encodeURIComponent(shareText)}`,
        "_blank"
      );
    } else if (platform === "telegram") {
      // Telegram: título + link + preview da imagem (mesmo modelo do WhatsApp)
      window.open(
        `https://t.me/share/url?url=${encodeURIComponent(url)}&text=${encodeURIComponent(article.title)}`,
        "_blank"
      );
    }
  };

  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="flex-grow">
        {/* Hero Section */}
        <div className="relative h-[500px] w-full">
          <img
            src={article.imageArticle || article.image || selectArticleImage(article.imageCategories, article.id)}
            // srcset pode ser habilitado quando houver CDN ou servidor de imagens que suporte redimensionamento
            // srcSet={/* gerar dinamicamente quando suporte disponível */}
            // sizes="(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1200px"
            alt={article.title}
            width={1200}
            height={675}
            loading="eager"
            decoding="sync"
            fetchPriority="high"
            className="w-full h-full object-cover"
            style={{ aspectRatio: '1200/675' }}
            onError={(e) => {
              // Tentar WebP primeiro, fallback para JPG se não existir
              const target = e.target as HTMLImageElement;
              if (target.src.endsWith('.webp')) {
                target.src = '/images/ai/ai_1.jpg';
              } else {
                target.src = '/images/ai/ai_1.webp';
              }
            }}
          />
          <div className="absolute inset-0 gradient-hero" />
          <div className="absolute inset-0 flex items-end">
            <div className="container mx-auto px-4 pb-12">
              <span className="inline-block px-4 py-1 bg-primary text-primary-foreground text-sm font-semibold rounded-full mb-4">
                {categoryLabels[article.category] || article.category
                  .replace(/_/g, ' ')
                  .split(' ')
                  .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
                  .join(' ')}
              </span>
              <h1 className="text-4xl md:text-5xl font-bold text-white mb-4 max-w-4xl">
                {article.title}
              </h1>
            </div>
          </div>
        </div>

        {/* Article Content */}
        <div className="container mx-auto px-4 py-12">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-12">
            {/* Main Content */}
            <article className="lg:col-span-2">
              {/* Meta Information */}
              <div className="flex flex-wrap items-center gap-6 text-sm text-foreground/75 mb-8">
                <span className="flex items-center">
                  <Calendar className="h-4 w-4 mr-2" aria-hidden="true" />
                  {new Date(article.date).toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })}
                </span>
                <span className="flex items-center">
                  <Clock className="h-4 w-4 mr-2" aria-hidden="true" />
                  {article.readTime} min read
                </span>
              </div>

              {/* Article Body */}
              <div className="prose prose-lg max-w-none">
                {article.content.split("\n\n").map((paragraph, index) => (
                  <p key={index} className="mb-4 text-foreground leading-relaxed text-justify">
                    {paragraph}
                  </p>
                ))}
              </div>

              {/* Related Articles */}
              {relatedArticles.length > 0 && (
                <section className="mt-16 pt-16 border-t">
                  <h2 className="text-2xl font-bold mb-6">Read Next</h2>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {relatedArticles.map((relatedArticle) => (
                      <ArticleCard
                        key={relatedArticle.id}
                        article={relatedArticle}
                      />
                    ))}
                  </div>
                </section>
              )}
            </article>

            {/* Sidebar */}
            <aside className="lg:col-span-1">
              <div className="sticky top-24 space-y-6">
                {/* Share Buttons */}
                <div className="bg-card rounded-xl p-6 shadow-card">
                  <div className="flex items-center mb-4">
                    <Share2 className="h-5 w-5 text-primary mr-2" />
                    <h3 className="text-lg font-bold">Share Article</h3>
                  </div>
                  <div className="space-y-3">
                    <Button
                      variant="outline"
                      className="w-full justify-start"
                      onClick={() => handleShare("twitter")}
                    >
                      <Twitter className="h-4 w-4 mr-2" />
                      Share on Twitter
                    </Button>
                    <Button
                      variant="outline"
                      className="w-full justify-start"
                      onClick={() => handleShare("linkedin")}
                    >
                      <Linkedin className="h-4 w-4 mr-2" />
                      Share on LinkedIn
                    </Button>
                    <Button
                      variant="outline"
                      className="w-full justify-start"
                      onClick={() => handleShare("whatsapp")}
                    >
                      <MessageCircle className="h-4 w-4 mr-2" />
                      Share on WhatsApp
                    </Button>
                    <Button
                      variant="outline"
                      className="w-full justify-start"
                      onClick={() => handleShare("telegram")}
                    >
                      <Send className="h-4 w-4 mr-2" />
                      Share on Telegram
                    </Button>
                    <Button
                      variant="outline"
                      className="w-full justify-start"
                      onClick={() => handleShare("copy")}
                    >
                      <Share2 className="h-4 w-4 mr-2" />
                      Copy Link
                    </Button>
                  </div>
                </div>

                {/* Author Info */}
                <div className="bg-card rounded-xl p-6 shadow-card">
                  <h3 className="text-lg font-bold mb-4">About the Author</h3>
                  <div className="flex items-start space-x-4">
                    <div className="w-16 h-16 rounded-full overflow-hidden flex-shrink-0">
                      <img
                        src="/images/Author.webp"
                        alt="Guilherme A."
                        width={80}
                        height={80}
                        loading="lazy"
                        decoding="async"
                        className="w-full h-full object-cover"
                        style={{ aspectRatio: '1/1' }}
                        onError={(e) => {
                          // Fallback to initial if image fails to load
                          const target = e.target as HTMLImageElement;
                          target.style.display = 'none';
                          const parent = target.parentElement;
                          if (parent) {
                            parent.innerHTML = '<div class="w-16 h-16 bg-primary rounded-full flex items-center justify-center text-primary-foreground text-xl font-bold">G</div>';
                          }
                        }}
                      />
                    </div>
                    <div className="flex-1">
                      <h4 className="font-semibold">Guilherme A.</h4>
                      <p className="text-sm text-foreground/75 mt-1 leading-relaxed">
                        Former dentist (MD) from Brazil, 41 years old, husband, and AI enthusiast. In 2020, he transitioned from a decade-long career in dentistry to pursue his passion for technology, entrepreneurship, and helping others grow.
                      </p>
                      <a
                        href="https://www.linkedin.com/in/guilherme-vnz/"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center mt-3 text-primary hover:text-primary/80 transition-colors text-sm font-medium"
                      >
                        <Linkedin className="h-4 w-4 mr-2" />
                        Connect on LinkedIn
                      </a>
                    </div>
                  </div>
                </div>
              </div>
            </aside>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default ArticleDetail;