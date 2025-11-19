"use client";

import * as React from "react";
import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ThemeToggleProps {
  size?: "default" | "compact";
}

export function ThemeToggle({ size = "default" }: ThemeToggleProps) {
  const [darkMode, setDarkMode] = React.useState(false);
  const [mounted, setMounted] = React.useState(false);

  // Avoid hydration mismatch and load from localStorage
  React.useEffect(() => {
    setMounted(true);
    // Check localStorage first
    const savedTheme = localStorage.getItem("airesearch-ui-theme");
    if (savedTheme === "dark") {
      setDarkMode(true);
      document.documentElement.classList.add("dark");
    } else if (savedTheme === "light") {
      setDarkMode(false);
      document.documentElement.classList.remove("dark");
    } else {
      // If no saved theme, check if dark class is already set (from system preference or default)
      const isDark = document.documentElement.classList.contains("dark");
      setDarkMode(isDark);
    }
  }, []);

  const toggleDarkMode = () => {
    const newDarkMode = !darkMode;
    setDarkMode(newDarkMode);
    document.documentElement.classList.toggle("dark");
    // Save to localStorage
    if (typeof window !== "undefined") {
      localStorage.setItem(
        "airesearch-ui-theme",
        newDarkMode ? "dark" : "light",
      );
    }
  };

  const iconClasses = size === "compact" ? "h-4 w-4" : "h-5 w-5";

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
      onClick={toggleDarkMode}
      className={size === "compact" ? "h-9 w-9" : "h-14 w-14"}
      aria-label={darkMode ? "Switch to light mode" : "Switch to dark mode"}
    >
      {darkMode ? (
        <Sun className={iconClasses} />
      ) : (
        <Moon className={iconClasses} />
      )}
    </Button>
  );
}
