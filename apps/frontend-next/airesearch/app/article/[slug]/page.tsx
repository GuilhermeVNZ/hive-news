import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { AuthorImage } from "@/components/AuthorImage";
import { ShareButton } from "@/components/ShareButton";
import {
  Clock,
  Calendar,
  User,
  ArrowLeft,
  BookOpen,
  Download,
  Linkedin,
} from "lucide-react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Button } from "@/components/ui/button";
import type { Metadata } from "next";
import Image from "next/image";
import { findArticleBySlug } from "@/lib/articles";

// Extract arXiv ID from folder name
// Handles formats:
// - YYYY-MM-DD_source_ID (e.g., "2025-11-02_unknown_2510.25319" -> "2510.25319")
// - Direct ID (e.g., "2510.25319" -> "2510.25319")
function extractArxivId(folderName: string): string {
  // First, try to find arXiv ID pattern (YYYY.NNNNN or YYYY.NNNNNN)
  // arXiv IDs are typically 4 digits, dot, 4-6 digits (e.g., 2510.25319, 2510.123456)
  const arxivIdMatch = folderName.match(/\d{4}\.\d{4,6}/);
  if (arxivIdMatch) {
    return arxivIdMatch[0];
  }

  // If no pattern found, split by underscore and get the last segment
  const parts = folderName.split("_");
  if (parts.length >= 3) {
    // Format: YYYY-MM-DD_source_ID
    // The ID is the last part after removing date and source
    const lastPart = parts[parts.length - 1];
    // Check if last part looks like an arXiv ID
    if (lastPart.match(/^\d{4}\.\d{4,6}$/)) {
      return lastPart;
    }
  } else if (parts.length === 1) {
    // Single segment - might be direct ID
    if (folderName.match(/^\d{4}\.\d{4,6}$/)) {
      return folderName;
    }
  }

  // Fallback: return as-is if no pattern found
  return folderName;
}

// Map category values to display labels
const categoryLabels: Record<string, string> = {
  ai: "AI",
  robotics: "Robotics",
  science: "Science",
  coding: "Coding",
  crypto: "Crypto",
  data: "Data",
  ethics: "Ethics",
  games: "Games",
  hardware: "Hardware",
  legal: "Legal",
  network: "Network",
  quantum_computing: "Quantum Computing",
  security: "Security",
  sound: "Sound",
};

// Generate metadata for SEO
export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const decodedSlug = decodeURIComponent(slug);

  try {
    const article = await findArticleBySlug(decodedSlug);

    if (article) {
      return {
        title: article.title,
        description: article.excerpt,
        keywords: [article.category, "AI", "Machine Learning", article.title],
        authors: [{ name: article.author }],
        openGraph: {
          title: article.title,
          description: article.excerpt,
          type: "article",
          publishedTime: article.publishedAt,
          authors: [article.author],
          tags: article.imageCategories,
        },
        twitter: {
          card: "summary_large_image",
          title: article.title,
          description: article.excerpt,
        },
      } satisfies Metadata;
    }
  } catch (error) {
    console.error("Error generating metadata:", error);
  }

  return {
    title: "Article Not Found",
  };
}

export default async function ArticlePage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const decodedSlug = decodeURIComponent(slug);
  const article = await findArticleBySlug(decodedSlug);

  if (!article) {
    notFound();
  }

  const primaryCategory = article.imageCategories[0] || "ai";
  const categoryLabel = categoryLabels[primaryCategory] || primaryCategory;

  const articleImage =
    article.imagePath && article.imagePath.length > 0
      ? article.imagePath
      : "/images/ai/ai_1.jpg";

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        {/* Article Hero Section */}
        <div className="relative bg-gradient-to-br from-primary/5 via-background to-background py-12">
          <div className="container mx-auto px-4">
            {/* Back Button */}
            <Link
              href="/"
              className="inline-flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors mb-8 group"
            >
              <ArrowLeft className="h-4 w-4 group-hover:-translate-x-1 transition-transform" />
              <span>Back to articles</span>
            </Link>

            {/* Category Badges */}
            <div className="mb-6 flex flex-wrap gap-2">
              {article.imageCategories.map((cat: string, index: number) => (
                <span
                  key={index}
                  className={`px-4 py-2 text-sm font-semibold rounded-full border ${
                    index === 0
                      ? "text-primary bg-primary/10 border-primary/20"
                      : "text-muted-foreground bg-muted border-border"
                  }`}
                >
                  {index === 0 && (
                    <BookOpen className="h-3.5 w-3.5 inline mr-1.5" />
                  )}
                  {categoryLabels[cat] || cat}
                </span>
              ))}
            </div>

            {/* Title */}
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6 text-foreground leading-tight">
              {article.title}
            </h1>

            {/* Excerpt */}
            {article.excerpt && (
              <p className="text-2xl text-muted-foreground mb-8 leading-relaxed">
                {article.excerpt}
              </p>
            )}

            {/* Meta Information */}
            <div className="flex flex-wrap items-center gap-6 pb-8 border-b border-border">
              <div className="flex items-center gap-2 text-muted-foreground">
                <User className="h-4 w-4" />
                <span className="text-sm font-medium">{article.author}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <Calendar className="h-4 w-4" />
                <span className="text-sm">
                  {new Date(article.publishedAt).toLocaleDateString("en-US", {
                    day: "2-digit",
                    month: "long",
                    year: "numeric",
                  })}
                </span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <Clock className="h-4 w-4" />
                <span className="text-sm">{article.readTime} min read</span>
              </div>
              <div className="flex-1" />
              <ShareButton
                article={{
                  title: article.title,
                  linkedinPost: article.linkedinPost,
                  xPost: article.xPost,
                  imagePath: article.imagePath,
                }}
              />
            </div>
          </div>
        </div>

        {/* Article Content */}
        <article className="container mx-auto px-4 py-12 max-w-4xl">
          {/* Article Image */}
          {articleImage && (
            <div className="mb-12">
              <div className="aspect-video relative rounded-2xl overflow-hidden border border-border">
                <Image
                  src={articleImage}
                  alt={article.title}
                  fill
                  className="object-cover"
                  priority
                  sizes="(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1200px"
                  quality={85}
                  placeholder="blur"
                  blurDataURL="data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAAIAAoDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAhEAACAQMDBQAAAAAAAAAAAAABAgMABAUGIWGRkqGx0f/EABUBAQEAAAAAAAAAAAAAAAAAAAMF/8QAGhEAAgIDAAAAAAAAAAAAAAAAAAECEgMRkf/aAAwDAQACEQMRAD8AltJagyeH0AthI5xdrLcNM91BF5pX2HaH9bcfaSXWGaRmknyJckliyjqTzSlT54b6bk+h0R//9k="
                />
              </div>
            </div>
          )}

          {/* Content */}
          <div className="prose prose-xl max-w-none dark:prose-invert prose-headings:font-bold prose-p:text-lg prose-p:leading-relaxed prose-p:text-foreground prose-li:text-lg prose-strong:text-foreground prose-p:text-justify">
            <div className="article-content whitespace-pre-wrap leading-relaxed text-lg text-justify">
              {article.article.trim()}
            </div>
          </div>

          {/* Download Section */}
          <div className="mt-16 py-12 border-t border-b border-border bg-gradient-to-r from-primary/5 via-transparent to-primary/5 -mx-4 px-4">
            <div className="flex flex-col md:flex-row md:items-start justify-between gap-6">
              <div className="flex-1">
                <h3 className="text-xl font-bold mb-2">
                  Download Original Article
                </h3>
                <p className="text-muted-foreground">
                  Get the complete research paper in its original PDF format
                </p>
              </div>
              <Button
                size="lg"
                className="gap-2 bg-gradient-to-r from-primary to-primary/80 hover:from-primary/90 hover:to-primary/70 shadow-lg hover:shadow-xl transition-all whitespace-nowrap"
                asChild
              >
                <a
                  href={`https://arxiv.org/pdf/${extractArxivId(article.id)}.pdf`}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Download className="h-5 w-5" />
                  Download PDF from arXiv
                </a>
              </Button>
            </div>
          </div>

          {/* Related Articles */}
          <div className="mt-16 pt-12 border-t border-border">
            <h2 className="text-2xl font-bold mb-6 bg-gradient-to-r from-foreground to-primary bg-clip-text text-transparent">
              Related Articles
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* More Category Research */}
              <Link href={`/?category=${primaryCategory}`}>
                <div className="p-6 border border-border rounded-lg hover:border-primary transition-colors cursor-pointer group">
                  <span className="text-xs font-medium text-primary bg-primary/10 px-2 py-1 rounded">
                    {categoryLabel}
                  </span>
                  <h3 className="mt-2 font-semibold text-lg group-hover:text-primary transition-colors">
                    More {categoryLabel} Research
                  </h3>
                  <p className="mt-2 text-sm text-muted-foreground line-clamp-2">
                    Explore more articles in this category...
                  </p>
                </div>
              </Link>
              {/* Latest AI Articles */}
              <Link href="/">
                <div className="p-6 border border-border rounded-lg hover:border-primary transition-colors cursor-pointer group">
                  <span className="text-xs font-medium text-primary bg-primary/10 px-2 py-1 rounded">
                    AI Research
                  </span>
                  <h3 className="mt-2 font-semibold text-lg group-hover:text-primary transition-colors">
                    Latest AI Articles
                  </h3>
                  <p className="mt-2 text-sm text-muted-foreground line-clamp-2">
                    Discover the latest developments in AI...
                  </p>
                </div>
              </Link>
            </div>
          </div>

          {/* Author Info Section */}
          <div className="mt-16 pt-12 border-t border-border">
            <div className="bg-card rounded-xl p-6 shadow-card border border-border">
              <h3 className="text-lg font-bold mb-4">About the Author</h3>
              <div className="flex items-start space-x-4">
                <AuthorImage />
                <div className="flex-1">
                  <h4 className="font-semibold">Guilherme A.</h4>
                  <p className="text-sm text-muted-foreground mt-1 leading-relaxed">
                    Former dentist (MD) from Brazil, 41 years old, husband, and
                    AI enthusiast. In 2020, he transitioned from a decade-long
                    career in dentistry to pursue his passion for technology,
                    entrepreneurship, and helping others grow.
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
        </article>
      </main>
      <Footer />
    </div>
  );
}
