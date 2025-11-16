import { useEffect, useRef, useState } from 'react';

interface UseInfiniteScrollOptions {
  /**
   * Callback chamado quando o usuário chega próximo ao final da página
   */
  onLoadMore: () => void;
  
  /**
   * Distância do bottom da página para trigger do load (em pixels)
   * Padrão: 200px
   */
  threshold?: number;
  
  /**
   * Se deve parar de carregar (todas as páginas foram carregadas)
   */
  hasMore?: boolean;
  
  /**
   * Se está carregando mais itens no momento
   */
  isLoading?: boolean;
}

/**
 * Hook para implementar scroll infinito
 * Detecta quando o usuário chega próximo ao final da página e chama onLoadMore
 * 
 * @example
 * ```tsx
 * const { loadMoreRef } = useInfiniteScroll({
 *   onLoadMore: () => {
 *     setPage(prev => prev + 1);
 *   },
 *   hasMore: hasMorePages,
 *   isLoading: loading,
 * });
 * 
 * return (
 *   <div>
 *     {items.map(item => <Item key={item.id} {...item} />)}
 *     <div ref={loadMoreRef}>{isLoading && 'Loading...'}</div>
 *   </div>
 * );
 * ```
 */
export function useInfiniteScroll({
  onLoadMore,
  threshold = 200,
  hasMore = true,
  isLoading = false,
}: UseInfiniteScrollOptions) {
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loadMoreRef = useRef<HTMLDivElement>(null);
  const [isObserving, setIsObserving] = useState(false);

  useEffect(() => {
    // Não observar se não há mais itens
    if (!hasMore) {
      if (observerRef.current && loadMoreRef.current) {
        observerRef.current.unobserve(loadMoreRef.current);
        setIsObserving(false);
      }
      return;
    }

    // Não observar se está carregando (evitar múltiplas chamadas)
    if (isLoading) {
      if (observerRef.current && loadMoreRef.current && isObserving) {
        observerRef.current.unobserve(loadMoreRef.current);
        setIsObserving(false);
      }
      return;
    }

    // Criar Intersection Observer se não existe
    if (!observerRef.current) {
      observerRef.current = new IntersectionObserver(
        (entries) => {
          const [entry] = entries;
          
          // Se o elemento está visível e não está carregando, carregar mais
          if (entry.isIntersecting && !isLoading && hasMore) {
            onLoadMore();
          }
        },
        {
          // Trigger quando o elemento está a `threshold` pixels do viewport
          rootMargin: `0px 0px ${threshold}px 0px`,
          threshold: 0.1,
        }
      );
    }

    // Observar o elemento de trigger se existe e não está observando
    if (loadMoreRef.current && !isObserving) {
      observerRef.current.observe(loadMoreRef.current);
      setIsObserving(true);
    }

    // Cleanup
    return () => {
      if (observerRef.current && loadMoreRef.current && isObserving) {
        observerRef.current.unobserve(loadMoreRef.current);
        setIsObserving(false);
      }
    };
  }, [onLoadMore, threshold, hasMore, isLoading, isObserving]);

  // Cleanup ao desmontar
  useEffect(() => {
    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
        observerRef.current = null;
      }
    };
  }, []);

  return {
    loadMoreRef,
  };
}

