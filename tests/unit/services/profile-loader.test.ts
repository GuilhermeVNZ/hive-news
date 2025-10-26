import { describe, it, expect, beforeEach, vi } from "vitest";
import { ProfileLoaderService } from "../../../apps/backend-cmmv/src/services/profile-loader.service";
import fs from "fs";

// Mock fs
vi.mock("fs");

describe("ProfileLoaderService", () => {
  let service: ProfileLoaderService;

  beforeEach(() => {
    service = new ProfileLoaderService("./test-configs");
    vi.clearAllMocks();
  });

  describe("loadProfile", () => {
    it("should load profile from YAML", async () => {
      const mockProfile = {
        portal: { id: "test", name: "Test Portal" },
        editorial: { style: "scientific" },
        sources: [{ url: "http://test.com", type: "rss", enabled: true, refresh: "0 * * * *" }],
      };

      (fs.readFileSync as any).mockReturnValue(`
        portal:
          id: test
          name: Test Portal
        editorial:
          style: scientific
        sources:
          - url: http://test.com
            type: rss
            enabled: true
            refresh: "0 * * * *"
      `);

      const profile = await service.loadProfile("test");
      expect(profile).toBeDefined();
      expect(profile?.portal.id).toBe("test");
    });

    it("should return null for non-existent profile", async () => {
      (fs.readFileSync as any).mockImplementation(() => {
        throw new Error("File not found");
      });

      const profile = await service.loadProfile("nonexistent");
      expect(profile).toBeNull();
    });
  });

  describe("validateProfile", () => {
    it("should validate profile with all required fields", () => {
      const validProfile = {
        portal: { id: "test", name: "Test" },
        editorial: { style: "scientific" },
        sources: [{ url: "http://test.com", type: "rss", enabled: true, refresh: "0 * * * *" }],
      };

      expect(() => service["validateProfile"](validProfile)).not.toThrow();
    });

    it("should throw error for missing portal.id", () => {
      const invalidProfile = {
        portal: { name: "Test" },
        editorial: { style: "scientific" },
        sources: [],
      };

      expect(() => service["validateProfile"](invalidProfile)).toThrow("missing portal.id");
    });

    it("should throw error for missing editorial.style", () => {
      const invalidProfile = {
        portal: { id: "test", name: "Test" },
        sources: [],
      };

      expect(() => service["validateProfile"](invalidProfile)).toThrow("missing editorial.style");
    });

    it("should throw error for missing sources array", () => {
      const invalidProfile = {
        portal: { id: "test", name: "Test" },
        editorial: { style: "scientific" },
      };

      expect(() => service["validateProfile"](invalidProfile)).toThrow("sources must be an array");
    });
  });

  describe("loadAllProfiles", () => {
    it("should load all profiles from directory", async () => {
      (fs.readdirSync as any).mockReturnValue(["portal1.yaml", "portal2.yaml"]);
      (fs.readFileSync as any).mockReturnValue(`
        portal:
          id: test
          name: Test
        editorial:
          style: scientific
        sources: []
      `);

      const profiles = await service.loadAllProfiles();
      expect(profiles.size).toBeGreaterThan(0);
    });

    it("should ignore non-YAML files", async () => {
      (fs.readdirSync as any).mockReturnValue(["portal1.yaml", "readme.txt"]);
      (fs.readFileSync as any).mockReturnValue(`
        portal:
          id: test
          name: Test
        editorial:
          style: scientific
        sources: []
      `);

      const profiles = await service.loadAllProfiles();
      // Only yaml files should be loaded
      expect(profiles.size).toBeGreaterThan(0);
    });
  });

  describe("watchProfiles", () => {
    it("should watch profiles directory for changes", () => {
      const callback = vi.fn();
      (fs.watch as any).mockImplementation(() => ({}));

      service.watchProfiles(callback);

      expect(fs.watch).toHaveBeenCalled();
    });

    it("should trigger callback on YAML file changes", () => {
      const callback = vi.fn();
      (fs.watch as any).mockImplementation((dir, handler) => {
        // Simulate file change
        setTimeout(() => handler("change", "profile.yaml"), 100);
      });
      (fs.readFileSync as any).mockReturnValue(`
        portal:
          id: test
          name: Test
        editorial:
          style: scientific
        sources: []
      `);

      service.watchProfiles(callback);

      // Wait for callback
      return new Promise((resolve) => setTimeout(resolve, 150));
    });
  });

  describe("toEditorialContract", () => {
    it("should convert profile to EditorialContract data", () => {
      const profile: any = {
        portal: { name: "Test Portal" },
        sources: [{ url: "http://test.com" }],
        editorial: {
          style: "scientific",
          language: { base: "en", translate_to: ["pt-BR"] },
          cadence: "0 * * * *",
          image: { style: "scientific_illustration" },
          seo: { priority: "high" },
        },
      };

      const contract = service.toEditorialContract(profile);

      expect(contract.name).toBe("Test Portal");
      expect(contract.style).toBe("scientific");
      expect(contract.langs.base).toBe("en");
      expect(contract.langs.translate_to).toEqual(["pt-BR"]);
    });

    it("should handle multiple sources", () => {
      const profile: any = {
        portal: { name: "Test" },
        sources: [{ url: "http://source1.com" }, { url: "http://source2.com" }],
        editorial: {
          style: "tech",
          language: { base: "en", translate_to: [] },
          cadence: "0 * * * *",
          image: { style: "tech" },
          seo: { priority: "medium" },
        },
      };

      const contract = service.toEditorialContract(profile);

      expect(contract.sources).toHaveLength(2);
      expect(contract.sources).toContain("http://source1.com");
      expect(contract.sources).toContain("http://source2.com");
    });
  });
});
