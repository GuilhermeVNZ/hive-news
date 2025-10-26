import { describe, it, expect } from "vitest";
import { RankerService } from "../../apps/backend-cmmv/src/services/ranker.service";
import { SourceManagerService } from "../../apps/backend-cmmv/src/services/source-manager.service";
import { HTMLScraperService } from "../../apps/backend-cmmv/src/services/html-scraper.service";
import { MetadataExtractorService } from "../../apps/backend-cmmv/src/services/metadata-extractor.service";

describe("Load Tests", () => {
  const LOAD_SIZES = {
    small: 100,
    medium: 1000,
    large: 10000,
  };

  describe("RankerService Load", () => {
    it("should handle small load (100 articles)", () => {
      const service = new RankerService();
      const start = Date.now();

      for (let i = 0; i < LOAD_SIZES.small; i++) {
        service.calculateRank(
          new Date(),
          0.8,
          0.7,
          0.9,
          { views: 100, clicks: 10, timeOnPage: 60, bounceRate: 0.3 },
          [10, 15, 20]
        );
      }

      const duration = Date.now() - start;
      expect(duration).toBeLessThan(1000); // Complete in under 1 second
    });

    it("should handle medium load (1000 articles)", () => {
      const service = new RankerService();
      const start = Date.now();

      for (let i = 0; i < LOAD_SIZES.medium; i++) {
        service.calculateRank(
          new Date(),
          0.8,
          0.7,
          0.9,
          { views: 100, clicks: 10, timeOnPage: 60, bounceRate: 0.3 },
          [10, 15, 20]
        );
      }

      const duration = Date.now() - start;
      expect(duration).toBeLessThan(5000); // Complete in under 5 seconds
    });
  });

  describe("SourceManagerService Load", () => {
    it("should handle batch source operations", () => {
      const service = new SourceManagerService();
      const sources = Array.from({ length: 100 }, (_, i) => ({
        portal_id: `portal-${i % 5}`,
        url: `https://example.com/feed-${i}`,
        kind: "rss" as const,
      }));

      const start = Date.now();

      sources.forEach((source) => {
        try {
          service.validateSource(source as any);
        } catch (error) {
          // Skip invalid sources
        }
      });

      const duration = Date.now() - start;
      expect(duration).toBeLessThan(500); // Complete in under 500ms
    });
  });

  describe("HTMLScraperService Load", () => {
    it("should handle concurrent scraping operations", async () => {
      const service = new HTMLScraperService();
      const htmlSamples = Array.from({ length: 50 }, (_, i) => `
        <html>
          <head><title>Article ${i}</title></head>
          <body>
            <article>
              <h1>Title ${i}</h1>
              <p>Content ${i}</p>
            </article>
          </body>
        </html>
      `);

      const start = Date.now();
      const results = await Promise.all(
        htmlSamples.map((html, index) => service.scrapeContent(html, `https://example.com/article-${index}`))
      );
      const duration = Date.now() - start;

      expect(results).toHaveLength(50);
      expect(duration).toBeLessThan(5000); // Complete in under 5 seconds
    });
  });

  describe("MetadataExtractorService Load", () => {
    it("should handle batch metadata extraction", async () => {
      const service = new MetadataExtractorService();
      const htmlSamples = Array.from({ length: 100 }, (_, i) => `
        <html>
          <head>
            <title>Article ${i}</title>
            <meta name="author" content="Author ${i}" />
            <meta property="og:description" content="Description ${i}" />
          </head>
          <body>
            <p>Content ${i}</p>
          </body>
        </html>
      `);

      const start = Date.now();
      const results = await Promise.all(
        htmlSamples.map((html) => Promise.resolve(service.extractAllMetadata(html)))
      );
      const duration = Date.now() - start;

      expect(results).toHaveLength(100);
      expect(duration).toBeLessThan(3000); // Complete in under 3 seconds
    });
  });

  describe("Memory Usage", () => {
    it("should not leak memory with repeated operations", () => {
      const service = new RankerService();
      const iterations = 1000;

      const initialMemory = process.memoryUsage().heapUsed;

      for (let i = 0; i < iterations; i++) {
        service.calculateRank(
          new Date(),
          Math.random(),
          Math.random(),
          Math.random(),
          {
            views: Math.floor(Math.random() * 1000),
            clicks: Math.floor(Math.random() * 100),
            timeOnPage: Math.random() * 180,
            bounceRate: Math.random(),
          },
          [10, 15, 20, 25]
        );
      }

      const finalMemory = process.memoryUsage().heapUsed;
      const memoryIncrease = finalMemory - initialMemory;
      const memoryIncreaseMB = memoryIncrease / 1024 / 1024;

      // Should not increase by more than 10MB
      expect(memoryIncreaseMB).toBeLessThan(10);
    });
  });
});
