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

function getBackendUrl(): string {
  return process.env.BACKEND_URL ?? "http://localhost:3005";
}

function buildArticlesUrl(
  categoryFilter?: string,
  limit?: number,
  offset?: number,
  searchQuery?: string
): URL {
  const url = new URL("/api/airesearch/articles", getBackendUrl());
  if (categoryFilter && categoryFilter.toLowerCase() !== "all") {
    url.searchParams.set("category", categoryFilter);
  }
  if (limit !== undefined) {
    url.searchParams.set("limit", limit.toString());
  }
  if (offset !== undefined) {
    url.searchParams.set("offset", offset.toString());
  }
  if (searchQuery && searchQuery.trim()) {
    url.searchParams.set("q", searchQuery.trim());
  }
  return url;
}

export async function getArticles(
  categoryFilter?: string,
  limit: number = 50,
  offset: number = 0,
  searchQuery?: string
): Promise<{ articles: Article[]; hasMore: boolean; total: number }> {
  const url = buildArticlesUrl(categoryFilter, limit, offset, searchQuery);
  
  // Cache otimizado: 5 minutos para lista de artigos (dados mudam com frequência moderada)
  // Reduz TTFB de ~500ms para ~100ms em requisições subsequentes
  const response = await fetch(url.toString(), {
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
  const url = new URL(
    `/api/airesearch/articles/${encodeURIComponent(slug)}`,
    getBackendUrl(),
  );

  // ISR (Incremental Static Regeneration) com cache agressivo
  // Artigos individuais são estáticos e mudam raramente
  // Revalida apenas em background, mantendo TTFB baixo
  const response = await fetch(url.toString(), {
    next: {
      revalidate: 3600, // Revalida a cada 1 hora (artigos individuais mudam menos)
      tags: ['article', slug],
    },
  });

  if (response.status === 404) {
    return null;
  }

  if (!response.ok) {
    throw new Error(
      `[AIResearch] Failed to load article: ${response.status} ${response.statusText}`,
    );
  }

  const payload = (await response.json()) as ArticleDetailResponse;
  return payload.article ?? null;
}
