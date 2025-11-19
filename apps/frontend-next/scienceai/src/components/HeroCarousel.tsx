import { useState, useEffect, memo, useMemo } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { selectArticleImage } from "@/lib/imageUtils";
import { ImagePreload } from "@/components/ImagePreload";

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

// Format category name: use label if available, otherwise format with spaces
const getCategoryName = (category: string): string => {
  if (categoryLabels[category]) {
    return categoryLabels[category];
  }
  // Fallback: replace underscores with spaces and capitalize first letter
  return category
    .replace(/_/g, ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
};

export const HeroCarousel = memo(({ articles, categories }: HeroCarouselProps) => {
  const [currentSlide, setCurrentSlide] = useState(0);
  // CRÍTICO: Primeira imagem deve ser carregada imediatamente (LCP)
  const [loadedImages, setLoadedImages] = useState<Set<number>>(new Set([0]));
  
  // Memoizar artigos do carousel para evitar recálculos
  // CRITICAL: Limitar a máximo 2 notícias por categoria para evitar repetição
  const finalCarouselArticles = useMemo(() => {
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
    
    // Limitar a máximo 2 artigos por categoria
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
    
    return limitedArticles;
  }, [articles]);
  
  // Preload da próxima imagem quando o slide atual muda
  useEffect(() => {
    if (finalCarouselArticles.length === 0) {
      return;
    }
    
    // Preload da próxima imagem apenas quando necessário
    const nextIndex = (currentSlide + 1) % finalCarouselArticles.length;
    if (!loadedImages.has(nextIndex) && finalCarouselArticles[nextIndex]) {
      const nextArticle = finalCarouselArticles[nextIndex];
      const nextImageUrl = nextArticle.imageCarousel || nextArticle.image || selectArticleImage(nextArticle.imageCategories, nextArticle.id);
      if (nextImageUrl) {
        const img = new Image();
        img.src = nextImageUrl;
        img.onload = () => {
          setLoadedImages(prev => new Set([...prev, nextIndex]));
        };
        img.onerror = () => {
          // Se falhar, marcar como carregado para evitar tentativas infinitas
          setLoadedImages(prev => new Set([...prev, nextIndex]));
        };
      }
    }
    
    const timer = setInterval(() => {
      setCurrentSlide((prev) => (prev + 1) % finalCarouselArticles.length);
    }, 5000);
    return () => clearInterval(timer);
  }, [finalCarouselArticles.length, currentSlide]);

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

  // Primeira imagem é a LCP - fazer preload
  const firstArticle = finalCarouselArticles[0];
  const firstImageUrl = firstArticle 
    ? (firstArticle.imageCarousel || firstArticle.image || selectArticleImage(firstArticle.imageCategories, firstArticle.id))
    : null;

  return (
    <section className="relative h-[600px] w-full overflow-hidden rounded-xl">
      {/* Preload da imagem LCP (primeira imagem do carousel) */}
      {firstImageUrl && <ImagePreload imageUrl={firstImageUrl} />}
      
      {finalCarouselArticles.map((article, index) => {
        const imageUrl = article.imageCarousel || article.image || selectArticleImage(article.imageCategories, article.id);
        const isVisible = index === currentSlide;
        // CRÍTICO: Primeira imagem (LCP) sempre deve carregar imediatamente, outras apenas se visíveis ou já carregadas
        const shouldLoad = index === 0 || loadedImages.has(index) || isVisible;
        
        // Criar srcset para diferentes tamanhos (apenas para primeira imagem/LCP)
        // Nota: srcset requer que o servidor suporte redimensionamento de imagens
        // Por enquanto, manter apenas src se não houver suporte a redimensionamento
        const baseImageUrl = imageUrl.startsWith('http') || imageUrl.startsWith('/') 
          ? imageUrl 
          : `/${imageUrl}`;
        // srcset pode ser adicionado quando houver CDN ou servidor de imagens que suporte redimensionamento
        const srcset = index === 0 && baseImageUrl.includes('?')
          ? `${baseImageUrl}&w=640 640w, ${baseImageUrl}&w=750 750w, ${baseImageUrl}&w=828 828w, ${baseImageUrl}&w=1080 1080w, ${baseImageUrl}&w=1200 1200w, ${baseImageUrl}&w=1920 1920w`
          : undefined;
        
        return (
          <div
            key={article.id}
            className={`absolute inset-0 transition-opacity duration-1000 ${
              isVisible ? "opacity-100 z-10" : "opacity-0 z-0"
            }`}
          >
            {shouldLoad ? (
              <img
                src={imageUrl}
                srcSet={srcset}
                sizes={index === 0 ? "(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1920px" : undefined}
                alt={article.title}
                width={1920}
                height={600}
                loading={index === 0 ? "eager" : "lazy"}
                decoding={index === 0 ? "sync" : "async"}
                fetchPriority={index === 0 ? "high" : "auto"}
                className="h-full w-full object-cover"
                style={{ aspectRatio: '1920/600' }}
                onError={(e) => {
                  (e.target as HTMLImageElement).src = '/images/ai/ai_1.jpg';
                }}
                onLoad={() => {
                  if (!loadedImages.has(index)) {
                    setLoadedImages(prev => new Set([...prev, index]));
                  }
                }}
              />
            ) : (
              // Placeholder enquanto não carrega
              <div className="h-full w-full bg-gradient-to-br from-primary/20 via-primary/10 to-background animate-pulse" />
            )}
            <div className="absolute inset-0 gradient-hero" />
          <div className="absolute inset-0 flex items-end">
            <div className="container mx-auto px-4 pb-12">
              <div className="max-w-3xl">
                <span className="inline-block px-4 py-1 bg-primary text-primary-foreground text-sm font-semibold rounded-full mb-4">
                  {getCategoryName(article.category)}
                </span>
                <h2 className="text-4xl md:text-5xl font-bold text-white mb-4">
                  {article.title}
                </h2>
                <p className="text-lg text-white/90 mb-6 text-justify">{article.excerpt}</p>
                {/* CRÍTICO: Desabilitar pointer events em slides não visíveis para evitar cliques errados */}
                <div style={{ pointerEvents: isVisible ? 'auto' : 'none' }}>
                  <Link to={`/article/${article.slug}`}>
                    <Button size="lg" className="gradient-primary">
                      Read Full Story
                    </Button>
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </div>
        );
      })}

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