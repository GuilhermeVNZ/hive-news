import { describe, it, expect, beforeEach } from "vitest";
import { SourceManagerService } from "../../../apps/backend-cmmv/src/services/source-manager.service";

describe("SourceManagerService", () => {
  let service: SourceManagerService;

  beforeEach(() => {
    service = new SourceManagerService();
  });

  describe("generateURLHash", () => {
    it("should generate consistent hash for same URL", () => {
      const url = "https://example.com/article";

      const hash1 = service.generateURLHash(url);
      const hash2 = service.generateURLHash(url);

      expect(hash1).toBe(hash2);
      expect(hash1).toHaveLength(64); // SHA-256 produces 64 hex chars
    });

    it("should generate different hashes for different URLs", () => {
      const hash1 = service.generateURLHash("https://example.com/article1");
      const hash2 = service.generateURLHash("https://example.com/article2");

      expect(hash1).not.toBe(hash2);
    });
  });

  describe("checkDuplicate", () => {
    it("should return false for new URLs", async () => {
      const isDuplicate = await service.checkDuplicate("portal-1", "https://example.com/article");

      expect(isDuplicate).toBe(false);
    });

    it("should return true for registered URLs", async () => {
      await service.registerSource("portal-1", "https://example.com/article");

      const isDuplicate = await service.checkDuplicate("portal-1", "https://example.com/article");

      expect(isDuplicate).toBe(true);
    });

    it("should differentiate between portals", async () => {
      await service.registerSource("portal-1", "https://example.com/article");

      const isDuplicate = await service.checkDuplicate("portal-2", "https://example.com/article");

      expect(isDuplicate).toBe(false);
    });
  });

  describe("createSource", () => {
    it("should create a new source", async () => {
      const source = await service.createSource({
        portal_id: "portal-1",
        url: "https://example.com/rss",
        kind: "rss",
      });

      expect(source.id).toBeDefined();
      expect(source.portal_id).toBe("portal-1");
      expect(source.url).toBe("https://example.com/rss");
      expect(source.kind).toBe("rss");
    });

    it("should generate ID if not provided", async () => {
      const source1 = await service.createSource({
        portal_id: "portal-1",
        url: "https://example.com/rss1",
        kind: "rss",
      });

      const source2 = await service.createSource({
        portal_id: "portal-1",
        url: "https://example.com/rss2",
        kind: "rss",
      });

      expect(source1.id).not.toBe(source2.id);
    });

    it("should throw error for duplicate URLs", async () => {
      await service.createSource({
        portal_id: "portal-1",
        url: "https://example.com/rss",
        kind: "rss",
      });

      await expect(
        service.createSource({
          portal_id: "portal-1",
          url: "https://example.com/rss",
          kind: "rss",
        })
      ).rejects.toThrow("Source already exists");
    });
  });

  describe("updateLastFetch", () => {
    it("should update last_fetch timestamp", () => {
      const source = {
        id: "source-1",
        portal_id: "portal-1",
        url: "https://example.com/rss",
        kind: "rss" as const,
        last_fetch: new Date("2024-01-01"),
      };

      const updated = service.updateLastFetch(source);

      expect(updated.last_fetch.getTime()).toBeGreaterThan(source.last_fetch.getTime());
    });
  });

  describe("validateSource", () => {
    it("should validate correct source", () => {
      const source = {
        portal_id: "portal-1",
        url: "https://example.com/rss",
        kind: "rss" as const,
      };

      const result = service.validateSource(source);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should reject source without portal_id", () => {
      const source = {
        url: "https://example.com/rss",
        kind: "rss" as const,
      };

      const result = service.validateSource(source);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("portal_id is required");
    });

    it("should reject source without url", () => {
      const source = {
        portal_id: "portal-1",
        kind: "rss" as const,
      };

      const result = service.validateSource(source);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("url is required");
    });

    it("should reject invalid URL format", () => {
      const source = {
        portal_id: "portal-1",
        url: "not-a-valid-url",
        kind: "rss" as const,
      };

      const result = service.validateSource(source);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("url must be a valid URL");
    });

    it("should reject invalid kind", () => {
      const source = {
        portal_id: "portal-1",
        url: "https://example.com/rss",
        kind: "invalid" as any,
      };

      const result = service.validateSource(source);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain("kind must be rss, api, or html");
    });
  });

  describe("scheduleFetches", () => {
    it("should schedule fetches for multiple sources", async () => {
      const sources = [
        {
          id: "source-1",
          portal_id: "portal-1",
          url: "https://example.com/rss",
          kind: "rss" as const,
          last_fetch: new Date(Date.now() - 3600000),
        },
        {
          id: "source-2",
          portal_id: "portal-1",
          url: "https://example.com/api",
          kind: "api" as const,
          last_fetch: new Date(Date.now() - 3600000),
        },
      ];

      const schedule = await service.scheduleFetches(sources);

      expect(schedule).toHaveLength(2);
      expect(schedule[0].scheduled).toBeInstanceOf(Date);
      expect(schedule[1].scheduled).toBeInstanceOf(Date);
    });

    it("should use different intervals for different source kinds", async () => {
      const sources = [
        {
          id: "rss-source",
          portal_id: "portal-1",
          url: "https://example.com/rss",
          kind: "rss" as const,
          last_fetch: new Date(Date.now() - 3600000),
        },
        {
          id: "html-source",
          portal_id: "portal-1",
          url: "https://example.com/html",
          kind: "html" as const,
          last_fetch: new Date(Date.now() - 3600000),
        },
      ];

      const schedule = await service.scheduleFetches(sources);

      expect(schedule[0].scheduled.getTime()).toBeLessThan(schedule[1].scheduled.getTime());
    });
  });

  describe("cleanupCache", () => {
    it("should not clear small cache", () => {
      // Add some entries
      service.registerSource("portal-1", "https://example.com/1");
      service.registerSource("portal-1", "https://example.com/2");

      service.cleanupCache();

      // Cache should still have entries
      expect(service["urlCache"].size).toBeGreaterThan(0);
    });
  });
});

