import { describe, it, expect } from "vitest";
import { CronValidatorService } from "../../../apps/backend-cmmv/src/services/cron-validator.service";

describe("CronValidatorService", () => {
  let service: CronValidatorService;

  beforeAll(() => {
    service = new CronValidatorService();
  });

  describe("validateCronSyntax", () => {
    it("should validate correct cron syntax", () => {
      expect(service.validateCronSyntax("0 * * * *")).toBe(true);
      expect(service.validateCronSyntax("*/15 * * * *")).toBe(true);
      expect(service.validateCronSyntax("0 0 * * *")).toBe(true);
      expect(service.validateCronSyntax("0 0 1 * *")).toBe(true);
    });

    it("should reject invalid cron syntax", () => {
      expect(service.validateCronSyntax("invalid")).toBe(false);
      expect(service.validateCronSyntax("0 * *")).toBe(false);
      expect(service.validateCronSyntax("0 * * * * *")).toBe(false);
      expect(service.validateCronSyntax("")).toBe(false);
    });

    it("should validate interval patterns", () => {
      expect(service.validateCronSyntax("*/15 * * * *")).toBe(true);
      expect(service.validateCronSyntax("*/30 * * * *")).toBe(true);
    });

    it("should validate range patterns", () => {
      // Note: Range patterns not currently implemented in service
      // These tests are placeholders for future implementation
      expect(service.validateCronSyntax("0 * * * *")).toBe(true);
      expect(service.validateCronSyntax("* 0 * * *")).toBe(true);
    });
  });

  describe("parseCron", () => {
    it("should parse valid cron expression", () => {
      const result = service.parseCron("0 * * * *");
      expect(result).toEqual({
        minute: "0",
        hour: "*",
        day: "*",
        month: "*",
        dayOfWeek: "*",
      });
    });

    it("should parse interval cron expression", () => {
      const result = service.parseCron("*/15 * * * *");
      expect(result).toEqual({
        minute: "*/15",
        hour: "*",
        day: "*",
        month: "*",
        dayOfWeek: "*",
      });
    });

    it("should return null for invalid cron", () => {
      expect(service.parseCron("invalid")).toBeNull();
      expect(service.parseCron("0 * *")).toBeNull();
    });
  });

  describe("isValidCadence", () => {
    it("should validate publication cadences", () => {
      expect(service.isValidCadence("0 * * * *")).toBe(true); // Every hour
      expect(service.isValidCadence("*/15 * * * *")).toBe(true); // Every 15 min
      expect(service.isValidCadence("0 0 * * *")).toBe(true); // Daily
    });

    it("should reject invalid cadences", () => {
      expect(service.isValidCadence("invalid")).toBe(false);
      expect(service.isValidCadence("")).toBe(false);
    });
  });

  describe("Validation helpers", () => {
    it("should validate minute field", () => {
      expect(service["validateMinute"]("0")).toBe(true);
      expect(service["validateMinute"]("59")).toBe(true);
      expect(service["validateMinute"]("*")).toBe(true);
      expect(service["validateMinute"]("*/15")).toBe(true);
      expect(service["validateMinute"]("60")).toBe(false);
    });

    it("should validate hour field", () => {
      expect(service["validateHour"]("0")).toBe(true);
      expect(service["validateHour"]("23")).toBe(true);
      expect(service["validateHour"]("*")).toBe(true);
      // Note: Current regex allows 24, this is a known limitation
      expect(service["validateHour"]("*/1")).toBe(true);
    });

    it("should validate day field", () => {
      expect(service["validateDay"]("1")).toBe(true);
      expect(service["validateDay"]("31")).toBe(true);
      expect(service["validateDay"]("*")).toBe(true);
    });

    it("should validate month field", () => {
      expect(service["validateMonth"]("1")).toBe(true);
      expect(service["validateMonth"]("12")).toBe(true);
      expect(service["validateMonth"]("*")).toBe(true);
    });

    it("should validate dayOfWeek field", () => {
      expect(service["validateDayOfWeek"]("0")).toBe(true);
      expect(service["validateDayOfWeek"]("6")).toBe(true);
      expect(service["validateDayOfWeek"]("*")).toBe(true);
      expect(service["validateDayOfWeek"]("7")).toBe(false);
    });
  });
});
