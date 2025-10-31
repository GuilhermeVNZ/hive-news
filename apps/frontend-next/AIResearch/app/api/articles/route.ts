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
  isPromotional?: boolean;
}

async function readArticlesFromDir(outputDir: string, isPromotional: boolean = false): Promise<Article[]> {
  const articles: Article[] = [];
  
  try {
    const dirs = await fs.readdir(outputDir);
    
    for (const articleId of dirs) {
      const articleDir = path.join(outputDir, articleId);
      
      try {
        const stats = await fs.stat(articleDir);
        if (!stats.isDirectory()) continue;
        
        // Ler arquivos necessários
        const titlePath = path.join(articleDir, 'title.txt');
        const articlePath = path.join(articleDir, 'article.md');
        
        const [title, articleContent, dirStats] = await Promise.all([
          fs.readFile(titlePath, 'utf-8').catch(() => ''),
          fs.readFile(articlePath, 'utf-8').catch(() => ''),
          fs.stat(articleDir), // Para obter data de modificação
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
            // Usar data de modificação da pasta como publishedAt (mais recente = modificado mais recentemente)
            publishedAt: dirStats.mtime.toISOString(),
            author: 'AI Research',
            category: imageCategories[0] || 'ai',
            readTime: Math.ceil(articleContent.split(' ').length / 200),
            imageCategories,
            // Marcar como promocional
            isPromotional: isPromotional,
          });
        }
      } catch (err) {
        console.error(`Error reading article ${articleId}:`, err);
      }
    }
  } catch (err) {
    // Se a pasta não existir, não é erro (pode não ter artigos promocionais ainda)
    if (isPromotional) {
      console.log('Promotional directory not found, skipping...');
    } else {
      console.error('Error reading output directory:', err);
    }
  }
  
  return articles;
}

async function readArticles(): Promise<Article[]> {
  const baseOutputDir = path.join(process.cwd(), '../../../output');
  const promotionalDir = path.join(baseOutputDir, 'Promotional');
  const normalDir = path.join(baseOutputDir, 'AIResearch');
  
  // Ler artigos promocionais primeiro
  const promotionalArticles = await readArticlesFromDir(promotionalDir, true);
  
  // Ler artigos normais
  const normalArticles = await readArticlesFromDir(normalDir, false);
  
  // Combinar: promocionais primeiro, depois normais
  const allArticles = [...promotionalArticles, ...normalArticles];
  
  // Ordenar por data de publicação (mais recente primeiro)
  // Artigos promocionais vêm primeiro automaticamente porque foram adicionados primeiro
  allArticles.sort((a, b) => {
    // Primeiro, garantir que promocionais vêm primeiro
    if (a.isPromotional && !b.isPromotional) return -1;
    if (!a.isPromotional && b.isPromotional) return 1;
    // Se ambos são promocionais ou ambos normais, ordenar por data (mais recente primeiro)
    return new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
  });
  
  return allArticles;
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
