import { NextResponse } from "next/server";
import { getArticles } from "@/lib/articles";

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
    const category = searchParams.get("category") ?? undefined;

    const { articles, hasMore, total } = await getArticles(category);
    const featuredCount = articles.filter((a) => a.featured).length;
    console.log(
      `[AIResearch Articles API] Returning ${articles.length}/${total} articles (hasMore: ${hasMore}), ${featuredCount} featured`,
    );

    return NextResponse.json(
      { articles },
      {
        headers: {
          "Cache-Control":
            "no-store, no-cache, must-revalidate, proxy-revalidate",
          Pragma: "no-cache",
          Expires: "0",
        },
      },
    );
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const stack = error instanceof Error ? error.stack : undefined;

    console.error("[AIResearch Articles API] ❌ Error in GET handler:", message);
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
      },
    );
  }
}
