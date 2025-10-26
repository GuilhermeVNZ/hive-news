/**
 * Publisher Service
 * Publishes articles to website, X.com, LinkedIn
 */

export interface PublishingTarget {
  website: boolean;
  x_com: boolean;
  linkedin: boolean;
}

export interface PublishResult {
  publishedUrl: string;
  xPostId?: string;
  linkedinPostId?: string;
  success: boolean;
  errors: string[];
}

export class PublisherService {
  /**
   * Publish article to website
   */
  async publishWebsite(_articleId: string, content: any, lang: string): Promise<string> {
    const slug = this.generateSlug(content.title);
    const url = `/${lang}/${slug}`;

    // Generate HTML page with SEO
    const html = this.generateHTMLPage(content);

    // Upload to static hosting or database
    await this.savePage(url, html);

    return url;
  }

  /**
   * Publish to X.com (placeholder)
   */
  async publishToX(content: any, _imageUrl?: string): Promise<string> {
    const text = this.formatForX(content.title, content.dek);

    // X.com API integration (placeholder)
    console.log("X.com post:", text);
    return "x_post_placeholder_id";
  }

  /**
   * Publish to LinkedIn (placeholder)
   */
  async publishToLinkedIn(content: any, _imageUrl?: string): Promise<string> {
    const text = this.formatForLinkedIn(content.title, content.dek);

    // LinkedIn API integration (placeholder)
    console.log("LinkedIn post:", text);
    return "linkedin_post_placeholder_id";
  }

  /**
   * Publish to all targets
   */
  async publishAll(
    _articleId: string,
    content: any,
    targets: PublishingTarget,
    lang: string
  ): Promise<PublishResult> {
    const result: PublishResult = {
      publishedUrl: "",
      success: false,
      errors: [],
    };

    try {
      // Publish to website
      if (targets.website) {
        result.publishedUrl = await this.publishWebsite("article-id", content, lang);
      }

      // Publish to X.com
      if (targets.x_com) {
        try {
          result.xPostId = await this.publishToX(content, content.coverImage);
        } catch (error: any) {
          result.errors.push(`X.com: ${error.message}`);
        }
      }

      // Publish to LinkedIn
      if (targets.linkedin) {
        try {
          result.linkedinPostId = await this.publishToLinkedIn(content, content.coverImage);
        } catch (error: any) {
          result.errors.push(`LinkedIn: ${error.message}`);
        }
      }

      result.success = result.errors.length === 0;
    } catch (error: any) {
      result.errors.push(error.message);
    }

    return result;
  }

  /**
   * Generate XML sitemap
   */
  async generateSitemap(
    articles: Array<{ url: string; lastmod: Date; priority: number }>
  ): Promise<string> {
    let xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">`;

    for (const article of articles) {
      xml += `
  <url>
    <loc>${article.url}</loc>
    <lastmod>${article.lastmod.toISOString()}</lastmod>
    <priority>${article.priority}</priority>
  </url>`;
    }

    xml += "\n</urlset>";
    return xml;
  }

  private generateSlug(title: string): string {
    return title
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "");
  }

  private generateHTMLPage(content: any): string {
    return `
<!DOCTYPE html>
<html lang="${content.lang}">
<head>
  <meta charset="UTF-8">
  <title>${content.title}</title>
  <meta name="description" content="${content.dek}">
  <meta property="og:title" content="${content.title}">
  <meta property="og:description" content="${content.dek}">
  <meta property="og:image" content="${content.coverImage}">
</head>
<body>
  <article>
    <h1>${content.title}</h1>
    <p>${content.dek}</p>
    <div>${content.body}</div>
  </article>
</body>
</html>`;
  }

  private formatForX(title: string, dek: string): string {
    const maxLength = 270;
    const text = `${title}\n\n${dek}`.trim();
    return text.length > maxLength ? text.substring(0, 267) + "..." : text;
  }

  private formatForLinkedIn(title: string, dek: string): string {
    return `${title}\n\n${dek}\n\n[Read more →]`;
  }

  private async savePage(_url: string, _html: string): Promise<void> {
    // Save to static hosting or database
    console.log(`Saved page`);
  }
}
