/**
 * Utilitários para otimização de imagens
 * Similar ao WebP Express - detecta suporte a WebP e converte automaticamente
 */

/**
 * Seleciona uma imagem de artigo baseada nas categorias (fallback client-side)
 * Esta é uma versão simplificada para o cliente - o servidor seleciona a imagem correta
 * @param imageCategories Array de categorias da imagem
 * @param articleId ID do artigo para seleção determinística
 * @returns Caminho da imagem selecionada
 */
export function selectArticleImage(
  imageCategories: string[] | undefined,
  articleId: string,
): string {
  // Se não há categorias, usar fallback padrão (priorizar WebP)
  if (!imageCategories || imageCategories.length === 0) {
    return '/images/ai/ai_1.webp';
  }

  // Mapear categoria para diretório de imagens
  const categoryMap: Record<string, string> = {
    ai: 'ai',
    robotics: 'robotics',
    science: 'science',
    coding: 'coding',
    crypto: 'crypto',
    data: 'data',
    ethics: 'ethics',
    games: 'games',
    hardware: 'hardware',
    legal: 'legal',
    network: 'network',
    security: 'security',
    sound: 'sound',
    quantum_computing: 'ai', // Fallback para ai
  };

  // Usar primeira categoria disponível
  const firstCategory = imageCategories[0]?.toLowerCase() || 'ai';
  const imageDir = categoryMap[firstCategory] || 'ai';

  // Seleção determinística baseada no ID do artigo (hash simples)
  let hash = 5381;
  for (let i = 0; i < articleId.length; i++) {
    hash = ((hash << 5) + hash) + articleId.charCodeAt(i);
    hash = hash & hash; // Convert to 32bit integer
  }

  // Usar hash para selecionar imagem de 1-10 (assumindo que há pelo menos 10 imagens por categoria)
  const imageNumber = (Math.abs(hash) % 10) + 1;

  // Priorizar WebP: tentar usar .webp primeiro (fallback para .jpg se não existir)
  return `/images/${imageDir}/${imageDir}_${imageNumber}.webp`;
}

/**
 * Verifica se o navegador suporta WebP
 */
export function supportsWebP(): Promise<boolean> {
  return new Promise((resolve) => {
    const webP = new Image();
    webP.onload = webP.onerror = () => {
      resolve(webP.height === 2);
    };
    webP.src =
      "data:image/webp;base64,UklGRjoAAABXRUJQVlA4IC4AAACyAgCdASoCAAIALmk0mk0iIiIiIgBoSygABc6WWgAA/veff/0PP8bA//LwYAAA";
  });
}

/**
 * Converte URL de imagem para WebP se suportado (equivalente ao WebP Express)
 * Fallback para formato original se WebP não suportado
 * Similar ao WebP Express - transforma todas as imagens para WebP automaticamente
 */
export async function getWebPImageUrl(
  originalUrl: string,
  webPUrl?: string,
): Promise<string> {
  // Se já é WebP, retornar direto
  if (originalUrl.toLowerCase().endsWith('.webp')) {
    return originalUrl;
  }

  // Se tem URL WebP específica, verificar suporte
  if (webPUrl) {
    const webPSupported = await supportsWebP();
    if (webPSupported) {
      return webPUrl;
    }
    return originalUrl;
  }
  
  // Se não tem WebP específico mas suporta, tentar substituir extensão
  // Similar ao WebP Express - converte automaticamente
  const webPSupported = await supportsWebP();
  if (webPSupported && originalUrl && !originalUrl.endsWith('.webp')) {
    // Tentar substituir extensão por .webp
    // Nota: Requer que o servidor tenha versão WebP das imagens
    // ou um middleware que converta automaticamente
    const webPUrl = originalUrl.replace(/\.(jpg|jpeg|png|gif)$/i, '.webp');
    return webPUrl;
  }
  
  return originalUrl;
}

/**
 * Hook React para usar WebP automaticamente (equivalente ao WebP Express)
 * Cacheia o resultado do suporte WebP para melhor performance
 */
let webPSupportedCache: boolean | null = null;
let webPCheckPromise: Promise<boolean> | null = null;

export function getWebPSupport(): Promise<boolean> {
  if (webPSupportedCache !== null) {
    return Promise.resolve(webPSupportedCache);
  }
  
  if (webPCheckPromise) {
    return webPCheckPromise;
  }
  
  webPCheckPromise = supportsWebP().then((supported) => {
    webPSupportedCache = supported;
    return supported;
  });
  
  return webPCheckPromise;
}

/**
 * Adiciona lazy loading e otimizações de imagem ao elemento
 */
export function optimizeImageElement(
  img: HTMLImageElement,
  options: {
    lazy?: boolean;
    fetchPriority?: 'high' | 'low' | 'auto';
    decoding?: 'async' | 'sync' | 'auto';
  } = {},
): void {
  const {
    lazy = true,
    fetchPriority = 'auto',
    decoding = 'async',
  } = options;

  // Adicionar lazy loading se suportado
  if (lazy && 'loading' in HTMLImageElement.prototype) {
    img.loading = 'lazy';
  }

  // Adicionar fetchPriority
  if ('fetchPriority' in HTMLImageElement.prototype) {
    img.fetchPriority = fetchPriority;
  }

  // Adicionar decoding
  img.decoding = decoding;
}

/**
 * Cria um Image Element otimizado
 */
export function createOptimizedImage(
  src: string,
  options: {
    alt?: string;
    lazy?: boolean;
    priority?: boolean;
    onLoad?: () => void;
    onError?: () => void;
  } = {},
): HTMLImageElement {
  const img = document.createElement('img');
  img.src = src;
  img.alt = options.alt || '';
  
  optimizeImageElement(img, {
    lazy: options.lazy !== false && !options.priority,
    fetchPriority: options.priority ? 'high' : 'auto',
    decoding: options.priority ? 'sync' : 'async',
  });

  if (options.onLoad) {
    img.onload = options.onLoad;
  }
  
  if (options.onError) {
    img.onerror = options.onError;
  }

  return img;
}