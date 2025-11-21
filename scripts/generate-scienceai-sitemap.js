#!/usr/bin/env node

/**
 * Script para gerar sitemap dinâmico do ScienceAI
 * Inclui páginas estáticas + artigos do backend
 */

const fs = require('fs');
const path = require('path');

// Configuração
const BASE_URL = 'https://scienceai.news';
const OUTPUT_PATH = path.join(__dirname, '../apps/frontend-next/scienceai/public/sitemap.xml');
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:3005';

// Páginas estáticas
const STATIC_PAGES = [
  {
    url: BASE_URL,
    changefreq: 'daily',
    priority: '1.0'
  },
  {
    url: `${BASE_URL}/about`,
    changefreq: 'monthly',
    priority: '0.8'
  },
  {
    url: `${BASE_URL}/contact`,
    changefreq: 'monthly',
    priority: '0.7'
  },
  {
    url: `${BASE_URL}/privacy`,
    changefreq: 'yearly',
    priority: '0.5'
  },
  {
    url: `${BASE_URL}/terms`,
    changefreq: 'yearly',
    priority: '0.5'
  }
];

async function fetchArticles() {
  try {
    console.log('Fetching articles from backend...');
    const response = await fetch(`${BACKEND_URL}/api/scienceai/articles?limit=1000`);
    
    if (!response.ok) {
      throw new Error(`Backend returned ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    return data.articles || [];
  } catch (error) {
    console.warn('Failed to fetch articles from backend:', error.message);
    return [];
  }
}

function generateSitemapXML(pages) {
  const now = new Date().toISOString();
  
  const urlEntries = pages.map(page => `  <url>
    <loc>${page.url}</loc>
    <lastmod>${page.lastmod || now}</lastmod>
    <changefreq>${page.changefreq}</changefreq>
    <priority>${page.priority}</priority>
  </url>`).join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urlEntries}
</urlset>`;
}

async function generateSitemap() {
  console.log('🗺️  Generating ScienceAI sitemap...');
  
  // Começar com páginas estáticas
  let allPages = [...STATIC_PAGES];
  
  // Tentar buscar artigos
  const articles = await fetchArticles();
  
  if (articles.length > 0) {
    console.log(`📄 Found ${articles.length} articles`);
    
    const articlePages = articles.map(article => ({
      url: `${BASE_URL}/article/${article.slug || article.id}`,
      lastmod: article.date || article.created_at || new Date().toISOString(),
      changefreq: 'weekly',
      priority: '0.9'
    }));
    
    allPages = [...allPages, ...articlePages];
  } else {
    console.log('📄 No articles found, using static pages only');
  }
  
  // Gerar XML
  const sitemapXML = generateSitemapXML(allPages);
  
  // Criar diretório se não existir
  const outputDir = path.dirname(OUTPUT_PATH);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  
  // Escrever arquivo
  fs.writeFileSync(OUTPUT_PATH, sitemapXML, 'utf8');
  
  console.log(`✅ Sitemap generated with ${allPages.length} URLs`);
  console.log(`📁 Saved to: ${OUTPUT_PATH}`);
  
  return allPages.length;
}

// Executar se chamado diretamente
if (require.main === module) {
  generateSitemap()
    .then(count => {
      console.log(`🎉 Sitemap generation completed! (${count} URLs)`);
      process.exit(0);
    })
    .catch(error => {
      console.error('❌ Sitemap generation failed:', error);
      process.exit(1);
    });
}

module.exports = { generateSitemap };
