"use client";

import * as React from "react";
import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";

interface ThemeToggleProps {
  size?: "default" | "compact";
}

export function ThemeToggle({ size = "default" }: ThemeToggleProps) {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = React.useState(false);

  // Avoid hydration mismatch
  React.useEffect(() => {
    setMounted(true);
  }, []);

  const toggleTheme = () => {
    if (theme === "dark") {
      setTheme("light");
    } else if (theme === "light") {
      setTheme("dark");
    } else {
      // System theme - determine current preference and toggle
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
        .matches
        ? "dark"
        : "light";
      setTheme(systemTheme === "dark" ? "light" : "dark");
    }
  };

  const iconClasses = size === "compact" ? "h-4 w-4" : "h-5 w-5";
  const isDark = theme === "dark" || (theme === "system" && typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  if (!mounted) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className={size === "compact" ? "h-9 w-9" : "h-14 w-14"}
      >
        <Sun className={iconClasses} />
      </Button>
    );
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={toggleTheme}
      className={size === "compact" ? "h-9 w-9" : "h-14 w-14"}
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
    >
      {isDark ? (
        <Sun className={iconClasses} />
      ) : (
        <Moon className={iconClasses} />
      )}
    </Button>
  );
}
