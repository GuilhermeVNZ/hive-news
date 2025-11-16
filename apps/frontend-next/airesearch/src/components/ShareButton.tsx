"use client";

import {
  Share2,
  Twitter,
  Linkedin,
  MessageCircle,
  Copy,
  Send,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { toast } from "sonner";

interface ShareButtonProps {
  article: {
    title: string;
    linkedinPost?: string;
    xPost?: string;
    imagePath?: string;
  };
  url?: string;
}

export function ShareButton({ article, url }: ShareButtonProps) {
  const currentUrl =
    url || (typeof window !== "undefined" ? window.location.href : "");

  const handleShare = (platform: string) => {
    if (platform === "copy") {
      navigator.clipboard.writeText(currentUrl);
      toast.success("Link copied to clipboard!");
    } else if (platform === "twitter" || platform === "x") {
      // X/Twitter: conteúdo do arquivo x.txt (sem link no texto, o parâmetro url já adiciona automaticamente)
      const shareText = article.xPost || article.title;
      window.open(
        `https://twitter.com/intent/tweet?text=${encodeURIComponent(shareText)}&url=${encodeURIComponent(currentUrl)}`,
        "_blank",
      );
    } else if (platform === "linkedin") {
      // LinkedIn: conteúdo do arquivo linkedin.txt + link + preview da imagem
      // LinkedIn usa share-offsite que permite preview automático da imagem via Open Graph
      window.open(
        `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(currentUrl)}`,
        "_blank",
      );
    } else if (platform === "whatsapp") {
      // WhatsApp: título + link + preview da imagem
      const shareText = `${article.title}\n\n${currentUrl}`;
      window.open(
        `https://api.whatsapp.com/send?text=${encodeURIComponent(shareText)}`,
        "_blank",
      );
    } else if (platform === "telegram") {
      // Telegram: título + link + preview da imagem (mesmo modelo do WhatsApp)
      window.open(
        `https://t.me/share/url?url=${encodeURIComponent(currentUrl)}&text=${encodeURIComponent(article.title)}`,
        "_blank",
      );
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" className="gap-2">
          <Share2 className="h-4 w-4" />
          Share
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => handleShare("twitter")}>
          <Twitter className="h-4 w-4 mr-2" />
          Share on X
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => handleShare("linkedin")}>
          <Linkedin className="h-4 w-4 mr-2" />
          Share on LinkedIn
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => handleShare("whatsapp")}>
          <MessageCircle className="h-4 w-4 mr-2" />
          Share on WhatsApp
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => handleShare("telegram")}>
          <Send className="h-4 w-4 mr-2" />
          Share on Telegram
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => handleShare("copy")}>
          <Copy className="h-4 w-4 mr-2" />
          Copy Link
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
