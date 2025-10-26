import { Search } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import logo from "@/assets/airesearch-logo.png";

const Header = () => {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-border bg-card/95 backdrop-blur supports-[backdrop-filter]:bg-card/60">
      <div className="container mx-auto flex h-16 items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <Image
            src={logo}
            alt="AIResearch Logo"
            width={40}
            height={40}
            className="object-cover object-center"
          />
        </div>

        <nav className="hidden md:flex items-center gap-6">
          <Link
            href="#"
            className="text-sm font-medium text-foreground hover:text-primary transition-colors"
          >
            Últimas
          </Link>
          <Link
            href="#"
            className="text-sm font-medium text-foreground hover:text-primary transition-colors"
          >
            Categorias
          </Link>
          <Link
            href="#"
            className="text-sm font-medium text-foreground hover:text-primary transition-colors"
          >
            Sobre
          </Link>
        </nav>

        <button className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors border border-border rounded-md hover:border-primary">
          <Search className="h-4 w-4" />
          <span className="hidden sm:inline">Buscar</span>
        </button>
      </div>
    </header>
  );
};

export default Header;
