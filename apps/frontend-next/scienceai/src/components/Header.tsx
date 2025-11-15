import { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import { Menu, Search, X, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import logoIcon from "@/assets/scienceai-icone.png";

interface Category {
  name: string;
  slug: string;
  icon: string;
  latestDate?: string;
}

interface HeaderProps {
  onSearch?: (query: string) => void;
}

export const Header = ({ onSearch }: HeaderProps) => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [darkMode, setDarkMode] = useState(false);
  const [categories, setCategories] = useState<Category[]>([]);
  const location = useLocation();

  // Buscar categorias dinâmicas da API (top 5 mais recentes) com cache
  useEffect(() => {
    async function fetchCategories() {
      try {
        // Cache por 5 minutos para categorias no Header
        const cacheKey = 'scienceai-header-categories';
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
          setCategories(sortedCategories.slice(0, 5));
          return;
        }
        
        const response = await fetch('/api/categories', {
          headers: {
            'Cache-Control': 'max-age=300', // 5 minutos
          },
        });
        const data = await response.json();
        
        // Salvar no cache
        sessionStorage.setItem(cacheKey, JSON.stringify({
          data,
          timestamp: now,
        }));
        
        // Ordenar da esquerda para direita (mais recente primeiro)
        const sortedCategories = (data.categories || []).sort((a: Category, b: Category) => {
          if (!a.latestDate || !b.latestDate) return 0;
          return new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime();
        });
        setCategories(sortedCategories.slice(0, 5)); // Máximo 5 categorias
      } catch (error) {
        console.error('Error fetching categories:', error);
        // Fallback: usar categorias padrão se API falhar
        setCategories([]);
      }
    }
    fetchCategories();
  }, []);

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
          <nav className="hidden md:flex items-center space-x-8">
            {categories.map((category) => (
              <Link
                key={category.slug}
                to={`/category/${category.slug}`}
                className={`text-sm font-medium transition-smooth hover:text-primary ${
                  isActive(`/category/${category.slug}`)
                    ? "text-primary"
                    : "text-foreground"
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
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => setSearchOpen(false)}
                >
                  <X className="h-5 w-5" />
                </Button>
              </form>
            ) : (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSearchOpen(true)}
                className="hidden md:flex"
              >
                <Search className="h-5 w-5" />
              </Button>
            )}

            {/* Dark Mode Toggle */}
            <Button variant="ghost" size="icon" onClick={toggleDarkMode}>
              {darkMode ? (
                <Sun className="h-5 w-5" />
              ) : (
                <Moon className="h-5 w-5" />
              )}
            </Button>

            {/* Mobile Menu Button */}
            <Button
              variant="ghost"
              size="icon"
              className="md:hidden"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            >
              {mobileMenuOpen ? (
                <X className="h-6 w-6" />
              ) : (
                <Menu className="h-6 w-6" />
              )}
            </Button>
          </div>
        </div>

        {/* Mobile Menu */}
        {mobileMenuOpen && (
          <nav className="md:hidden py-4 space-y-4 border-t animate-fade-in">
            {categories.map((category) => (
              <Link
                key={category.slug}
                to={`/category/${category.slug}`}
                className="block py-2 text-sm font-medium hover:text-primary transition-smooth"
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
              />
            </form>
          </nav>
        )}
      </div>
    </header>
  );
};
