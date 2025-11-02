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
        const linkedinPostPath = path.join(articleDir, 'linkedin.txt');
        
        const [title, articleContent, linkedinPostContent, dirStats] = await Promise.all([
          fs.readFile(titlePath, 'utf-8').catch(() => ''),
          fs.readFile(articlePath, 'utf-8').catch(() => ''),
          fs.readFile(linkedinPostPath, 'utf-8').catch(() => ''),
          fs.stat(articleDir), // Para obter data de modificação
        ]);
        
        // Usar conteúdo de linkedin.txt como excerpt (subtítulo) para AIResearch
        // Se linkedin.txt não existir ou estiver vazio, usar fallback das primeiras linhas do artigo
        let excerpt = linkedinPostContent.trim();
        if (!excerpt) {
          // Fallback: extrair excerpt das primeiras 3 linhas do artigo
          excerpt = articleContent
            .split('\n')
            .filter(line => line.trim())
            .slice(0, 3)
            .join(' ')
            .substring(0, 200) + '...';
        }
        
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
  
  // Ler registry para verificar featured status
  const featuredMap = new Map<string, boolean>();
  try {
    const registryPath = path.join(process.cwd(), '../../../articles_registry.json');
    const registryContent = await fs.readFile(registryPath, 'utf-8');
    const registry = JSON.parse(registryContent);
    if (registry.articles) {
      for (const [id, meta] of Object.entries(registry.articles)) {
        const metadata = meta as any;
        if (metadata.featured === true) {
          featuredMap.set(id, true);
        }
      }
    }
  } catch (err) {
    // Registry não encontrado ou erro - continuar sem featured
  }
  
  // Adicionar campo featured aos artigos
  for (const article of allArticles) {
    (article as any).featured = featuredMap.get(article.id) || false;
  }
  
  // Ordenar: featured primeiro, depois promocionais, depois por data (mais recente primeiro)
  allArticles.sort((a, b) => {
    const aFeatured = (a as any).featured || false;
    const bFeatured = (b as any).featured || false;
    
    // Featured sempre vem primeiro
    if (aFeatured && !bFeatured) return -1;
    if (!aFeatured && bFeatured) return 1;
    
    // Se ambos são featured ou ambos não são featured, considerar promocional
    if (aFeatured === bFeatured) {
      // Promocionais vêm depois dos featured
      if (a.isPromotional && !b.isPromotional) return -1;
      if (!a.isPromotional && b.isPromotional) return 1;
      
      // Se ambos são promocionais ou ambos normais, ordenar por data (mais recente primeiro)
      return new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
    }
    
    // Fallback (não deve chegar aqui)
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
