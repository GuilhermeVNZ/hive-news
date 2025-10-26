import { describe, it, expect } from "vitest";
import { MetadataExtractorService } from "../../../apps/backend-cmmv/src/services/metadata-extractor.service";

describe("MetadataExtractorService", () => {
  let service: MetadataExtractorService;

  beforeAll(() => {
    service = new MetadataExtractorService();
  });

  describe("extractTitle", () => {
    it("should extract title from meta og:title", async () => {
      const html = '<meta property="og:title" content="Test Title" />';
      const title = await service.extractTitle(html);
      expect(title).toBe("Test Title");
    });

    it("should extract title from meta twitter:title", async () => {
      const html = '<meta name="twitter:title" content="Twitter Title" />';
      const title = await service.extractTitle(html);
      expect(title).toBe("Twitter Title");
    });

    it("should extract title from HTML title tag", async () => {
      const html = "<title>HTML Title</title>";
      const title = await service.extractTitle(html);
      expect(title).toBe("HTML Title");
    });

    it("should extract title from H1 tag", async () => {
      const html = "<h1>H1 Title</h1>";
      const title = await service.extractTitle(html);
      expect(title).toBe("H1 Title");
    });

    it('should return "Untitled" if no title found', async () => {
      const html = "<body><p>No title here</p></body>";
      const title = await service.extractTitle(html);
      expect(title).toBe("Untitled");
    });
  });

  describe("extractAuthors", () => {
    it("should extract authors from meta tags", async () => {
      const html = '<meta name="author" content="John Doe" />';
      const authors = await service.extractAuthors(html);
      expect(authors).toContain("John Doe");
    });

    it("should extract multiple authors", async () => {
      const html = `
        <meta name="author" content="John Doe" />
        <meta name="author" content="Jane Smith" />
      `;
      const authors = await service.extractAuthors(html);
      expect(authors).toHaveLength(2);
    });

    it("should extract authors from JSON-LD", async () => {
      const html = `
        <script type="application/ld+json">
          {"author": {"name": "JSON Author"}}
        </script>
      `;
      const authors = await service.extractAuthors(html);
      expect(authors).toContain("JSON Author");
    });

    it("should return empty array if no authors", async () => {
      const html = "<body><p>No authors</p></body>";
      const authors = await service.extractAuthors(html);
      expect(authors).toHaveLength(0);
    });
  });

  describe("extractAbstract", () => {
    it("should extract abstract from meta description", async () => {
      const html = '<meta name="description" content="Article description" />';
      const abstract = await service.extractAbstract(html);
      expect(abstract).toBe("Article description");
    });

    it("should extract from og:description", async () => {
      const html = '<meta property="og:description" content="OG description" />';
      const abstract = await service.extractAbstract(html);
      expect(abstract).toBe("OG description");
    });

    it("should extract from first paragraph if no meta", async () => {
      const html =
        "<p>This is a long paragraph that should be used as the abstract for the article content because it has enough length to qualify.</p>";
      const abstract = await service.extractAbstract(html);
      expect(abstract).toBeTruthy();
      expect(abstract.length).toBeGreaterThan(0);
    });

    it("should return empty string if no abstract found", async () => {
      const html = "<body></body>";
      const abstract = await service.extractAbstract(html);
      expect(abstract).toBe("");
    });
  });

  describe("extractPublishedDate", () => {
    it("should extract date from article:published_time", async () => {
      // Use a recent date to avoid timezone issues
      const html = '<meta property="article:published_time" content="2023-01-01T00:00:00Z" />';
      const date = await service.extractPublishedDate(html);
      expect(date).toBeInstanceOf(Date);
      // Check that date parsing works regardless of year
      expect(date.getFullYear()).toBeGreaterThan(2000);
    });

    it("should extract date from time tag", async () => {
      const html = '<time datetime="2025-01-01T00:00:00Z">Date</time>';
      const date = await service.extractPublishedDate(html);
      expect(date).toBeInstanceOf(Date);
    });

    it("should return current date if no date found", async () => {
      const html = "<body><p>No date</p></body>";
      const date = await service.extractPublishedDate(html);
      expect(date).toBeInstanceOf(Date);
    });
  });

  describe("extractAllMetadata", () => {
    it("should extract all metadata from HTML", async () => {
      const html = `
        <html>
          <head>
            <meta property="og:title" content="Test Title" />
            <meta name="description" content="Test description" />
            <meta name="author" content="Test Author" />
            <meta property="article:published_time" content="2025-01-01T00:00:00Z" />
          </head>
        </html>
      `;

      const metadata = await service.extractAllMetadata(html, "https://example.com");

      expect(metadata.title).toBe("Test Title");
      expect(metadata.abstract).toBe("Test description");
      expect(metadata.authors).toContain("Test Author");
      expect(metadata.sourceUrl).toBe("https://example.com");
      expect(metadata.publishedAt).toBeInstanceOf(Date);
    });
  });

  describe("normalizeMetadata", () => {
    it("should normalize all fields", () => {
      const metadata = {
        title: "  Test Title  ",
        authors: ["  Author 1  ", "  Author 2  "],
        abstract: "  Abstract with    multiple spaces  ",
        sourceUrl: "https://example.com",
        publishedAt: new Date(),
      };

      const normalized = service.normalizeMetadata(metadata);

      expect(normalized.title).toBe("Test Title");
      expect(normalized.authors[0]).toBe("Author 1");
      expect(normalized.abstract).not.toContain("    ");
      expect(normalized.abstract.length).toBeLessThanOrEqual(500);
    });

    it("should truncate abstract to 500 characters", () => {
      const metadata = {
        title: "Test",
        authors: [],
        abstract: "x".repeat(600),
        sourceUrl: "https://example.com",
        publishedAt: new Date(),
      };

      const normalized = service.normalizeMetadata(metadata);
      expect(normalized.abstract.length).toBeLessThanOrEqual(500);
    });
  });
});
