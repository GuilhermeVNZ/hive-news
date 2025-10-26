import * as cheerio from "cheerio";

/**
 * HTML Scraper Service
 * Extracts content from HTML pages using cheerio
 */

export interface ScrapedContent {
  title: string;
  content: string;
  metadata: {
    description?: string;
    author?: string;
    date?: string;
    tags?: string[];
  };
  html: string;
}

export class HTMLScraperService {
  /**
   * Scrape content from HTML string
   */
  async scrapeContent(html: string, _url?: string): Promise<ScrapedContent> {
    const $ = cheerio.load(html);

    // Extract title
    const title =
      $("title").text() ||
      $("h1").first().text() ||
      $('meta[property="og:title"]').attr("content") ||
      "";

    // Extract main content
    let content = "";

    // Try to find article content using common selectors
    const articleSelectors = [
      "article",
      '[role="article"]',
      ".article",
      ".post",
      ".content",
      "main",
    ];

    for (const selector of articleSelectors) {
      const element = $(selector).first();
      if (element.length > 0) {
        content = element.text();
        break;
      }
    }

    // Fallback: get body content
    if (!content) {
      content = $("body").text();
    }

    // Extract metadata
    const metadata = {
      description:
        $('meta[name="description"]').attr("content") ||
        $('meta[property="og:description"]').attr("content") ||
        "",
      author: $('meta[name="author"]').attr("content") || $('[rel="author"]').attr("content") || "",
      date:
        $('meta[property="article:published_time"]').attr("content") ||
        $("time[datetime]").attr("datetime") ||
        $('[itemprop="datePublished"]').attr("content") ||
        "",
      tags:
        $('meta[property="article:tag"]')
          .map((_i, el) => $(el).attr("content"))
          .get() || [],
    };

    return {
      title,
      content,
      metadata,
      html: $.html(),
    };
  }

  /**
   * Extract article content from HTML
   */
  async extractArticleContent(html: string): Promise<string> {
    const $ = cheerio.load(html);

    // Remove unwanted elements
    $("script, style, nav, header, footer, aside, .sidebar, .navigation, .menu").remove();

    // Find main article
    const article = $("article").first();

    if (article.length > 0) {
      return article.text();
    }

    // Fallback: get body text
    return $("body").text();
  }

  /**
   * Extract metadata from HTML
   */
  async extractMetadata(html: string): Promise<Record<string, any>> {
    const $ = cheerio.load(html);

    const metadata: Record<string, any> = {};

    // Extract meta tags
    $("meta").each((_i, el) => {
      const name = $(el).attr("name") || $(el).attr("property") || $(el).attr("itemprop");
      const content = $(el).attr("content");

      if (name && content) {
        metadata[name] = content;
      }
    });

    return metadata;
  }

  /**
   * Extract links from HTML
   */
  async extractLinks(
    html: string,
    baseUrl?: string
  ): Promise<Array<{ text: string; url: string }>> {
    const $ = cheerio.load(html);
    const links: Array<{ text: string; url: string }> = [];

    $("a").each((_i, el) => {
      const href = $(el).attr("href");
      const text = $(el).text().trim();

      if (href) {
        const url = baseUrl && !href.startsWith("http") ? new URL(href, baseUrl).toString() : href;

        links.push({ text, url });
      }
    });

    return links;
  }

  /**
   * Clean and normalize HTML content
   */
  async cleanContent(content: string): Promise<string> {
    return content.replace(/\s+/g, " ").replace(/\n+/g, "\n").trim();
  }
}
