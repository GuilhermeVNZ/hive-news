import { describe, it, expect, beforeAll } from "vitest";
import { RankerService } from "../../../apps/backend-cmmv/src/services/ranker.service";

describe("RankerService", () => {
  let service: RankerService;

  beforeAll(() => {
    service = new RankerService();
  });

  it("should calculate freshness score", () => {
    const now = new Date();
    const recent = new Date(now.getTime() - 3600000); // 1 hour ago

    expect(service.calculateFreshness(recent)).toBeGreaterThan(0.9);
  });

  it("should calculate relevance score", () => {
    const score = service.calculateRelevance(0.8, 0.7, 0.9);
    expect(score).toBeGreaterThan(0);
    expect(score).toBeLessThanOrEqual(1);
  });

  it("should combine all ranking factors", () => {
    const publishedAt = new Date(Date.now() - 86400000); // 24 hours ago
    const engagement = {
      views: 100,
      clicks: 20,
      timeOnPage: 180,
      bounceRate: 0.3,
    };

    const result = service.calculateRank(
      publishedAt,
      0.8, // vectorSimilarity
      0.7, // keywordScore
      0.9, // contentQuality
      engagement,
      [10, 15, 20], // trendHistory
      undefined // no QA validation
    );

    expect(result.finalRank).toBeGreaterThan(0);
    expect(result.finalRank).toBeLessThanOrEqual(1);
  });

  describe("QA Feedback Loop", () => {
    it("should apply penalty for rejected articles", () => {
      const qaValidation = {
        isFactual: true,
        isNeutralTone: true,
        qualityScore: 0.8,
        status: "rejected" as const,
      };

      const penalty = service.calculateQAPenalty(qaValidation);
      expect(penalty).toBeLessThan(0.6); // Should be heavily penalized
    });

    it("should apply moderate penalty for pending articles", () => {
      const qaValidation = {
        isFactual: true,
        isNeutralTone: true,
        qualityScore: 0.9,
        status: "pending" as const,
      };

      const penalty = service.calculateQAPenalty(qaValidation);
      expect(penalty).toBeLessThan(0.95);
      expect(penalty).toBeGreaterThan(0.8);
    });

    it("should not penalize approved articles", () => {
      const qaValidation = {
        isFactual: true,
        isNeutralTone: true,
        qualityScore: 1.0,
        status: "approved" as const,
      };

      const penalty = service.calculateQAPenalty(qaValidation);
      expect(penalty).toBeCloseTo(1.0);
    });

    it("should penalize non-factual articles", () => {
      const qaValidation = {
        isFactual: false,
        isNeutralTone: true,
        qualityScore: 0.8,
        status: "approved" as const,
      };

      const penalty = service.calculateQAPenalty(qaValidation);
      expect(penalty).toBeLessThan(0.6); // 0.8 * 0.7 = 0.56
    });

    it("should apply QA penalty in final rank calculation", () => {
      const publishedAt = new Date();
      const engagement = { views: 100, clicks: 20, timeOnPage: 180, bounceRate: 0.3 };
      const qaValidation = {
        isFactual: true,
        isNeutralTone: true,
        qualityScore: 0.8,
        status: "rejected" as const,
      };

      const result = service.calculateRank(
        publishedAt,
        0.9,
        0.8,
        0.9,
        engagement,
        [10, 15],
        qaValidation
      );

      expect(result.qaPenalty).toBeLessThan(1.0);
      expect(result.finalRank).toBeLessThan(0.8); // Should be penalized
    });
  });

  describe("Dynamic Threshold", () => {
    it("should calculate dynamic threshold based on rank mean", () => {
      const threshold1 = service.calculateDynamicThreshold(0.8); // High average rank
      const threshold2 = service.calculateDynamicThreshold(0.3); // Low average rank

      expect(threshold1).toBeLessThan(threshold2); // Lower threshold for better articles
      expect(threshold1).toBeGreaterThanOrEqual(0.5);
      expect(threshold1).toBeLessThanOrEqual(0.9);
    });

    it("should keep threshold within bounds", () => {
      const threshold = service.calculateDynamicThreshold(0.0);
      expect(threshold).toBeGreaterThanOrEqual(0.5);

      const thresholdHigh = service.calculateDynamicThreshold(1.0);
      expect(thresholdHigh).toBeLessThanOrEqual(0.9);
    });

    it("should lower threshold when average rank is high", () => {
      const highRankThreshold = service.calculateDynamicThreshold(0.9);
      const mediumRankThreshold = service.calculateDynamicThreshold(0.5);

      expect(highRankThreshold).toBeLessThan(mediumRankThreshold);
    });
  });

  describe("Freshness score variations", () => {
    it("should return 1.0 for articles less than 1 hour old", () => {
      const now = new Date();
      const recent = new Date(now.getTime() - 3000000); // 50 minutes ago
      expect(service.calculateFreshness(recent)).toBe(1.0);
    });

    it("should handle articles between 1-24 hours", () => {
      const hoursAgo = new Date(Date.now() - 7200000); // 2 hours ago
      const freshness = service.calculateFreshness(hoursAgo);
      expect(freshness).toBeGreaterThan(0.5);
      expect(freshness).toBeLessThan(1.0);
    });

    it("should handle articles between 24 hours and 7 days", () => {
      const daysAgo = new Date(Date.now() - 259200000); // 3 days ago
      const freshness = service.calculateFreshness(daysAgo);
      expect(freshness).toBeGreaterThan(0.1);
      expect(freshness).toBeLessThan(0.5);
    });

    it("should handle old articles (>30 days)", () => {
      const old = new Date(Date.now() - 2592000000); // 30 days ago
      const freshness = service.calculateFreshness(old);
      expect(freshness).toBeGreaterThan(0.1);
      expect(freshness).toBeLessThan(1.0);
    });
  });

  describe("Trend calculations", () => {
    it("should handle empty trend history", () => {
      const engagement = { views: 100, clicks: 10, timeOnPage: 60, bounceRate: 0.3 };
      const trend = service.calculateTrend(engagement, []);
      expect(trend).toBe(0.5);
    });

    it("should calculate trend with multiple history points", () => {
      const engagement = { views: 500, clicks: 100, timeOnPage: 120, bounceRate: 0.2 };
      const trendHistory = [50, 60, 70, 80, 90];

      const result = service.calculateRank(
        new Date(Date.now() - 3600000),
        0.9,
        0.8,
        0.9,
        engagement,
        trendHistory
      );

      expect(result.finalRank).toBeGreaterThan(0);
      expect(result.trend).toBeDefined();
      expect(result.trend).toBeGreaterThanOrEqual(0);
      expect(result.trend).toBeLessThanOrEqual(1);
    });

    it("should handle declining trend", () => {
      const engagement = { views: 100, clicks: 10, timeOnPage: 60, bounceRate: 0.3 };
      const trendHistory = [100, 80, 60]; // declining

      const trend = service.calculateTrend(engagement, trendHistory);
      expect(trend).toBeGreaterThanOrEqual(0);
      expect(trend).toBeLessThanOrEqual(1);
    });
  });

  describe("Social signal calculations", () => {
    it("should calculate social signal with high CTR", () => {
      const engagement = { views: 100, clicks: 20, timeOnPage: 120, bounceRate: 0.1 };

      const result = service.calculateRank(new Date(), 0.8, 0.7, 0.9, engagement, [10, 15]);

      expect(result.socialSignal).toBeGreaterThan(0);
      expect(result.socialSignal).toBeLessThanOrEqual(1);
    });

    it("should handle high bounce rate", () => {
      const engagement = { views: 100, clicks: 5, timeOnPage: 10, bounceRate: 0.9 };

      const result = service.calculateRank(new Date(), 0.5, 0.5, 0.5, engagement, [10, 10]);

      expect(result.socialSignal).toBeLessThan(0.5);
    });
  });

  describe("Update schedule", () => {
    it("should return correct cron expression", () => {
      const schedule = service.getUpdateSchedule();
      expect(schedule).toBe("*/15 * * * *");
    });
  });
});
