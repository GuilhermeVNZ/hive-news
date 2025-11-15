import { useEffect } from 'react';

/**
 * Componente para preload da imagem principal (LCP)
 * Adiciona link rel="preload" dinamicamente para melhorar LCP
 */
interface ImagePreloadProps {
  imageUrl: string;
}

export function ImagePreload({ imageUrl }: ImagePreloadProps) {
  useEffect(() => {
    if (!imageUrl) return;

    // Criar link de preload para a imagem principal (LCP)
    const link = document.createElement('link');
    link.rel = 'preload';
    link.as = 'image';
    link.href = imageUrl;
    link.setAttribute('fetchpriority', 'high');
    
    // Adicionar ao head
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

