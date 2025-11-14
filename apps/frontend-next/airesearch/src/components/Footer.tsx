import Link from "next/link";
import Image from "next/image";
import icon from "@/assets/airesearch-icon.png";

const Footer = () => {
  return (
    <footer className="relative border-t border-border bg-gradient-to-br from-card via-background to-card mt-auto overflow-hidden">
      {/* Decorative gradient */}
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-primary to-transparent opacity-20" />

      <div className="container mx-auto px-4 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div className="relative">
            <div className="flex items-center gap-2 mb-4">
              <Image
                src={icon}
                alt="AIResearch Icon"
                width={24}
                height={24}
                className="object-contain"
              />
              <h3 className="text-lg font-bold bg-gradient-to-r from-foreground to-primary bg-clip-text text-transparent">
                AIResearch
              </h3>
            </div>
            <p className="text-sm text-muted-foreground leading-relaxed">
              Scientific news about Artificial Intelligence
            </p>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Categories</h4>
            <ul className="space-y-2.5 text-sm">
              <li>
                <Link
                  href="/"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  Articles
                </Link>
              </li>
              <li>
                <Link
                  href="/education"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  Education
                </Link>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Resources</h4>
            <ul className="space-y-2.5 text-sm">
              <li>
                <Link
                  href="/about"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  About
                </Link>
              </li>
              <li>
                <Link
                  href="mailto:contact@hive-hub.ai"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  Contact
                </Link>
              </li>
              <li>
                <Link
                  href="/rss"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  RSS Feed
                </Link>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Legal</h4>
            <ul className="space-y-2.5 text-sm">
              <li>
                <Link
                  href="/terms"
                  className="text-muted-foreground hover:text-primary transition-colors duration-200 inline-block hover:translate-x-1"
                >
                  Terms
                </Link>
              </li>
            </ul>
          </div>
        </div>

        <div className="border-t border-border mt-10 pt-8">
          <div className="flex items-center justify-center">
            <p className="text-sm text-muted-foreground">
              &copy; 2025 AIResearch. All rights reserved.
            </p>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
