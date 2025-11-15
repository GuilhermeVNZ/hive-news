import { Link } from "react-router-dom";
import { useCategories } from "@/hooks/useCategories";

export const Footer = () => {
  // Usar hook compartilhado para categorias (evita duplicação)
  const { categories } = useCategories({
    cacheKey: "scienceai-footer-categories",
    maxCategories: 5,
  });

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
            <p className="text-sm text-foreground/70 mb-4">
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
                  className="text-foreground/75 hover:text-primary transition-smooth"
                >
                  Home
                </Link>
              </li>
              <li>
                <Link
                  to="/about"
                  className="text-foreground/75 hover:text-primary transition-smooth"
                >
                  About Us
                </Link>
              </li>
              <li>
                <a
                  href="mailto:contact@hive-hub.ai"
                  className="text-foreground/75 hover:text-primary transition-smooth"
                >
                  Contact
                </a>
              </li>
              <li>
                <Link
                  to="/privacy-policy"
                  className="text-foreground/75 hover:text-primary transition-smooth"
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
                      className="text-foreground/75 hover:text-primary transition-smooth"
                    >
                      {category.name}
                    </Link>
                  </li>
                ))
              ) : (
                <li className="text-foreground/70">Loading categories...</li>
              )}
            </ul>
          </div>
        </div>

        <div className="border-t border-border mt-8 pt-8 text-center text-sm text-foreground/70">
          <p>&copy; 2025 ScienceAI.news. All rights reserved.</p>
        </div>
      </div>
    </footer>
  );
};
