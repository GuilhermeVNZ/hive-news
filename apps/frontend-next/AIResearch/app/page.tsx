"use client";

import { useState } from "react";
import Header from "@/components/Header";
import Hero from "@/components/Hero";
import ArticleGrid from "@/components/ArticleGrid";
import Footer from "@/components/Footer";

export default function Home() {
  const [selectedCategory, setSelectedCategory] = useState<string>("");

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

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">
        <Hero 
          selectedCategory={selectedCategory}
          onCategorySelect={handleCategorySelect}
        />
        <ArticleGrid selectedCategory={selectedCategory} />
      </main>
      <Footer />
    </div>
  );
}






