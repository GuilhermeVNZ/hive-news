import { describe, it, expect } from "vitest";
import { HTMLScraperService } from "../../../apps/backend-cmmv/src/services/html-scraper.service";

describe("HTMLScraperService", () => {
  let service: HTMLScraperService;

  beforeAll(() => {
    service = new HTMLScraperService();
  });

  describe("scrapeContent", () => {
    it("should extract title and content from HTML", async () => {
      const html = `
        <html>
          <head><title>Test Article</title></head>
          <body>
            <article>
              <h1>Article Title</h1>
              <p>Article content here.</p>
            </article>
          </body>
        </html>
      `;

      const result = await service.scrapeContent(html);

      expect(result.title).toBe("Test Article");
      expect(result.content).toContain("Article content here");
      expect(result.html).toBeDefined();
    });

    it("should extract metadata from meta tags", async () => {
      const html = `
        <html>
          <head>
            <meta name="description" content="Article description" />
            <meta name="author" content="John Doe" />
            <meta property="article:published_time" content="2025-01-01T00:00:00Z" />
          </head>
          <body><p>Content</p></body>
        </html>
      `;

      const result = await service.scrapeContent(html);

      expect(result.metadata.description).toBe("Article description");
      expect(result.metadata.author).toBe("John Doe");
      expect(result.metadata.date).toBe("2025-01-01T00:00:00Z");
    });

    it("should fallback to H1 if title not found", async () => {
      const html = `
        <html>
          <body>
            <h1>Fallback Title</h1>
            <p>Content</p>
          </body>
        </html>
      `;

      const result = await service.scrapeContent(html);
      expect(result.title).toBe("Fallback Title");
    });

    it("should handle HTML without article tag", async () => {
      const html = `
        <html>
          <head><title>Test</title></head>
          <body>
            <main><p>Content here</p></main>
          </body>
        </html>
      `;

      const result = await service.scrapeContent(html);
      expect(result.content).toContain("Content here");
    });
  });

  describe("extractArticleContent", () => {
    it("should extract content from article tag", async () => {
      const html = `
        <article>
          <p>Article content paragraph 1.</p>
          <p>Article content paragraph 2.</p>
        </article>
      `;

      const result = await service.extractArticleContent(html);
      expect(result).toContain("Article content paragraph 1");
      expect(result).toContain("Article content paragraph 2");
    });

    it("should remove script and style tags", async () => {
      const html = `
        <article>
          <p>Article content</p>
          <script>alert('test');</script>
          <style>.hidden { display: none; }</style>
        </article>
      `;

      const result = await service.extractArticleContent(html);
      expect(result).toContain("Article content");
      expect(result).not.toContain("alert");
      expect(result).not.toContain("hidden");
    });

    it("should fallback to body content", async () => {
      const html = `
        <body>
          <p>Body content here</p>
        </body>
      `;

      const result = await service.extractArticleContent(html);
      expect(result).toContain("Body content here");
    });
  });

  describe("extractMetadata", () => {
    it("should extract all meta tags", async () => {
      const html = `
        <head>
          <meta name="description" content="Desc" />
          <meta property="og:title" content="OG Title" />
          <meta itemprop="author" content="Author" />
        </head>
        <body></body>
      `;

      const result = await service.extractMetadata(html);

      expect(result.description).toBe("Desc");
      expect(result["og:title"]).toBe("OG Title");
      expect(result.author).toBe("Author");
    });

    it("should return empty object if no meta tags", async () => {
      const html = "<body><p>No metadata</p></body>";
      const result = await service.extractMetadata(html);
      expect(Object.keys(result)).toHaveLength(0);
    });
  });

  describe("extractLinks", () => {
    it("should extract links from HTML", async () => {
      const html = `
        <body>
          <a href="https://example.com/page1">Link 1</a>
          <a href="https://example.com/page2">Link 2</a>
        </body>
      `;

      const result = await service.extractLinks(html);

      expect(result).toHaveLength(2);
      expect(result[0].url).toBe("https://example.com/page1");
      expect(result[0].text).toBe("Link 1");
    });

    it("should resolve relative URLs with baseUrl", async () => {
      const html = '<body><a href="/page1">Link</a></body>';
      const result = await service.extractLinks(html, "https://example.com");

      expect(result[0].url).toBe("https://example.com/page1");
    });

    it("should handle empty links", async () => {
      const html = "<body><p>No links</p></body>";
      const result = await service.extractLinks(html);
      expect(result).toHaveLength(0);
    });
  });

  describe("cleanContent", () => {
    it("should normalize whitespace", async () => {
      const content = "Multiple    spaces   and\n\n\nnewlines";
      const result = await service.cleanContent(content);

      expect(result).not.toContain("    ");
      expect(result.split("\n").length).toBeLessThanOrEqual(2);
    });

    it("should trim content", async () => {
      const content = "   content with spaces   ";
      const result = await service.cleanContent(content);

      expect(result).toBe("content with spaces");
    });
  });
});

