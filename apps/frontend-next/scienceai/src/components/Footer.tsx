import { useState, useEffect } from "react";
import { Link } from "react-router-dom";

interface Category {
  name: string;
  slug: string;
  icon: string;
  latestDate?: string;
}

export const Footer = () => {
  const [categories, setCategories] = useState<Category[]>([]);

  // Buscar categorias dinâmicas (top 5 mais recentes) com cache para evitar CLS
  useEffect(() => {
    async function fetchCategories() {
      try {
        // Cache por 5 minutos para categorias no Footer
        const cacheKey = 'scienceai-footer-categories';
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();
        const cacheDuration = 5 * 60 * 1000; // 5 minutos
        
        if (cached && (now - cacheTime) < cacheDuration) {
          const data = JSON.parse(cached).data;
          const sortedCategories = (data.categories || []).sort((a: Category, b: Category) => {
            if (!a.latestDate || !b.latestDate) return 0;
            return new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime();
          });
          
          // CRÍTICO: Garantir que não há duplicatas por slug
          const seenSlugs = new Set<string>();
          const uniqueCategories = sortedCategories.filter((cat) => {
            const slug = cat.slug.toLowerCase().trim();
            if (seenSlugs.has(slug)) {
              return false;
            }
            seenSlugs.add(slug);
            return true;
          });
          
          setCategories(uniqueCategories.slice(0, 5));
          return;
        }
        
        const response = await fetch('/api/categories', {
          headers: { 'Cache-Control': 'max-age=300' },
        });
        const data = await response.json();
        
        // Salvar no cache
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data,
          timestamp: now,
        }));
        
        const sortedCategories = (data.categories || []).sort((a: Category, b: Category) => {
          if (!a.latestDate || !b.latestDate) return 0;
          return new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime();
        });
        
        // CRÍTICO: Garantir que não há duplicatas por slug
        const seenSlugs = new Set<string>();
        const uniqueCategories = sortedCategories.filter((cat) => {
          const slug = cat.slug.toLowerCase().trim();
          if (seenSlugs.has(slug)) {
            return false;
          }
          seenSlugs.add(slug);
          return true;
        });
        
        setCategories(uniqueCategories.slice(0, 5)); // Máximo 5 categorias
      } catch (error) {
        console.error('Error fetching categories:', error);
        setCategories([]);
      }
    }
    fetchCategories();
  }, []);

  return (
    <footer className="bg-muted text-foreground mt-20" style={{ minHeight: '300px' }}>
      <div className="container mx-auto px-4 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Logo & Description */}
          <div className="col-span-1 md:col-span-2">
            <div className="text-2xl font-bold mb-4">
              <span className="text-foreground">Science</span>
              <span className="text-primary">AI</span>
            </div>
            <p className="text-sm text-muted-foreground mb-4">
              Your trusted source for the latest breakthroughs in artificial
              intelligence, robotics, medicine, space exploration, and data science.
            </p>
          </div>

          {/* Quick Links */}
          <div>
            <h3 className="font-semibold mb-4">Quick Links</h3>
            <ul className="space-y-2 text-sm">
              <li>
                <Link
                  to="/"
                  className="text-muted-foreground hover:text-primary transition-smooth"
                >
                  Home
                </Link>
              </li>
              <li>
                <Link
                  to="/about"
                  className="text-muted-foreground hover:text-primary transition-smooth"
                >
                  About Us
                </Link>
              </li>
              <li>
                <a
                  href="mailto:contact@hive-hub.ai"
                  className="text-muted-foreground hover:text-primary transition-smooth"
                >
                  Contact
                </a>
              </li>
              <li>
                <Link
                  to="/privacy-policy"
                  className="text-muted-foreground hover:text-primary transition-smooth"
                >
                  Privacy Policy
                </Link>
              </li>
            </ul>
          </div>

          {/* Categories - Dinâmicas (top 5 mais recentes) */}
          <div>
            <h3 className="font-semibold mb-4">Categories</h3>
            <ul className="space-y-2 text-sm">
              {categories.length > 0 ? (
                categories.map((category) => (
                  <li key={category.slug}>
                    <Link
                      to={`/category/${category.slug}`}
                      className="text-muted-foreground hover:text-primary transition-smooth"
                    >
                      {category.name}
                    </Link>
                  </li>
                ))
              ) : (
                <li className="text-muted-foreground">Loading categories...</li>
              )}
            </ul>
          </div>
        </div>

        <div className="border-t border-border mt-8 pt-8 text-center text-sm text-muted-foreground">
          <p>&copy; 2025 ScienceAI.news. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
};
