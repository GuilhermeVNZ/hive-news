import { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { Menu, Search, X, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import logoIcon from "@/assets/scienceai-icone.png";
import { useCategories } from "@/hooks/useCategories";

interface HeaderProps {
  onSearch?: (query: string) => void;
}

export const Header = ({ onSearch }: HeaderProps) => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [darkMode, setDarkMode] = useState(false);
  const location = useLocation();
  
  // Usar hook compartilhado para categorias
  const { categories } = useCategories({
    cacheKey: "scienceai-header-categories",
    maxCategories: 5,
  });

  const isActive = (path: string) => location.pathname === path;

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    onSearch?.(searchQuery);
    setSearchOpen(false);
  };

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle("dark");
  };

  return (
    <header className="sticky top-0 z-50 bg-card border-b shadow-card">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <Link to="/" className="flex items-center space-x-2">
            <img 
              src={logoIcon} 
              alt="ScienceAI" 
              width={40}
              height={40}
              loading="eager"
              decoding="sync"
              className="h-10 w-auto"
              style={{ aspectRatio: '1/1' }}
            />
            <div className="text-2xl font-bold">
              <span className="text-foreground">Science</span>
              <span className="text-primary">AI</span>
            </div>
          </Link>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex items-center space-x-8" aria-label="Main navigation">
            {categories.map((category) => (
              <Link
                key={category.slug}
                to={`/category/${category.slug}`}
                className={`text-sm font-medium transition-smooth hover:text-primary ${
                  isActive(`/category/${category.slug}`)
                    ? "text-primary"
                    : "text-foreground/80"
                }`}
              >
                {category.name}
              </Link>
            ))}
          </nav>

          {/* Actions */}
          <div className="flex items-center space-x-4">
            {/* Search */}
            {searchOpen ? (
              <form onSubmit={handleSearch} className="flex items-center space-x-2">
                <Input
                  type="text"
                  placeholder="Search articles..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-64"
                  autoFocus
                  aria-label="Search articles"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => setSearchOpen(false)}
                  aria-label="Close search"
                >
                  <X className="h-5 w-5" aria-hidden="true" />
                </Button>
              </form>
            ) : (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSearchOpen(true)}
                className="hidden md:flex"
                aria-label="Open search"
              >
                <Search className="h-5 w-5" aria-hidden="true" />
              </Button>
            )}

            {/* Dark Mode Toggle */}
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleDarkMode}
              aria-label={darkMode ? "Switch to light mode" : "Switch to dark mode"}
            >
              {darkMode ? (
                <Sun className="h-5 w-5" aria-hidden="true" />
              ) : (
                <Moon className="h-5 w-5" aria-hidden="true" />
              )}
            </Button>

            {/* Mobile Menu Button */}
            <Button
              variant="ghost"
              size="icon"
              className="md:hidden"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              aria-label={mobileMenuOpen ? "Close mobile menu" : "Open mobile menu"}
              aria-expanded={mobileMenuOpen}
              aria-controls="mobile-menu"
            >
              {mobileMenuOpen ? (
                <X className="h-6 w-6" aria-hidden="true" />
              ) : (
                <Menu className="h-6 w-6" aria-hidden="true" />
              )}
            </Button>
          </div>
        </div>

        {/* Mobile Menu */}
        {mobileMenuOpen && (
          <nav
            className="md:hidden py-4 space-y-4 border-t animate-fade-in"
            id="mobile-menu"
            aria-label="Mobile navigation"
          >
            {categories.map((category) => (
              <Link
                key={category.slug}
                to={`/category/${category.slug}`}
                className="block py-2 text-sm font-medium text-foreground/80 hover:text-primary transition-smooth"
                onClick={() => setMobileMenuOpen(false)}
              >
                {category.name}
              </Link>
            ))}
            <form onSubmit={handleSearch} className="pt-2">
              <Input
                type="text"
                placeholder="Search articles..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                aria-label="Search articles"
              />
            </form>
          </nav>
        )}
      </div>
    </header>
  );
};
