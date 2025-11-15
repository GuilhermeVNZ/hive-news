import { memo } from "react";
import { Link } from "react-router-dom";
import { Calendar, Clock } from "lucide-react";
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

interface ArticleCardProps {
  article: Article;
  priority?: boolean; // Para artigos above the fold
}

export const ArticleCard = memo(({ article, priority = false }: ArticleCardProps) => {
  // Use image from server if available (selected using AIResearch logic)
  // Otherwise fallback to client-side selection
  const imageUrl = article.image || selectArticleImage(article.imageCategories, article.id);
  
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

  // Use first category from imageCategories if available, otherwise fallback to category
  const primaryCategory = article.imageCategories && article.imageCategories.length > 0
    ? article.imageCategories[0]
    : article.category;
  const categoryName = getCategoryName(primaryCategory);

  return (
    <Link
      to={`/article/${article.slug}`}
      className="group block bg-card rounded-xl overflow-hidden shadow-card hover:shadow-hover transition-smooth"
    >
      <div className="relative h-48 overflow-hidden">
        <img
          src={imageUrl}
          srcSet={priority ? `${imageUrl}?w=400 400w, ${imageUrl}?w=640 640w, ${imageUrl}?w=800 800w` : undefined}
          sizes={priority ? "(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 400px" : undefined}
          alt={article.title}
          width={400}
          height={192}
          loading={priority ? "eager" : "lazy"}
          decoding={priority ? "sync" : "async"}
          fetchPriority={priority ? "high" : "auto"}
          className="w-full h-full object-cover transition-smooth group-hover:scale-105"
          style={{ aspectRatio: '400/192' }}
          onError={(e) => {
            // Fallback to default image if selected image doesn't exist
            (e.target as HTMLImageElement).src = '/images/ai/ai_1.jpg';
          }}
        />
        <div className="absolute top-4 left-4">
          <span className="inline-block px-3 py-1 bg-primary text-primary-foreground text-xs font-semibold rounded-full">
            {categoryName}
          </span>
        </div>
      </div>

      <div className="p-6">
        <h3 className="text-xl font-bold mb-3 group-hover:text-primary transition-smooth line-clamp-2">
          {article.title}
        </h3>
        <p className="text-foreground/80 text-sm mb-4 line-clamp-3 text-justify">
          {article.excerpt}
        </p>

        <div className="flex items-center justify-between text-xs text-foreground/70">
          <div className="flex items-center space-x-4">
            <span className="flex items-center">
              <Calendar className="h-4 w-4 mr-1" aria-hidden="true" />
              {new Date(article.date).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
              })}
            </span>
            <span className="flex items-center">
              <Clock className="h-4 w-4 mr-1" aria-hidden="true" />
              {article.readTime} min read
            </span>
          </div>
          <span className="text-primary font-medium group-hover:underline">
            Read more →
          </span>
        </div>
      </div>
    </Link>
  );
});

ArticleCard.displayName = 'ArticleCard';
