import { Clock } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import Link from "next/link";

interface ArticleCardProps {
  id: string;
  title: string;
  excerpt: string;
  publishedAt: string;
  author: string;
  category: string;
  readTime: number;
}

const ArticleCard = ({
  title,
  excerpt,
  publishedAt,
  author,
  category,
  readTime,
}: ArticleCardProps) => {
  return (
    <Link href={`/article/${encodeURIComponent(title)}`}>
      <Card className="hover:border-primary transition-colors cursor-pointer h-full">
        <CardHeader>
          <div className="flex items-center gap-2 mb-2">
            <span className="px-2 py-1 text-xs font-medium rounded-full bg-primary/10 text-primary">
              {category}
            </span>
          </div>
          <CardTitle className="line-clamp-2">{title}</CardTitle>
          <CardDescription className="line-clamp-3">{excerpt}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span>{author}</span>
            <div className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              <span>{readTime} min leitura</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
};

export default ArticleCard;
