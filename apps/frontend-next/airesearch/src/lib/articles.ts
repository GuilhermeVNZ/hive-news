import { Article } from "@/types/article";

interface ArticleApiResponse {
  articles: Article[];
  pagination?: {
    total: number;
    limit: number;
    offset: number;
    hasMore: boolean;
  };
}

interface ArticleDetailResponse {
  article: Article;
}

// Removed unused getBackendUrl function - now using relative URLs with Next.js proxy

function buildArticlesUrl(
  categoryFilter?: string,
  limit?: number,
  offset?: number,
  searchQuery?: string
): string {
  // Build query parameters
  const params = new URLSearchParams();
  
  if (categoryFilter && categoryFilter.toLowerCase() !== "all") {
    params.set("category", categoryFilter);
  }
  if (limit !== undefined) {
    params.set("limit", limit.toString());
  }
  if (offset !== undefined) {
    params.set("offset", offset.toString());
  }
  if (searchQuery && searchQuery.trim()) {
    params.set("q", searchQuery.trim());
  }
  
  // Return path with query string
  const queryString = params.toString();
  return `/api/airesearch/articles${queryString ? `?${queryString}` : ''}`;
}

export async function getArticles(
  categoryFilter?: string,
  limit: number = 50,
  offset: number = 0,
  searchQuery?: string
): Promise<{ articles: Article[]; hasMore: boolean; total: number }> {
  const relativePath = buildArticlesUrl(categoryFilter, limit, offset, searchQuery);
  
  // Check if we're running on the server (SSR/SSG) or client
  const isServer = typeof window === 'undefined';
  const url = isServer 
    ? `http://backend:3005${relativePath}` // Use internal Docker service name on server
    : relativePath; // Use relative URL on client (leverages Next.js proxy)
  
  console.log(`[AIResearch] Fetching articles - Server: ${isServer}, URL: ${url}`);
  
  // Cache otimizado: 5 minutos para lista de artigos (dados mudam com frequência moderada)
  // Reduz TTFB de ~500ms para ~100ms em requisições subsequentes
  const response = await fetch(url, {
    next: {
      revalidate: 300, // Cache de 5 minutos (ISR)
    },
  });

  if (!response.ok) {
    throw new Error(
      `[AIResearch] Backend returned ${response.status}: ${response.statusText}`,
    );
  }

  const payload = (await response.json()) as ArticleApiResponse;
  return {
    articles: payload.articles ?? [],
    hasMore: payload.pagination?.hasMore ?? false,
    total: payload.pagination?.total ?? payload.articles?.length ?? 0,
  };
}

export async function findArticleBySlug(slug: string): Promise<Article | null> {
  console.log(`[AIResearch] Fetching article by slug: ${slug}`);
  
  const relativePath = `/api/airesearch/articles/${encodeURIComponent(slug)}`;
  
  // Check if we're running on the server (SSR/SSG) or client
  const isServer = typeof window === 'undefined';
  const url = isServer 
    ? `http://backend:3005${relativePath}` // Use internal Docker service name on server
    : relativePath; // Use relative URL on client (leverages Next.js proxy)
  
  console.log(`[AIResearch] Server: ${isServer}, URL: ${url}`);

  try {
    const response = await fetch(url, {
      next: {
        revalidate: 3600, // Revalida a cada 1 hora
        tags: ['article', slug],
      },
    });

    console.log(`[AIResearch] Response status: ${response.status}`);

    if (response.status === 404) {
      console.log(`[AIResearch] Article not found: ${slug}`);
      return null;
    }

    if (!response.ok) {
      console.error(`[AIResearch] Failed to load article: ${response.status} ${response.statusText}`);
      throw new Error(
        `[AIResearch] Failed to load article: ${response.status} ${response.statusText}`,
      );
    }

    const payload = (await response.json()) as ArticleDetailResponse;
    console.log(`[AIResearch] Successfully loaded article: ${payload.article.title}`);
    return payload.article ?? null;
  } catch (error) {
    console.error(`[AIResearch] Error fetching article: ${error}`);
    // During build time, return null instead of throwing
    if (error instanceof Error && error.message.includes('ECONNREFUSED')) {
      console.log(`[AIResearch] Build time error, returning null for slug: ${slug}`);
      return null;
    }
    throw error;
  }
}
