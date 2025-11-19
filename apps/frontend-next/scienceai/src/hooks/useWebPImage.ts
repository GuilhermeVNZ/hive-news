import { useState, useEffect } from 'react';
import { getWebPSupport, getWebPImageUrl } from '@/lib/imageUtils';

/**
 * Hook React para usar imagens WebP automaticamente
 * Similar ao WebP Express - converte automaticamente para WebP se suportado
 * @param originalUrl URL original da imagem
 * @param webPUrl URL específica do WebP (opcional)
 * @returns URL da imagem (WebP se suportado, original caso contrário)
 */
export function useWebPImage(originalUrl: string, webPUrl?: string): string {
  const [imageUrl, setImageUrl] = useState<string>(originalUrl);

  useEffect(() => {
    if (!originalUrl) {
      setImageUrl('');
      return;
    }

    // Se já é WebP, usar direto
    if (originalUrl.toLowerCase().endsWith('.webp')) {
      setImageUrl(originalUrl);
      return;
    }

    // Converter para WebP se suportado (equivalente ao WebP Express)
    getWebPImageUrl(originalUrl, webPUrl).then((url) => {
      setImageUrl(url);
    });
  }, [originalUrl, webPUrl]);

  return imageUrl;
}









