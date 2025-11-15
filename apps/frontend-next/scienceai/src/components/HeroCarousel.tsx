import { useState, useEffect, memo } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
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
  featured?: boolean; // Featured status from registry
}

interface Category {
  name: string;
  slug: string;
  icon: string;
  latestDate?: string;
}

interface HeroCarouselProps {
  articles: Article[];
  categories: Category[]; // Top 5 categories from trending topics
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

export const HeroCarousel = memo(({ articles, categories }: HeroCarouselProps) => {
  const [currentSlide, setCurrentSlide] = useState(0);
  
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

  const finalCarouselArticles = sortedArticles.slice(0, 5);
  
  useEffect(() => {
    if (finalCarouselArticles.length === 0) {
      return;
    }
    const timer = setInterval(() => {
      setCurrentSlide((prev) => (prev + 1) % finalCarouselArticles.length);
    }, 5000);
    return () => clearInterval(timer);
  }, [finalCarouselArticles.length]);

  const nextSlide = () => {
    setCurrentSlide((prev) => (prev + 1) % finalCarouselArticles.length);
  };

  const prevSlide = () => {
    setCurrentSlide(
      (prev) => (prev - 1 + finalCarouselArticles.length) % finalCarouselArticles.length
    );
  };

  // Se não tiver nenhuma notícia, não renderizar carrossel
  if (finalCarouselArticles.length === 0) return null;

  return (
    <section className="relative h-[600px] w-full overflow-hidden rounded-xl">
      {finalCarouselArticles.map((article, index) => (
        <div
          key={article.id}
          className={`absolute inset-0 transition-opacity duration-1000 ${
            index === currentSlide ? "opacity-100" : "opacity-0"
          }`}
        >
          <img
            src={article.imageCarousel || article.image || selectArticleImage(article.imageCategories, article.id)}
            alt={article.title}
            width={1920}
            height={600}
            loading={index === 0 ? "eager" : "lazy"}
            decoding={index === 0 ? "sync" : "async"}
            className="h-full w-full object-cover"
            style={{ aspectRatio: '1920/600' }}
            onError={(e) => {
              (e.target as HTMLImageElement).src = '/images/ai/ai_1.jpg';
            }}
          />
          <div className="absolute inset-0 gradient-hero" />
          <div className="absolute inset-0 flex items-end">
            <div className="container mx-auto px-4 pb-12">
              <div className="max-w-3xl">
                <span className="inline-block px-4 py-1 bg-primary text-primary-foreground text-sm font-semibold rounded-full mb-4">
                  {categoryLabels[article.category] || article.category.toUpperCase()}
                </span>
                <h2 className="text-4xl md:text-5xl font-bold text-white mb-4">
                  {article.title}
                </h2>
                <p className="text-lg text-white/90 mb-6 text-justify">{article.excerpt}</p>
                <Link to={`/article/${article.slug}`}>
                  <Button size="lg" className="gradient-primary">
                    Read Full Story
                  </Button>
                </Link>
              </div>
            </div>
          </div>
        </div>
      ))}

      {/* Navigation Buttons */}
      <Button
        variant="ghost"
        size="icon"
        className="absolute left-4 top-1/2 -translate-y-1/2 bg-white/20 hover:bg-white/30 text-white"
        onClick={prevSlide}
      >
        <ChevronLeft className="h-6 w-6" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="absolute right-4 top-1/2 -translate-y-1/2 bg-white/20 hover:bg-white/30 text-white"
        onClick={nextSlide}
      >
        <ChevronRight className="h-6 w-6" />
      </Button>

      {/* Indicators */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex space-x-2">
        {finalCarouselArticles.map((_, index) => (
          <button
            key={index}
            onClick={() => setCurrentSlide(index)}
            className={`h-2 rounded-full transition-all ${
              index === currentSlide
                ? "w-8 bg-primary"
                : "w-2 bg-white/50 hover:bg-white/70"
            }`}
            aria-label={`Go to slide ${index + 1}`}
          />
        ))}
      </div>
    </section>
  );
});

HeroCarousel.displayName = 'HeroCarousel';