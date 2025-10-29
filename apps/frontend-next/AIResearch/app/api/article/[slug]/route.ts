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
  imagePath?: string;
}

async function findArticleById(articleId: string): Promise<Article | null> {
  const outputDir = path.join(process.cwd(), '../../../output/AIResearch');
  const articleDir = path.join(outputDir, articleId);
  
  try {
    const stats = await fs.stat(articleDir);
    if (!stats.isDirectory()) return null;
    
    // Read article files
    const titlePath = path.join(articleDir, 'title.txt');
    const articlePath = path.join(articleDir, 'article.md');
    const categoriesPath = path.join(articleDir, 'image_categories.txt');
    
    const [title, articleContent, categoriesContent] = await Promise.all([
      fs.readFile(titlePath, 'utf-8').catch(() => ''),
      fs.readFile(articlePath, 'utf-8').catch(() => ''),
      fs.readFile(categoriesPath, 'utf-8').catch(() => ''),
    ]);
    
    if (!title || !articleContent) return null;
    
    // Extract excerpt (first 3 lines)
    const excerpt = articleContent
      .split('\n')
      .filter(line => line.trim())
      .slice(0, 3)
      .join(' ')
      .substring(0, 200) + '...';
    
    // Parse image categories
    const imageCategories = categoriesContent
      .split('\n')
      .filter(c => c.trim());
    
    // Select image based on categories and sequential order
    const imagePath = await selectArticleImage(imageCategories, articleId);
    
    const category = imageCategories[0] || 'ai';
    
    return {
      id: articleId,
      title: title.trim(),
      excerpt,
      article: articleContent,
      publishedAt: stats.mtime.toISOString(),
      author: 'AI Research',
      category,
      readTime: Math.ceil(articleContent.split(' ').length / 200),
      imageCategories,
      imagePath,
    };
  } catch (err) {
    console.error(`Error reading article ${articleId}:`, err);
    return null;
  }
}

async function selectArticleImage(categories: string[], articleId: string): Promise<string | undefined> {
  const imagesDir = path.join(process.cwd(), '../../../images');
  
  try {
    // Get all available categories
    const dirs = await fs.readdir(imagesDir);
    
    // Try each category in order of priority
    for (const category of categories) {
      const categoryDir = path.join(imagesDir, category);
      
      try {
        const stats = await fs.stat(categoryDir);
        if (!stats.isDirectory()) continue;
        
        const files = await fs.readdir(categoryDir);
        const imageFiles = files.filter(f => /\.(jpg|jpeg|png|webp)$/i.test(f));
        
        if (imageFiles.length > 0) {
          // Sort by number in filename to ensure sequential order
          imageFiles.sort((a, b) => {
            const numA = parseInt(a.match(/\d+/)?.[0] || '0');
            const numB = parseInt(b.match(/\d+/)?.[0] || '0');
            return numA - numB;
          });
          
          // Use article ID to determine which image to use (ensures different images)
          const imageIndex = parseInt(articleId.split('.').pop()?.replace(/[^0-9]/g, '') || '0') % imageFiles.length;
          const selectedImage = imageFiles[imageIndex];
          
          // Return relative path for use in img src
          return `/images/${category}/${selectedImage}`;
        }
      } catch (err) {
        continue;
      }
    }
  } catch (err) {
    console.error('Error selecting article image:', err);
  }
  
  return undefined;
}

export async function GET(
  request: Request,
  { params }: { params: Promise<{ slug: string }> }
) {
  const { slug } = await params;
  
  console.log('Searching for article with slug:', slug);
  
  // Find article by slug (convert slug back to article ID)
  const outputDir = path.join(process.cwd(), '../../../output/AIResearch');
  
  try {
    const dirs = await fs.readdir(outputDir);
    console.log(`Found ${dirs.length} article directories`);
    
    for (const articleId of dirs) {
      try {
        const article = await findArticleById(articleId);
        
        if (article) {
          const articleSlug = article.title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
          
          if (articleSlug === slug) {
            console.log(`Found article: ${articleId}`);
            return NextResponse.json({ article });
          }
        }
      } catch (err) {
        console.error(`Error processing article ${articleId}:`, err);
        continue;
      }
    }
    
    console.log('Article not found');
    return NextResponse.json({ error: 'Article not found' }, { status: 404 });
  } catch (err) {
    console.error('Error reading articles directory:', err);
    return NextResponse.json({ error: 'Error reading articles' }, { status: 500 });
  }
}

