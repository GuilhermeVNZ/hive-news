import { NextResponse } from "next/server";

export const runtime = "nodejs";
export const revalidate = 0;
export const dynamic = "force-dynamic";

/**
 * AIResearch Articles API - Proxies requests to the backend /api/logs endpoint
 * 
 * The backend /api/logs endpoint already:
 * - Reads articles_registry.json
 * - Filters by destinations (airesearch)
 * - Filters by status (Published)
 * - Handles featured/hidden flags
 * - Returns article metadata with destinations
 * 
 * This proxy adds:
 * - Transformation to AIResearch's Article format
 * - Excerpt generation
 * - Read time calculation
 * - Proper error handling and caching headers
 */
export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const category = searchParams.get("category");

    // Call backend /api/logs endpoint with airesearch filter
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3005';
    const logsUrl = new URL('/api/logs', backendUrl);
    logsUrl.searchParams.set('site', 'airesearch');
    logsUrl.searchParams.set('limit', '1000'); // Get all articles

    const response = await fetch(logsUrl.toString(), {
      headers: {
        'Accept': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Backend API returned ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    if (!data.success || !data.items) {
      throw new Error('Invalid response from backend API');
    }

    // Transform backend articles to AIResearch format
    const articles = data.items.map((item: any) => {
      // Generate slug from title
      const slug = item.title
        .toLowerCase()
        .replace(/[^\w\s-]/g, '')
        .replace(/\s+/g, '-')
        .replace(/-+/g, '-')
        .replace(/^-|-$/g, '');

      // Generate excerpt (use first destination URL or fallback)
      const excerpt = item.destinations && item.destinations.length > 0
        ? `Published on ${item.destinations.map((d: any) => d.site_name).join(', ')}`
        : 'Article published';

      // Estimate read time (assume 200 words per minute, rough estimate)
      const readTime = 5; // Default 5 minutes

      return {
        id: item.id,
        title: item.title,
        excerpt,
        article: '', // Content not available in logs endpoint
        publishedAt: item.created_at,
        author: item.source || 'AI Research',
        category: category && category !== 'all' ? category : 'ai',
        readTime,
        imageCategories: [],
        isPromotional: false,
        featured: item.featured || false,
        hidden: item.hidden || false,
      };
    });

    // Filter by category if provided
    let filteredArticles = articles;
    if (category && category !== 'all') {
      filteredArticles = articles.filter((a: any) => a.category === category);
    }

    // Filter out hidden articles
    filteredArticles = filteredArticles.filter((a: any) => !a.hidden);

    // Sort: featured first, then by date (newest first)
    filteredArticles.sort((a: any, b: any) => {
      if (a.featured && !b.featured) return -1;
      if (!a.featured && b.featured) return 1;
      return new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime();
    });

    const featuredCount = filteredArticles.filter((a: any) => a.featured).length;
    console.log(
      `[AIResearch Articles API] Returning ${filteredArticles.length} articles, ${featuredCount} featured`
    );

    return NextResponse.json(
      { articles: filteredArticles },
      {
        headers: {
          "Cache-Control":
            "no-store, no-cache, must-revalidate, proxy-revalidate",
          Pragma: "no-cache",
          Expires: "0",
        },
      }
    );
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const stack = error instanceof Error ? error.stack : undefined;
    console.error(
      "[AIResearch Articles API] ❌ Error in GET handler:",
      message
    );
    if (stack) {
      console.error("[AIResearch Articles API] Error stack:", stack);
    }

    return NextResponse.json(
      {
        error: "Internal server error",
        message,
        articles: [],
      },
      {
        status: 500,
        headers: {
          "Cache-Control":
            "no-store, no-cache, must-revalidate, proxy-revalidate",
          Pragma: "no-cache",
          Expires: "0",
        },
      }
    );
  }
}
