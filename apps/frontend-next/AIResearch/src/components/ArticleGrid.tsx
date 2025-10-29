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
}

interface ArticleGridProps {
  selectedCategory?: string;
}

const ArticleGrid = ({ selectedCategory }: ArticleGridProps) => {
  const [articles, setArticles] = useState<Article[]>([]);
  const [loading, setLoading] = useState(true);
  const [displayedCount, setDisplayedCount] = useState(6);
  
  useEffect(() => {
    async function fetchArticles() {
      try {
        const response = await fetch('/api/articles');
        const data = await response.json();
        setArticles(data.articles || []);
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
  
  const filteredArticles = selectedCategory 
    ? articles.filter(article => {
        // Check if any image category matches
        return article.imageCategories && article.imageCategories.includes(selectedCategory.toLowerCase());
      })
    : articles;
  
  // Display only first N articles
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
