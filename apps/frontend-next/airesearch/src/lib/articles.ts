import { Article } from "@/types/article";

interface ArticleApiResponse {
  articles: Article[];
}

interface ArticleDetailResponse {
  article: Article;
}

function getBackendUrl(): string {
  return process.env.BACKEND_URL ?? "http://localhost:3005";
}

function buildArticlesUrl(categoryFilter?: string): URL {
  const url = new URL("/api/airesearch/articles", getBackendUrl());
  if (categoryFilter && categoryFilter.toLowerCase() !== "all") {
    url.searchParams.set("category", categoryFilter);
  }
  return url;
}

export async function getArticles(categoryFilter?: string): Promise<Article[]> {
  const url = buildArticlesUrl(categoryFilter);
  // Desabilitar cache do Next.js para payloads grandes (>2MB)
  // O cache será gerenciado pelo backend/Nginx via Cache-Control headers
  // Isso evita o erro "items over 2MB can not be cached"
  const response = await fetch(url.toString(), {
    cache: 'no-store', // Desabilita cache do Next.js para payloads grandes
  });

  if (!response.ok) {
    throw new Error(
      `[AIResearch] Backend returned ${response.status}: ${response.statusText}`,
    );
  }

  const payload = (await response.json()) as ArticleApiResponse;
  return payload.articles ?? [];
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
