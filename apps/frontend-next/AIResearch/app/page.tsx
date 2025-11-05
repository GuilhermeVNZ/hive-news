"use client";

import { useState, useEffect, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Header from "@/components/Header";
import Hero from "@/components/Hero";
import ArticleGrid from "@/components/ArticleGrid";
import Footer from "@/components/Footer";

function HomeContent() {
  const searchParams = useSearchParams();
  const [selectedCategory, setSelectedCategory] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [committedQuery, setCommittedQuery] = useState<string>("");

  // Read category from URL query param
  useEffect(() => {
    const categoryParam = searchParams.get("category");
    if (categoryParam) {
      setSelectedCategory(categoryParam);
      // Smooth scroll to articles after category is set
      setTimeout(() => {
        const articlesSection = document.getElementById("articles");
        if (articlesSection) {
          articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      }, 100);
    }
  }, [searchParams]);

  const handleCategorySelect = (category: string) => {
    setSelectedCategory(category === selectedCategory ? "" : category);
    
    // Smooth scroll to articles
    setTimeout(() => {
      const articlesSection = document.getElementById("articles");
      if (articlesSection) {
        articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 100);
  };

  const handleSubmitSearch = () => {
    setCommittedQuery(searchQuery);
    // scroll to articles
    setTimeout(() => {
      const articlesSection = document.getElementById("articles");
      if (articlesSection) {
        articlesSection.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }, 50);
  };

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        <Hero 
          selectedCategory={selectedCategory}
          onCategorySelect={handleCategorySelect}
          searchQuery={searchQuery}
          onSearchQueryChange={setSearchQuery}
          onSubmitSearch={handleSubmitSearch}
        />
        <ArticleGrid selectedCategory={selectedCategory} searchQuery={committedQuery} />
      </main>
      <Footer />
    </div>
  );
}

export default function Home() {
  return (
    <Suspense fallback={
      <div className="flex min-h-screen flex-col">
        <Header />
        <main className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground">Loading...</p>
        </main>
        <Footer />
      </div>
    }>
      <HomeContent />
    </Suspense>
  );
}
