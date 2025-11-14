import { NextResponse } from "next/server";
import { getArticles } from "@/lib/articles";

export const runtime = "nodejs";
export const revalidate = 0;
export const dynamic = "force-dynamic";

/**
 * RSS Feed for AIResearch.news
 * Generates a valid RSS 2.0 feed from published articles
 */
export async function GET() {
  try {
    const articles = await getArticles();
    
    // Filter only published articles (not hidden)
    const publishedArticles = articles
      .filter((article) => !article.hidden)
      .sort((a, b) => {
        // Sort by published date, newest first
        return (
          new Date(b.publishedAt).getTime() - new Date(a.publishedAt).getTime()
        );
      })
      .slice(0, 50); // Limit to 50 most recent articles

    const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://www.airesearch.news";
    const feedUrl = `${siteUrl}/rss`;
    const now = new Date().toUTCString();

    // Generate RSS XML
    const rssXml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>AIResearch.news - Latest AI Research & News</title>
    <link>${siteUrl}</link>
    <description>Latest breakthroughs in artificial intelligence research, practical applications of ML/deep learning, and industry news with expert analysis.</description>
    <language>en-US</language>
    <lastBuildDate>${now}</lastBuildDate>
    <atom:link href="${feedUrl}" rel="self" type="application/rss+xml"/>
    <generator>AIResearch.news RSS Generator</generator>
    <webMaster>info@airesearch.news (AIResearch Team)</webMaster>
    <managingEditor>info@airesearch.news (AIResearch Team)</managingEditor>
    <copyright>Copyright ${new Date().getFullYear()} AIResearch.news. All rights reserved.</copyright>
    <ttl>60</ttl>
    ${publishedArticles
      .map((article) => {
        const articleUrl = `${siteUrl}/article/${encodeURIComponent(article.slug)}`;
        const pubDate = new Date(article.publishedAt).toUTCString();
        
        // Clean HTML from article text for description
        const description = article.excerpt
          .replace(/<[^>]*>/g, "")
          .replace(/&nbsp;/g, " ")
          .replace(/&amp;/g, "&")
          .replace(/&lt;/g, "<")
          .replace(/&gt;/g, ">")
          .replace(/&quot;/g, '"')
          .replace(/&#39;/g, "'")
          .trim()
          .substring(0, 500); // Limit description length

        // Clean HTML from article content
        const content = article.article
          .replace(/<[^>]*>/g, "")
          .replace(/&nbsp;/g, " ")
          .replace(/&amp;/g, "&")
          .replace(/&lt;/g, "<")
          .replace(/&gt;/g, ">")
          .replace(/&quot;/g, '"')
          .replace(/&#39;/g, "'")
          .trim();

        // Escape XML special characters
        const escapeXml = (text: string) => {
          return text
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
        };

        const escapedTitle = escapeXml(article.title);
        const escapedDescription = escapeXml(description);
        const escapedAuthor = escapeXml(article.author || "AIResearch Team");

        return `    <item>
      <title>${escapedTitle}</title>
      <link>${articleUrl}</link>
      <guid isPermaLink="true">${articleUrl}</guid>
      <description>${escapedDescription}</description>
      <content:encoded><![CDATA[${content}]]></content:encoded>
      <author>${escapedAuthor}</author>
      <pubDate>${pubDate}</pubDate>
      <category>${article.imageCategories[0] || "ai"}</category>
    </item>`;
      })
      .join("\n")}
  </channel>
</rss>`;

    return new NextResponse(rssXml, {
      status: 200,
      headers: {
        "Content-Type": "application/rss+xml; charset=utf-8",
        "Cache-Control": "public, max-age=3600, s-maxage=3600",
      },
    });
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : "Unknown error";
    console.error("[AIResearch RSS] ❌ Error generating RSS feed:", message);

    // Return empty RSS feed on error
    const emptyRss = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>AIResearch.news - Latest AI Research & News</title>
    <link>${process.env.NEXT_PUBLIC_SITE_URL || "https://www.airesearch.news"}</link>
    <description>Latest breakthroughs in artificial intelligence research</description>
    <language>en-US</language>
    <lastBuildDate>${new Date().toUTCString()}</lastBuildDate>
  </channel>
</rss>`;

    return new NextResponse(emptyRss, {
      status: 200,
      headers: {
        "Content-Type": "application/rss+xml; charset=utf-8",
      },
    });
  }
}


