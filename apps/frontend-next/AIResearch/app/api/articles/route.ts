import { NextResponse } from 'next/server';
import fs from 'fs/promises';
import path from 'path';

interface Article {
  id: string;
  title: string;
  excerpt: string;
  article: string;
  publishedAt: string;
  author: string;
  category: string;
  readTime: number;
  imageCategories: string[];
}

async function readArticles(): Promise<Article[]> {
  const outputDir = path.join(process.cwd(), '../../../output/AIResearch');
  
  try {
    const dirs = await fs.readdir(outputDir);
    const articles: Article[] = [];
    
    for (const articleId of dirs) {
      const articleDir = path.join(outputDir, articleId);
      
      try {
        const stats = await fs.stat(articleDir);
        if (!stats.isDirectory()) continue;
        
        // Ler arquivos necessários
        const titlePath = path.join(articleDir, 'title.txt');
        const articlePath = path.join(articleDir, 'article.md');
        
        const [title, articleContent] = await Promise.all([
          fs.readFile(titlePath, 'utf-8').catch(() => ''),
          fs.readFile(articlePath, 'utf-8').catch(() => ''),
        ]);
        
        // Extrair excerpt (primeiras 3 linhas)
        const excerpt = articleContent
          .split('\n')
          .filter(line => line.trim())
          .slice(0, 3)
          .join(' ')
          .substring(0, 200) + '...';
        
        // Ler image categories
        const categoriesPath = path.join(articleDir, 'image_categories.txt');
        const categoriesContent = await fs.readFile(categoriesPath, 'utf-8').catch(() => '');
        const imageCategories = categoriesContent
          .split('\n')
          .filter(c => c.trim());
        
        if (title && articleContent) {
          articles.push({
            id: articleId,
            title: title.trim(),
            excerpt,
            article: articleContent,
            publishedAt: new Date().toISOString(),
            author: 'AI Research',
            category: imageCategories[0] || 'ai',
            readTime: Math.ceil(articleContent.split(' ').length / 200),
            imageCategories,
          });
        }
      } catch (err) {
        console.error(`Error reading article ${articleId}:`, err);
      }
    }
    
    return articles;
  } catch (err) {
    console.error('Error reading output directory:', err);
    return [];
  }
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const category = searchParams.get('category');
  
  const articles = await readArticles();
  
  let filteredArticles = articles;
  if (category && category !== 'all') {
    filteredArticles = articles.filter(a => a.category === category);
  }
  
  return NextResponse.json({ articles: filteredArticles });
}

