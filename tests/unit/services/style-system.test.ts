import { describe, it, expect } from "vitest";
import { StyleSystemService } from "../../../apps/backend-cmmv/src/services/style-system.service";

describe("StyleSystemService", () => {
  let service: StyleSystemService;

  beforeAll(() => {
    service = new StyleSystemService();
  });

  describe("getPreset", () => {
    it("should return scientific style preset", () => {
      const preset = service.getPreset("scientific");

      expect(preset).toBeDefined();
      expect(preset?.name).toBe("scientific");
      expect(preset?.tone).toContain("Academic");
      expect(preset?.keywords).toContain("research");
    });

    it("should return tech style preset", () => {
      const preset = service.getPreset("tech");

      expect(preset).toBeDefined();
      expect(preset?.name).toBe("tech");
      expect(preset?.tone).toContain("Technical");
      expect(preset?.keywords).toContain("technology");
    });

    it("should return policy style preset", () => {
      const preset = service.getPreset("policy");

      expect(preset).toBeDefined();
      expect(preset?.name).toBe("policy");
      expect(preset?.tone).toContain("Analytical");
      expect(preset?.keywords).toContain("policy");
    });

    it("should return undefined for invalid style", () => {
      const preset = service.getPreset("invalid" as any);

      expect(preset).toBeUndefined();
    });
  });

  describe("isValidStyle", () => {
    it("should validate scientific style", () => {
      expect(service.isValidStyle("scientific")).toBe(true);
    });

    it("should validate tech style", () => {
      expect(service.isValidStyle("tech")).toBe(true);
    });

    it("should validate policy style", () => {
      expect(service.isValidStyle("policy")).toBe(true);
    });

    it("should reject invalid styles", () => {
      expect(service.isValidStyle("invalid")).toBe(false);
      expect(service.isValidStyle("")).toBe(false);
    });
  });

  describe("getAllStyles", () => {
    it("should return all available styles", () => {
      const styles = service.getAllStyles();

      expect(styles).toContain("scientific");
      expect(styles).toContain("tech");
      expect(styles).toContain("policy");
      expect(styles).toHaveLength(3);
    });
  });

  describe("Preset structure", () => {
    it("should have proper structure for scientific preset", () => {
      const preset = service.getPreset("scientific");

      expect(preset).toHaveProperty("name");
      expect(preset).toHaveProperty("tone");
      expect(preset).toHaveProperty("format");
      expect(preset).toHaveProperty("structure");
      expect(preset).toHaveProperty("keywords");

      expect(preset?.structure).toContain("Abstract");
      expect(preset?.structure).toContain("References");
    });

    it("should have proper structure for tech preset", () => {
      const preset = service.getPreset("tech");

      expect(preset).toHaveProperty("name");
      expect(preset?.structure).toContain("Problem");
      expect(preset?.structure).toContain("Solution");
    });

    it("should have proper structure for policy preset", () => {
      const preset = service.getPreset("policy");

      expect(preset).toHaveProperty("name");
      expect(preset?.structure).toContain("Context");
      expect(preset?.structure).toContain("Recommendations");
    });
  });

  describe("Keywords", () => {
    it("should include relevant keywords for scientific style", () => {
      const preset = service.getPreset("scientific");

      expect(preset?.keywords).toContain("research");
      expect(preset?.keywords).toContain("methodology");
      expect(preset?.keywords).toContain("hypothesis");
    });

    it("should include relevant keywords for tech style", () => {
      const preset = service.getPreset("tech");

      expect(preset?.keywords).toContain("technology");
      expect(preset?.keywords).toContain("implementation");
      expect(preset?.keywords).toContain("development");
    });

    it("should include relevant keywords for policy style", () => {
      const preset = service.getPreset("policy");

      expect(preset?.keywords).toContain("policy");
      expect(preset?.keywords).toContain("regulation");
      expect(preset?.keywords).toContain("stakeholders");
    });
  });

  describe("Integration scenarios", () => {
    it("should handle full workflow: get all styles and validate", () => {
      const styles = service.getAllStyles();

      expect(styles.length).toBeGreaterThan(0);

      for (const style of styles) {
        expect(service.isValidStyle(style)).toBe(true);
        const preset = service.getPreset(style);
        expect(preset).toBeDefined();
        expect(preset?.name).toBe(style);
      }
    });
  });
});
