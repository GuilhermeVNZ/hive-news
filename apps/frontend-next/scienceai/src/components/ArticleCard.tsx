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
  
  // Map category slugs to display names (todas as possibilidades)
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

  const categoryName = categoryLabels[article.category] || article.category.toUpperCase();

  return (
    <Link
      to={`/article/${article.slug}`}
      className="group block bg-card rounded-xl overflow-hidden shadow-card hover:shadow-hover transition-smooth"
    >
      <div className="relative h-48 overflow-hidden">
        <img
          src={imageUrl}
          alt={article.title}
          width={400}
          height={192}
          loading={priority ? "eager" : "lazy"}
          decoding={priority ? "sync" : "async"}
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
        <p className="text-muted-foreground text-sm mb-4 line-clamp-3 text-justify">
          {article.excerpt}
        </p>

        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center space-x-4">
            <span className="flex items-center">
              <Calendar className="h-4 w-4 mr-1" />
              {new Date(article.date).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
              })}
            </span>
            <span className="flex items-center">
              <Clock className="h-4 w-4 mr-1" />
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
