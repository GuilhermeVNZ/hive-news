"use client";

import { useState, useEffect } from "react";
import ArticleCard from "./ArticleCard";
import { Button } from "@/components/ui/button";

interface Article {
  id: string;
  title: string;
  excerpt: string;
  article: string;
  publishedAt: string;
  author: string;
  category: string;
  readTime: number;
  imageCategories: string[];
  featured?: boolean; // Featured status from registry
}

interface ArticleGridProps {
  selectedCategory?: string;
  searchQuery?: string;
}

const ArticleGrid = ({ selectedCategory, searchQuery }: ArticleGridProps) => {
  const [articles, setArticles] = useState<Article[]>([]);
  const [loading, setLoading] = useState(true);
  const [displayedCount, setDisplayedCount] = useState(6);
  
  useEffect(() => {
    async function fetchArticles() {
      try {
        // Add cache-busting to ensure fresh data
        const response = await fetch('/api/articles?' + new Date().getTime());
        const data = await response.json();
        const articles = data.articles || [];
        
        // Debug: Log featured articles
        const featuredArticles = articles.filter((a: Article) => a.featured === true);
        console.log(`[AIResearch ArticleGrid] Fetched ${articles.length} articles, ${featuredArticles.length} featured:`, 
          featuredArticles.map((a: Article) => ({ id: a.id, title: a.title.substring(0, 50), featured: a.featured })));
        
        setArticles(articles);
      } catch (error) {
        console.error('Error fetching articles:', error);
      } finally {
        setLoading(false);
      }
    }
    
    fetchArticles();
  }, []);
  
  // Reset displayed count when category changes
  useEffect(() => {
    setDisplayedCount(6);
  }, [selectedCategory]);
  
  // Normalize helper (similar to dashboard Logs)
  const normalize = (s: string) =>
    s
      .toLowerCase()
      .replace(/[^\w\s]/g, " ")
      .replace(/\s+/g, " ")
      .trim();

  const words = (searchQuery ?? "").trim()
    ? normalize(searchQuery as string).split(" ")
    : [];

  // Filter by category first
  let byCategory = selectedCategory
    ? articles.filter(a => a.imageCategories && a.imageCategories.includes(selectedCategory.toLowerCase()))
    : articles;
  
  // If no category selected and no search, prioritize featured articles
  // Articles are already sorted by backend (featured first), but ensure they stay that way
  if (!selectedCategory && words.length === 0) {
    // Debug: Log featured articles
    if (process.env.NODE_ENV === 'development') {
      const featuredArticles = byCategory.filter(a => a.featured === true);
      console.log(`[AIResearch ArticleGrid] Featured articles found: ${featuredArticles.length}`, 
        featuredArticles.map(a => ({ id: a.id, title: a.title.substring(0, 50), featured: a.featured })));
    }
    // Sort by featured first, then by date (most recent first)
    // This ensures featured articles appear at the top
    byCategory = [...byCategory].sort((a, b) => {
      const aFeatured = a.featured === true;
      const bFeatured = b.featured === true;
      if (aFeatured !== bFeatured) {
        return aFeatured ? -1 : 1; // Featured first
      }
      // If both featured or both not featured, sort by date
      return new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
    });
  }

  const filteredArticles = words.length === 0
    ? byCategory
    : byCategory.filter(a => {
        const topics = Array.isArray(a.imageCategories) ? a.imageCategories.join(" ") : "";
        const hay = normalize(`${a.title} ${a.id} ${a.excerpt} ${a.category ?? ""} ${topics}`);
        return words.every(w => hay.includes(w));
      });
  
  // Display only first N articles
  // Featured articles should be at the top due to sorting above
  const displayedArticles = filteredArticles.slice(0, displayedCount);
  const hasMore = filteredArticles.length > displayedCount;

  if (loading) {
    return (
      <section className="container mx-auto px-4 py-16" id="articles">
        <div className="flex justify-center items-center h-64">
          <p className="text-muted-foreground">Loading articles...</p>
        </div>
      </section>
    );
  }

  return (
    <section className="container mx-auto px-4 py-16" id="articles">
      <div className="mb-10">
        <h2 className="text-3xl md:text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent">
          {selectedCategory ? `Articles in ${selectedCategory}` : "Featured Articles"}
        </h2>
        <p className="text-muted-foreground text-lg">
          {selectedCategory 
            ? `${filteredArticles.length} ${filteredArticles.length === 1 ? 'article found' : 'articles found'}`
            : "Explore the latest research and developments in AI"
          }
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
            style={{ animationDelay: `${index * 150}ms`, animationFillMode: 'both' }}
          >
            <ArticleCard {...article} />
          </div>
        ))}
      </div>
          
          {hasMore && (
            <div className="flex justify-center mt-12">
              <Button 
                onClick={() => setDisplayedCount(prev => prev + 6)}
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
