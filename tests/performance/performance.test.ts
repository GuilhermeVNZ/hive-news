import { describe, it, expect } from "vitest";
import { RankerService } from "../../apps/backend-cmmv/src/services/ranker.service";
import { CronValidatorService } from "../../apps/backend-cmmv/src/services/cron-validator.service";
import { MetadataExtractorService } from "../../apps/backend-cmmv/src/services/metadata-extractor.service";
import { HTMLScraperService } from "../../apps/backend-cmmv/src/services/html-scraper.service";
import { SourceManagerService } from "../../apps/backend-cmmv/src/services/source-manager.service";

describe("Performance Tests", () => {
  // Performance thresholds
  const RANKER_THRESHOLD = 10; // ms
  const CRON_VALIDATOR_THRESHOLD = 5; // ms
  const METADATA_EXTRACTOR_THRESHOLD = 20; // ms
  const HTML_SCRAPER_THRESHOLD = 50; // ms
  const SOURCE_HASH_THRESHOLD = 2; // ms

  describe("RankerService Performance", () => {
    it("should calculate rank for 100 articles in <10ms per article", () => {
      const service = new RankerService();
      const start = Date.now();

      for (let i = 0; i < 100; i++) {
        const engagement = {
          views: Math.floor(Math.random() * 1000),
          clicks: Math.floor(Math.random() * 100),
          timeOnPage: Math.random() * 180,
          bounceRate: Math.random(),
        };

        service.calculateRank(
          new Date(),
          0.8,
          0.7,
          0.9,
          engagement,
          [10, 15, 20]
        );
      }

      const duration = Date.now() - start;
      const avgDuration = duration / 100;

      expect(avgDuration).toBeLessThan(RANKER_THRESHOLD);
      expect(duration).toBeLessThan(1000); // Total under 1 second
    });
  });

  describe("CronValidatorService Performance", () => {
    it("should validate 1000 cron expressions in <5ms per expression", () => {
      const service = new CronValidatorService();
      const expressions = [
        "0 * * * *",
        "*/15 * * * *",
        "0 0 * * *",
        "0 0 1 * *",
        "0 0 * * 0",
      ];

      const start = Date.now();

      for (let i = 0; i < 1000; i++) {
        const expr = expressions[i % expressions.length];
        service.validateCronSyntax(expr);
      }

      const duration = Date.now() - start;
      const avgDuration = duration / 1000;

      expect(avgDuration).toBeLessThan(CRON_VALIDATOR_THRESHOLD);
    });
  });

  describe("MetadataExtractorService Performance", () => {
    it("should extract metadata from HTML in <20ms", async () => {
      const service = new MetadataExtractorService();
      const html = `
        <html>
          <head>
            <title>Test Article</title>
            <meta name="author" content="John Doe" />
            <meta property="og:description" content="Article description" />
          </head>
          <body>
            <article>
              <h1>Test Title</h1>
              <p>Article content</p>
            </article>
          </body>
        </html>
      `;

      const start = Date.now();
      const metadata = await service.extractAllMetadata(html);
      const duration = Date.now() - start;

      expect(duration).toBeLessThan(METADATA_EXTRACTOR_THRESHOLD);
      expect(metadata.title).toBeDefined();
    });
  });

  describe("HTMLScraperService Performance", () => {
    it("should scrape content from HTML in <50ms", async () => {
      const service = new HTMLScraperService();
      const html = `
        <html>
          <head><title>Test</title></head>
          <body>
            <article>
              <h1>Title</h1>
              <p>Content paragraph with text.</p>
              <p>Another paragraph.</p>
            </article>
          </body>
        </html>
      `;

      const start = Date.now();
      const result = await service.scrapeContent(html, "https://example.com");
      const duration = Date.now() - start;

      expect(duration).toBeLessThan(HTML_SCRAPER_THRESHOLD);
      expect(result.title).toBeDefined();
      expect(result.content).toBeDefined();
    });
  });

  describe("SourceManagerService Performance", () => {
    it("should generate URL hash efficiently", () => {
      const service = new SourceManagerService();
      const url = "https://example.com/article";

      const start = Date.now();
      const hash = service.generateURLHash(url);
      const duration = Date.now() - start;

      expect(duration).toBeLessThanOrEqual(SOURCE_HASH_THRESHOLD);
      expect(hash).toBeDefined();
      expect(typeof hash).toBe("string");
    });

    it("should handle 1000 hash operations efficiently", () => {
      const service = new SourceManagerService();
      const start = Date.now();

      for (let i = 0; i < 1000; i++) {
        service.generateURLHash(`https://example.com/article-${i}`);
      }

      const duration = Date.now() - start;
      const avgDuration = duration / 1000;

      expect(avgDuration).toBeLessThan(1); // <1ms per hash
      expect(duration).toBeLessThan(100); // Total <100ms
    });
  });

  describe("Concurrent Operations Performance", () => {
    it("should handle concurrent rank calculations", async () => {
      const service = new RankerService();
      const operations = Array.from({ length: 50 }, (_, i) => {
        return service.calculateRank(
          new Date(Date.now() - i * 3600000),
          0.8,
          0.7,
          0.9,
          {
            views: 100,
            clicks: 10,
            timeOnPage: 60,
            bounceRate: 0.3,
          },
          [10, 15, 20]
        );
      });

      const start = Date.now();
      const results = await Promise.all(operations);
      const duration = Date.now() - start;

      expect(results).toHaveLength(50);
      expect(duration).toBeLessThan(100); // All operations complete in <100ms
    });
  });
});
