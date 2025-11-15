"use client";

import { Menu } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import icon from "@/assets/airesearch-icon.png";
import { ThemeToggle } from "@/components/theme-toggle";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";

const Header = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

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
              priority
              quality={90}
            />
            <div className="absolute inset-0 bg-primary/20 rounded-full opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-300" />
          </div>
          <div className="hidden sm:block">
            <h2 className="text-lg font-bold bg-gradient-to-r from-foreground via-primary to-foreground bg-clip-text text-transparent animate-gradient">
              AIResearch
            </h2>
          </div>
        </Link>

        <nav className="hidden md:flex items-center gap-1" aria-label="Main navigation">
          <Link
            href="/"
            className="px-4 py-2 text-sm font-semibold text-foreground/80 hover:text-foreground hover:bg-accent rounded-lg transition-performance duration-200"
          >
            Articles
          </Link>
          <Link
            href="/education"
            className="px-4 py-2 text-sm font-semibold text-foreground/80 hover:text-foreground hover:bg-accent rounded-lg transition-performance duration-200"
          >
            Education
          </Link>
        </nav>

        <div className="flex items-center gap-2">
          <ThemeToggle size="compact" />
          <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
            <SheetTrigger asChild>
              <button
                className="md:hidden p-2 text-foreground hover:text-primary transition-colors hover:bg-accent rounded-lg"
                aria-label="Open mobile navigation menu"
                aria-expanded={mobileMenuOpen}
                aria-controls="mobile-menu"
              >
                <Menu className="h-5 w-5" aria-hidden="true" />
              </button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[300px] sm:w-[400px]" id="mobile-menu">
              <SheetHeader>
                <SheetTitle className="flex items-center gap-3">
                  <Image
                    src={icon}
                    alt="AIResearch Icon"
                    width={32}
                    height={32}
                    className="object-contain"
                  />
                  <span>AIResearch</span>
                </SheetTitle>
              </SheetHeader>
              <nav className="flex flex-col gap-4 mt-8" aria-label="Mobile navigation">
                <Link
                  href="/"
                  onClick={() => setMobileMenuOpen(false)}
                  className="px-4 py-3 text-base font-semibold text-foreground/80 hover:text-foreground hover:bg-accent rounded-lg transition-performance duration-200"
                >
                  Articles
                </Link>
                <Link
                  href="/education"
                  onClick={() => setMobileMenuOpen(false)}
                  className="px-4 py-3 text-base font-semibold text-foreground/80 hover:text-foreground hover:bg-accent rounded-lg transition-performance duration-200"
                >
                  Education
                </Link>
              </nav>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
};

export default Header;
