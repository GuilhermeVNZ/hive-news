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
  const response = await fetch(url.toString(), { cache: "no-store" });

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

  const response = await fetch(url.toString(), { cache: "no-store" });

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
