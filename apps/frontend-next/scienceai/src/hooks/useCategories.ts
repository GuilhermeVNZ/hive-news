import { useState, useEffect, useRef } from "react";

interface Category {
  name: string;
  slug: string;
  icon: string;
  latestDate?: string;
}

interface UseCategoriesOptions {
  cacheKey?: string;
  maxCategories?: number;
  cacheDuration?: number; // em ms
}

/**
 * Hook compartilhado para buscar categorias com deduplicação garantida
 * e cache para evitar múltiplas requisições
 */
export function useCategories(options: UseCategoriesOptions = {}) {
  const {
    cacheKey = "scienceai-categories",
    maxCategories = 5,
    cacheDuration = 5 * 60 * 1000, // 5 minutos
  } = options;

  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  // Usar ref para evitar requisições duplicadas durante React Strict Mode
  const fetchingRef = useRef(false);
  const abortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    async function fetchCategories() {
      // Evitar requisições duplicadas (React Strict Mode)
      if (fetchingRef.current) {
        return;
      }

      try {
        fetchingRef.current = true;
        
        // Cancelar requisição anterior se existir
        if (abortControllerRef.current) {
          abortControllerRef.current.abort();
        }
        
        // Criar novo AbortController
        const abortController = new AbortController();
        abortControllerRef.current = abortController;

        // Verificar cache
        const cached = sessionStorage.getItem(cacheKey);
        const cacheTime = cached ? JSON.parse(cached).timestamp : 0;
        const now = Date.now();

        if (cached && (now - cacheTime) < cacheDuration) {
          const data = JSON.parse(cached).data;
          const processedCategories = processCategories(
            data.categories || [],
            maxCategories,
          );
          setCategories(processedCategories);
          setLoading(false);
          fetchingRef.current = false;
          return;
        }

        // Buscar da API
        const response = await fetch("/api/categories", {
          headers: {
            "Cache-Control": "max-age=300", // 5 minutos
          },
          signal: abortController.signal,
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();

        // Processar e deduplicar categorias
        const processedCategories = processCategories(
          data.categories || [],
          maxCategories,
        );

        // Salvar no cache
        sessionStorage.setItem(
          cacheKey,
          JSON.stringify({
            data,
            timestamp: now,
          }),
        );

        setCategories(processedCategories);
        setError(null);
      } catch (err: any) {
        // Ignorar erros de abort
        if (err.name === "AbortError") {
          return;
        }
        
        console.error("Error fetching categories:", err);
        setError(err);
        
        // Tentar usar cache mesmo que expirado em caso de erro
        const cached = sessionStorage.getItem(cacheKey);
        if (cached) {
          try {
            const data = JSON.parse(cached).data;
            const processedCategories = processCategories(
              data.categories || [],
              maxCategories,
            );
            setCategories(processedCategories);
          } catch {
            // Ignorar erro de parse do cache
          }
        }
      } finally {
        setLoading(false);
        fetchingRef.current = false;
      }
    }

    fetchCategories();

    // Cleanup: cancelar requisição se componente desmontar
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      fetchingRef.current = false;
    };
  }, [cacheKey, maxCategories, cacheDuration]);

  return { categories, loading, error };
}

/**
 * Processa e deduplica categorias, garantindo que não há duplicatas
 */
function processCategories(
  categories: Category[],
  maxCategories: number,
): Category[] {
  // Ordenar por data mais recente primeiro
  const sortedCategories = [...categories].sort((a, b) => {
    if (!a.latestDate || !b.latestDate) return 0;
    return (
      new Date(b.latestDate).getTime() - new Date(a.latestDate).getTime()
    );
  });

  // Deduplicar por slug (case-insensitive)
  const seenSlugs = new Set<string>();
  const uniqueCategories = sortedCategories.filter((cat) => {
    const slug = cat.slug.toLowerCase().trim();
    if (seenSlugs.has(slug)) {
      console.warn(
        `[useCategories] ⚠️ Duplicate category filtered out: ${slug}`,
      );
      return false;
    }
    seenSlugs.add(slug);
    return true;
  });

  // Retornar apenas as N primeiras categorias
  return uniqueCategories.slice(0, maxCategories);
}








