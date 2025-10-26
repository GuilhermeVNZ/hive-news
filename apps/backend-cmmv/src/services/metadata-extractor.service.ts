/**
 * Metadata Extractor Service
 * Extracts title, authors, abstract, date from documents
 */

export interface ExtractedMetadata {
  title: string;
  authors: string[];
  abstract: string;
  sourceUrl: string;
  publishedAt: Date;
}

export class MetadataExtractorService {
  /**
   * Extract title from document
   */
  async extractTitle(html: string): Promise<string> {
    // Try multiple selectors
    const titleSelectors = [
      'meta[property="og:title"]',
      'meta[name="twitter:title"]',
      "title",
      "h1",
      '[role="heading"]',
    ];

    for (const _selector of titleSelectors) {
      const regex = new RegExp(
        `<meta property="og:title" content="([^"]+)"|<meta name="twitter:title" content="([^"]+)"|<title>([^<]+)</title>|<h1[^>]*>([^<]+)</h1>`,
        "i"
      );
      const match = html.match(regex);
      if (match) return match[1] || match[2] || match[3] || match[4];
    }

    return "Untitled";
  }

  /**
   * Extract authors from document
   */
  async extractAuthors(html: string): Promise<string[]> {
    const authors: string[] = [];

    // Try meta tags
    const metaAuthorRegex =
      /<meta[^>]+name=["'](author|creator|dc.creator)["']\s+content=["']([^"']+)["']/gi;
    let match;
    while ((match = metaAuthorRegex.exec(html)) !== null) {
      if (match[2]) authors.push(match[2]);
    }

    // Try JSON-LD
    const jsonLdRegex = /"author"[:\s]*{?[^}]*"name"[:\s]*"([^"]+)"/gi;
    while ((match = jsonLdRegex.exec(html)) !== null) {
      if (match[1]) authors.push(match[1]);
    }

    return [...new Set(authors)];
  }

  /**
   * Extract abstract/description from document
   */
  async extractAbstract(html: string): Promise<string> {
    // Try meta description
    const metaDescRegex = /<meta[^>]+name=["']description["']\s+content=["']([^"']+)["']/i;
    const match = html.match(metaDescRegex);
    if (match) return match[1];

    // Try Open Graph description
    const ogDescRegex = /<meta[^>]+property=["']og:description["']\s+content=["']([^"']+)["']/i;
    const ogMatch = html.match(ogDescRegex);
    if (ogMatch) return ogMatch[1];

    // Try first paragraph
    const pRegex = /<p[^>]*>([^<]{100,500})/i;
    const pMatch = html.match(pRegex);
    if (pMatch) return pMatch[1].substring(0, 300);

    return "";
  }

  /**
   * Extract publication date from document
   */
  async extractPublishedDate(html: string): Promise<Date> {
    // Try meta tags
    const dateRegex =
      /<meta[^>]+property=["']article:published_time["']\s+content=["']([^"']+)["']|<meta[^>]+name=["'](publication_date|date)["']\s+content=["']([^"']+)["']|<time[^>]*datetime=["']([^"']+)["']/i;
    const match = html.match(dateRegex);
    if (match) {
      const dateStr = match[1] || match[3] || match[4];
      const parsed = new Date(dateStr);
      if (!isNaN(parsed.getTime())) return parsed;
    }

    return new Date();
  }

  /**
   * Extract all metadata from document
   */
  async extractAllMetadata(html: string, sourceUrl: string): Promise<ExtractedMetadata> {
    return {
      title: await this.extractTitle(html),
      authors: await this.extractAuthors(html),
      abstract: await this.extractAbstract(html),
      sourceUrl,
      publishedAt: await this.extractPublishedDate(html),
    };
  }

  /**
   * Normalize field formats
   */
  normalizeMetadata(metadata: ExtractedMetadata): ExtractedMetadata {
    return {
      ...metadata,
      title: this.normalizeTitle(metadata.title),
      authors: metadata.authors.map((a) => this.normalizeAuthor(a)),
      abstract: this.normalizeAbstract(metadata.abstract),
    };
  }

  private normalizeTitle(title: string): string {
    return title.trim().replace(/\s+/g, " ");
  }

  private normalizeAuthor(author: string): string {
    return author.trim();
  }

  private normalizeAbstract(abstract: string): string {
    return abstract.trim().replace(/\s+/g, " ").substring(0, 500);
  }
}
