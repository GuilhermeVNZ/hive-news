import { MetadataRoute } from 'next';
import { getArticles } from '@/lib/articles';

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const baseUrl = 'https://airesearch.com';
  
  // Páginas estáticas
  const staticPages: MetadataRoute.Sitemap = [
    {
      url: baseUrl,
      lastModified: new Date(),
      changeFrequency: 'daily',
      priority: 1,
    },
    {
      url: `${baseUrl}/education`,
      lastModified: new Date(),
      changeFrequency: 'weekly',
      priority: 0.8,
    },
  ];
  
  // Tentar obter artigos, mas não falhar o build se o backend não estiver disponível
  let articlePages: MetadataRoute.Sitemap = [];
  try {
    const { articles } = await getArticles();
    articlePages = articles.map((article) => ({
      url: `${baseUrl}/article/${article.slug}`,
      lastModified: new Date(article.publishedAt),
      changeFrequency: 'weekly' as const,
      priority: 0.9,
    }));
  } catch (error) {
    // Durante o build do Docker, o backend pode não estar disponível
    // Não falhar o build, apenas retornar páginas estáticas
    console.warn('[Sitemap] Failed to fetch articles, returning static pages only:', error);
  }
  
  return [...staticPages, ...articlePages];
}

