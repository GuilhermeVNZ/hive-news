import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { PublisherService } from "../../../apps/backend-cmmv/src/services/publisher.service";

describe("PublisherService", () => {
  let service: PublisherService;
  let consoleLogSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    service = new PublisherService();
    consoleLogSpy = vi.spyOn(console, "log").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("publishWebsite", () => {
    it("should generate URL from article content", async () => {
      const content = {
        title: "Test Article Title",
        dek: "Test description",
        body: "Article body content",
        lang: "en",
      };

      const url = await service.publishWebsite("article-123", content, "en");

      expect(url).toBe("/en/test-article-title");
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it("should generate multilingual URLs", async () => {
      const content = {
        title: "Artikel Test",
        dek: "Beschreibung",
        body: "Inhalt",
        lang: "de",
      };

      const url = await service.publishWebsite("article-456", content, "de");

      expect(url).toBe("/de/artikel-test");
    });

    it("should handle special characters in titles", async () => {
      const content = {
        title: "Article with @#$% special chars!",
        dek: "Test",
        body: "Content",
        lang: "en",
      };

      const url = await service.publishWebsite("article-789", content, "en");

      expect(url).toBe("/en/article-with-special-chars");
    });
  });

  describe("publishToX", () => {
    it("should format content for X.com", async () => {
      const content = {
        title: "Short Title",
        dek: "Short description",
      };

      const postId = await service.publishToX(content);

      expect(postId).toBe("x_post_placeholder_id");
      expect(consoleLogSpy).toHaveBeenCalledWith(
        "X.com post:",
        expect.stringContaining("Short Title")
      );
    });

    it("should truncate long content for X.com", async () => {
      const content = {
        title: "This is a very long title that will be truncated",
        dek: "This is a very long description that will also be truncated to fit within the 270 character limit for X.com posts",
      };

      const text = service["formatForX"](content.title, content.dek);

      expect(text.length).toBeLessThanOrEqual(270);
    });
  });

  describe("publishToLinkedIn", () => {
    it("should format content for LinkedIn", async () => {
      const content = {
        title: "Professional Title",
        dek: "Professional description",
      };

      const postId = await service.publishToLinkedIn(content);

      expect(postId).toBe("linkedin_post_placeholder_id");
      expect(consoleLogSpy).toHaveBeenCalledWith(
        "LinkedIn post:",
        expect.stringContaining("Professional Title")
      );
    });
  });

  describe("publishAll", () => {
    it("should publish to all targets successfully", async () => {
      const content = {
        title: "Article Title",
        dek: "Description",
        body: "Body content",
        coverImage: "https://example.com/image.jpg",
        lang: "en",
      };

      const targets = {
        website: true,
        x_com: true,
        linkedin: true,
      };

      const result = await service.publishAll("article-123", content, targets, "en");

      expect(result.success).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.publishedUrl).toBeTruthy();
      expect(result.xPostId).toBeTruthy();
      expect(result.linkedinPostId).toBeTruthy();
    });

    it("should handle partial publishing", async () => {
      const content = {
        title: "Article Title",
        dek: "Description",
        body: "Body",
        lang: "en",
      };

      const targets = {
        website: true,
        x_com: false,
        linkedin: false,
      };

      const result = await service.publishAll("article-123", content, targets, "en");

      expect(result.publishedUrl).toBeTruthy();
      expect(result.xPostId).toBeUndefined();
      expect(result.linkedinPostId).toBeUndefined();
    });
  });

  describe("generateSitemap", () => {
    it("should generate valid XML sitemap", async () => {
      const articles = [
        { url: "/en/article-1", lastmod: new Date("2024-01-01"), priority: 0.8 },
        { url: "/en/article-2", lastmod: new Date("2024-01-02"), priority: 0.9 },
      ];

      const sitemap = await service.generateSitemap(articles);

      expect(sitemap).toContain('<?xml version="1.0" encoding="UTF-8"?>');
      expect(sitemap).toContain('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">');
      expect(sitemap).toContain("/en/article-1");
      expect(sitemap).toContain("/en/article-2");
      expect(sitemap).toContain("</urlset>");
    });

    it("should handle empty article list", async () => {
      const sitemap = await service.generateSitemap([]);

      expect(sitemap).toContain('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">');
      expect(sitemap).toContain("</urlset>");
    });
  });
});
