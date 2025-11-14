import { useState, useEffect } from "react";
import { useParams, Link } from "react-router-dom";
import { Calendar, Clock, Share2, Twitter, Linkedin } from "lucide-react";
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
}

const categoryLabels: Record<string, string> = {
  nvidia: 'NVIDIA',
  openai: 'OpenAI',
  google: 'Google',
  anthropic: 'Anthropic',
  deepseek: 'DeepSeek',
  meta: 'Meta',
  x: 'X',
  mistral: 'Mistral',
  alibaba: 'Alibaba',
  microsoft: 'Microsoft',
  hivehub: 'HiveHub',
  unknown: 'Technology',
  technology: 'Technology',
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
        const response = await fetch('/api/articles');
        if (!response.ok) {
          throw new Error('Failed to fetch articles');
        }

        const data = await response.json();
        const articles: Article[] = data.articles || [];

        const selected = articles.find((item: Article) => item.slug === slug);
        if (selected) {
          setArticle(selected);
          const related = articles
            .filter((a: Article) => a.category === selected.category && a.id !== selected.id)
            .slice(0, 3);
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
            <p className="text-muted-foreground">Loading article...</p>
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
    const text = article.title;
    
    if (platform === "copy") {
      navigator.clipboard.writeText(url);
      toast.success("Link copied to clipboard!");
    } else if (platform === "twitter") {
      window.open(
        `https://twitter.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(url)}`,
        "_blank"
      );
    } else if (platform === "linkedin") {
      window.open(
        `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
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
            alt={article.title}
            className="w-full h-full object-cover"
            onError={(e) => {
              (e.target as HTMLImageElement).src = '/images/ai/ai_1.jpg';
            }}
          />
          <div className="absolute inset-0 gradient-hero" />
          <div className="absolute inset-0 flex items-end">
            <div className="container mx-auto px-4 pb-12">
              <span className="inline-block px-4 py-1 bg-primary text-primary-foreground text-sm font-semibold rounded-full mb-4">
                {categoryLabels[article.category] || article.category.toUpperCase()}
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
              <div className="flex flex-wrap items-center gap-6 text-sm text-muted-foreground mb-8">
                <span className="flex items-center">
                  <Calendar className="h-4 w-4 mr-2" />
                  {new Date(article.date).toLocaleDateString("en-US", {
                    year: "numeric",
                    month: "long",
                    day: "numeric",
                  })}
                </span>
                <span className="flex items-center">
                  <Clock className="h-4 w-4 mr-2" />
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
                        src="/images/author.jpeg"
                        alt="Guilherme A."
                        className="w-full h-full object-cover"
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
                      <p className="text-sm text-muted-foreground mt-1 leading-relaxed">
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