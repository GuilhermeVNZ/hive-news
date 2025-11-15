"use client";

import { memo } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

interface HeroProps {
  selectedCategory?: string;
  onCategorySelect?: (category: string) => void;
  searchQuery?: string;
  onSearchQueryChange?: (q: string) => void;
  onSubmitSearch?: () => void;
}

const Hero = memo(({
  selectedCategory,
  onCategorySelect,
  searchQuery,
  onSearchQueryChange,
  onSubmitSearch,
}: HeroProps) => {
  const categories = [
    { label: "Machine Learning", value: "ai" },
    { label: "Robotics", value: "robotics" },
    { label: "Science", value: "science" },
    { label: "Coding", value: "coding" },
    { label: "Hardware", value: "hardware" },
    { label: "Network", value: "network" },
    { label: "Security", value: "security" },
    { label: "Quantum Computing", value: "quantum_computing" },
    { label: "Data", value: "data" },
  ];

  return (
    <section className="relative overflow-hidden bg-gradient-to-br from-primary/10 via-background to-background py-10 md:py-16">
      {/* Animated Background Blobs */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-0 -left-4 w-96 h-96 bg-primary/20 rounded-full mix-blend-multiply filter blur-3xl opacity-30 animate-blob"></div>
        <div className="absolute top-0 -right-4 w-96 h-96 bg-cyan-500/20 rounded-full mix-blend-multiply filter blur-3xl opacity-30 animate-blob animation-delay-2000"></div>
        <div className="absolute -bottom-8 left-20 w-96 h-96 bg-blue-400/20 rounded-full mix-blend-multiply filter blur-3xl opacity-30 animate-blob animation-delay-4000"></div>
      </div>

      <div className="container mx-auto px-4 relative z-10">
        <div className="max-w-5xl mx-auto text-center animate-fade-in-up">
          <h1 className="text-5xl md:text-6xl lg:text-7xl font-bold mb-12 bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent animate-gradient">
            Discover the Latest
            <br />
            <span className="text-primary">AI Research</span>
          </h1>

          <div className="flex flex-col sm:flex-row gap-4 max-w-2xl mx-auto mb-12">
            <div className="relative flex-1 group">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground group-focus-within:text-primary transition-colors" />
              <Input
                id="hero-search-input"
                placeholder="Search articles, topics or researchers..."
                className="pl-12 h-14 text-base border-2 border-border bg-card/50 backdrop-blur-sm focus:border-primary transition-all"
                value={searchQuery ?? ""}
                onChange={(e) => onSearchQueryChange?.(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") onSubmitSearch?.();
                }}
              />
            </div>
            <Button
              size="lg"
              className="h-14 px-8 bg-gradient-to-r from-primary to-primary/80 hover:from-primary/90 hover:to-primary/70 text-primary-foreground shadow-lg hover:shadow-xl transition-all hover-lift"
              onClick={() => onSubmitSearch?.()}
            >
              Search
            </Button>
          </div>

          <div className="flex flex-wrap md:flex-nowrap gap-3 justify-center">
            {categories.map((cat, index) => (
              <button
                key={cat.value}
                onClick={() => onCategorySelect?.(cat.value)}
                className={`group px-4 py-2 text-xs sm:text-sm font-medium rounded-full border-2 transition-all duration-300 animate-fade-in whitespace-nowrap flex-shrink-0 ${
                  selectedCategory === cat.value
                    ? "border-primary bg-primary text-primary-foreground"
                    : "border-border bg-card/50 backdrop-blur-sm hover:border-primary hover:bg-primary/10 hover:text-primary"
                }`}
                style={{ animationDelay: `${index * 100}ms` }}
              >
                {cat.label}
              </button>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
});

Hero.displayName = 'Hero';

export default Hero;
