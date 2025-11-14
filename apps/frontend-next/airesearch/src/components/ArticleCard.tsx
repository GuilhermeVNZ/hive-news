import { Clock, ArrowRight } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import Link from "next/link";

interface ArticleCardProps {
  id: string;
  slug?: string;
  title: string;
  excerpt: string;
  publishedAt: string;
  author: string;
  category: string;
  readTime: number;
  imageCategories?: string[];
}

const ArticleCard = ({
  title,
  excerpt,
  publishedAt,
  author,
  category,
  readTime,
  imageCategories = [],
  slug: providedSlug,
}: ArticleCardProps) => {
  const slug =
    providedSlug ||
    title
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "");

  // Map category values to display labels
  const categoryLabels: Record<string, string> = {
    ai: "AI",
    robotics: "Robotics",
    science: "Science",
    coding: "Coding",
    crypto: "Crypto",
    database: "Database",
    ethics: "Ethics",
    games: "Games",
    hardware: "Hardware",
    legal: "Legal",
    network: "Network",
    security: "Security",
    sound: "Sound",
  };

  return (
    <Link href={`/article/${slug}`}>
      <Card className="group relative overflow-hidden hover:border-primary/50 transition-all duration-300 hover-lift cursor-pointer h-full bg-gradient-to-br from-card via-card to-card/50">
        {/* Gradient overlay on hover */}
        <div className="absolute inset-0 bg-gradient-to-br from-primary/0 via-primary/0 to-primary/0 group-hover:from-primary/5 group-hover:via-primary/0 group-hover:to-primary/5 transition-all duration-300 pointer-events-none" />

        <CardHeader className="relative">
          <div className="flex items-center gap-2 mb-3 flex-wrap">
            {imageCategories.length > 0 ? (
              // Show all 3 categories in order of priority
              imageCategories.slice(0, 3).map((cat, index) => (
                <span
                  key={index}
                  className={`px-3 py-1 text-xs font-semibold rounded-full border transition-all ${
                    index === 0
                      ? "bg-gradient-to-r from-primary/20 to-primary/10 text-primary border-primary/20 group-hover:border-primary/40"
                      : index === 1
                        ? "bg-gradient-to-r from-cyan-500/20 to-cyan-500/10 text-cyan-700 dark:text-cyan-400 border-cyan-500/20"
                        : "bg-gradient-to-r from-blue-500/20 to-blue-500/10 text-blue-700 dark:text-blue-400 border-blue-500/20"
                  }`}
                >
                  {categoryLabels[cat] || cat}
                </span>
              ))
            ) : (
              // Fallback to single category
              <span className="px-3 py-1 text-xs font-semibold rounded-full bg-gradient-to-r from-primary/20 to-primary/10 text-primary border border-primary/20 group-hover:border-primary/40 transition-all">
                {category}
              </span>
            )}
          </div>
          <CardTitle className="line-clamp-2 group-hover:text-primary transition-colors duration-300">
            {title}
          </CardTitle>
          <CardDescription className="line-clamp-3 mt-2">
            {excerpt}
          </CardDescription>
        </CardHeader>

        <CardContent className="relative">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4 text-xs text-muted-foreground">
              <span className="group-hover:text-primary transition-colors">
                {author}
              </span>
              <span>
                {new Date(publishedAt).toLocaleDateString("en-US", {
                  day: "2-digit",
                  month: "short",
                  year: "numeric",
                })}
              </span>
              <div className="flex items-center gap-1.5">
                <Clock className="h-3.5 w-3.5" />
                <span>{readTime} min</span>
              </div>
            </div>
            <ArrowRight className="h-4 w-4 text-muted-foreground group-hover:text-primary group-hover:translate-x-1 transition-all duration-300" />
          </div>
        </CardContent>
      </Card>
    </Link>
  );
};

export default ArticleCard;
