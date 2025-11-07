import { Search, Menu } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import icon from "@/assets/airesearch-icon.png";
import { ThemeToggle } from "@/components/theme-toggle";

const Header = () => {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/80 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 shadow-sm">
      <div className="container mx-auto flex h-16 items-center justify-between px-4">
        <Link href="/" className="flex items-center gap-3 group">
          <div className="relative">
            <Image
              src={icon}
              alt="AIResearch Icon"
              width={40}
              height={40}
              className="object-contain group-hover:scale-105 transition-transform duration-300"
            />
            <div className="absolute inset-0 bg-primary/20 rounded-full opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-300" />
          </div>
          <div className="hidden sm:block">
            <h2 className="text-lg font-bold bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent animate-gradient">
              AIResearch
            </h2>
          </div>
        </Link>

        <nav className="hidden md:flex items-center gap-1">
          <Link
            href="/"
            className="px-4 py-2 text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-all duration-200"
          >
            Articles
          </Link>
          <Link
            href="/education"
            className="px-4 py-2 text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-all duration-200"
          >
            Education
          </Link>
        </nav>

        <div className="flex items-center gap-2">
          <button className="hidden sm:flex items-center gap-2 px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-all hover:bg-accent rounded-lg border border-border hover:border-primary">
            <Search className="h-4 w-4" />
            <span>Search</span>
          </button>
          <ThemeToggle size="compact" />
          <button className="md:hidden p-2 text-muted-foreground hover:text-foreground transition-colors hover:bg-accent rounded-lg">
            <Menu className="h-5 w-5" />
          </button>
        </div>
      </div>
    </header>
  );
};

export default Header;
