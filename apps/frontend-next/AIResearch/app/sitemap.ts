import { MetadataRoute } from 'next';
import fs from 'fs';
import path from 'path';

// Função para obter todos os artigos do diretório output
function getArticles() {
  const outputDir = path.join(process.cwd(), '../../output/AIResearch');
  
  if (!fs.existsSync(outputDir)) {
    return [];
  }
  
  const articles: Array<{ id: string; date: Date }> = [];
  
  try {
    const folders = fs.readdirSync(outputDir, { withFileTypes: true });
    
    folders.forEach((folder) => {
      if (folder.isDirectory()) {
        const articleId = folder.name;
        const articlePath = path.join(outputDir, articleId, 'article.md');
        
        if (fs.existsSync(articlePath)) {
          const stats = fs.statSync(articlePath);
          articles.push({
            id: articleId,
            date: stats.mtime,
          });
        }
      }
    });
  } catch (error) {
    console.error('Error reading articles directory:', error);
  }
  
  return articles;
}

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = 'https://airesearch.com';
  const articles = getArticles();
  
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
  
  // Páginas dinâmicas de artigos
  const articlePages: MetadataRoute.Sitemap = articles.map((article) => ({
    url: `${baseUrl}/article/${article.id}`,
    lastModified: article.date,
    changeFrequency: 'weekly' as const,
    priority: 0.9,
  }));
  
  return [...staticPages, ...articlePages];
}

