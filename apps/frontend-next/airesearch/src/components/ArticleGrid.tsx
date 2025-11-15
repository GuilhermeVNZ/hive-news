"use client";

import { useEffect, useMemo, useState } from "react";
import dynamic from "next/dynamic";
import ArticleCard from "./ArticleCard";
import { Button } from "@/components/ui/button";
import type { Article } from "@/types/article";

// Lazy load do ArticleCard para melhorar performance inicial
// O ArticleCard será carregado apenas quando necessário
const LazyArticleCard = dynamic(() => import("./ArticleCard"), {
  loading: () => <div className="h-64 bg-muted animate-pulse rounded-lg" />,
});

interface ArticleGridProps {
  articles: Article[];
  selectedCategory?: string;
  searchQuery?: string;
}

const ArticleGrid = ({
  articles,
  selectedCategory,
  searchQuery,
}: ArticleGridProps) => {
  const [displayedCount, setDisplayedCount] = useState(6);

  useEffect(() => {
    setDisplayedCount(6);
  }, [selectedCategory, searchQuery]);

  const normalize = (value: string) =>
    value
      .toLowerCase()
      .replace(/[^\w\s]/g, " ")
      .replace(/\s+/g, " ")
      .trim();

  const normalizedCategory = selectedCategory?.toLowerCase().trim();
  const searchWords = useMemo(() => {
    if (!searchQuery || !searchQuery.trim()) {
      return [];
    }
    return normalize(searchQuery).split(" ");
  }, [searchQuery]);

  const filteredByCategory = useMemo(() => {
    if (!normalizedCategory) {
      return articles;
    }

    return articles.filter((article) =>
      Array.isArray(article.imageCategories)
        ? article.imageCategories.includes(normalizedCategory)
        : false,
    );
  }, [articles, normalizedCategory]);

  const prioritizedArticles = useMemo(() => {
    if (normalizedCategory || searchWords.length > 0) {
      return filteredByCategory;
    }

    return [...filteredByCategory].sort((a, b) => {
      const dateDiff =
        new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
      if (dateDiff !== 0) {
        return dateDiff;
      }

      const aFeatured = a.featured === true;
      const bFeatured = b.featured === true;
      if (aFeatured !== bFeatured) {
        return aFeatured ? -1 : 1;
      }

      return String(b.id).localeCompare(String(a.id));
    });
  }, [filteredByCategory, normalizedCategory, searchWords.length]);

  const filteredArticles = useMemo(() => {
    if (searchWords.length === 0) {
      return prioritizedArticles;
    }

    return prioritizedArticles.filter((article) => {
      const topics = Array.isArray(article.imageCategories)
        ? article.imageCategories.join(" ")
        : "";
      const haystack = normalize(
        `${article.title} ${article.id} ${article.excerpt} ${article.category ?? ""} ${topics}`,
      );
      return searchWords.every((word) => haystack.includes(word));
    });
  }, [prioritizedArticles, searchWords]);

  const displayedArticles = useMemo(
    () => filteredArticles.slice(0, displayedCount),
    [filteredArticles, displayedCount],
  );

  const hasMore = filteredArticles.length > displayedCount;

  return (
    <section className="container mx-auto px-4 py-16" id="articles">
      <div className="mb-10">
        <h2 className="text-3xl md:text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
          {selectedCategory
            ? `Articles in ${selectedCategory}`
            : "Featured Articles"}
        </h2>
        <p className="text-muted-foreground text-lg">
          {selectedCategory
            ? `${filteredArticles.length} ${
                filteredArticles.length === 1
                  ? "article found"
                  : "articles found"
              }`
            : "Explore the latest research and developments in AI"}
        </p>
      </div>
      {filteredArticles.length === 0 ? (
        <div className="text-center py-16">
          <p className="text-muted-foreground text-lg">No articles found</p>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {displayedArticles.map((article, index) => (
              <div
                key={article.id}
                className="animate-fade-in-up"
                style={{
                  animationDelay: `${index * 150}ms`,
                  animationFillMode: "both",
                }}
              >
                {/* Usar ArticleCard normal para os primeiros 6 (above the fold) */}
                {index < 6 ? (
                  <ArticleCard {...article} />
                ) : (
                  <LazyArticleCard {...article} />
                )}
              </div>
            ))}
          </div>

          {hasMore && (
            <div className="flex justify-center mt-12">
              <Button
                onClick={() => setDisplayedCount((prev) => prev + 6)}
                className="px-8 py-6 text-lg"
                variant="outline"
              >
                Load More Articles
              </Button>
            </div>
          )}
        </>
      )}
    </section>
  );
};

export default ArticleGrid;
