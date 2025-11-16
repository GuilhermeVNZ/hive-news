"use client";

import { useEffect } from "react";

/**
 * Componente para preload da imagem principal (LCP)
 * Adiciona link rel="preload" dinamicamente após renderização
 */
interface ImagePreloadProps {
  imageUrl: string;
}

export function ImagePreload({ imageUrl }: ImagePreloadProps) {
  useEffect(() => {
    // Criar link de preload para a imagem principal (LCP)
    const link = document.createElement("link");
    link.rel = "preload";
    link.as = "image";
    link.href = imageUrl;
    link.setAttribute("fetchpriority", "high");

    // Adicionar srcset para diferentes tamanhos (Next.js Image otimiza automaticamente)
    // O Next.js já gera srcset, então preload apenas da imagem original
    document.head.appendChild(link);

    return () => {
      // Cleanup: remover link quando componente desmontar
      if (document.head.contains(link)) {
        document.head.removeChild(link);
      }
    };
  }, [imageUrl]);

  return null;
}
