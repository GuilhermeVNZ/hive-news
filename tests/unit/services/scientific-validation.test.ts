import { describe, it, expect, beforeAll } from "vitest";
import { ScientificValidationService } from "../../../apps/backend-cmmv/src/services/scientific-validation.service";

describe("ScientificValidationService", () => {
  let service: ScientificValidationService;

  beforeAll(() => {
    service = new ScientificValidationService();
  });

  describe("Basic validation flow", () => {
    it("should skip validation for non-academic sources", async () => {
      const article = {
        id: "test-1",
        title: "Tech News Article",
        source: "https://techcrunch.com/article",
        domain: "techcrunch.com",
        authors: ["John Doe"],
        full_text: "Some tech news content...",
        published_at: new Date(),
      };

      const config = {
        enable_scientific_validation: true,
        source_types: {
          "techcrunch.com": "news",
        },
      };

      const result = await service.validate(article, config);

      expect(result.validation_score).toBe(1.0);
      expect(result.flagged).toBe(false);
      expect(result.source_verified).toBe(true);
    });

    it("should validate academic sources", async () => {
      const article = {
        id: "test-2",
        title: "AI Research Paper",
        source: "https://arxiv.org/abs/1234.5678",
        domain: "arxiv.org",
        authors: ["Jane Smith", "Bob Wilson"],
        full_text: "Abstract: This paper presents... DOI: 10.1234/example",
        published_at: new Date(),
      };

      const config = {
        enable_scientific_validation: true,
        source_types: {
          "arxiv.org": "academic",
        },
      };

      const result = await service.validate(article, config);

      expect(result.validation_score).toBeGreaterThan(0);
      expect(result.validation_score).toBeLessThanOrEqual(1);
      expect(result.document_id).toBe("test-2");
    });

    it("should skip validation when disabled", async () => {
      const article = {
        id: "test-3",
        title: "Research Paper",
        source: "https://arxiv.org/abs/1234.5678",
        domain: "arxiv.org",
        authors: ["Author"],
        full_text: "Content...",
        published_at: new Date(),
      };

      const config = {
        enable_scientific_validation: false,
        source_types: {
          "arxiv.org": "academic",
        },
      };

      const result = await service.validate(article, config);

      expect(result.validation_score).toBe(1.0);
      expect(result.flagged).toBe(false);
    });
  });

  describe("Score combination", () => {
    it("should combine scores with correct weights", () => {
      const result = service.combineScores(0.8, 0.9, true, 0.2);

      // 0.8*0.4 + 0.9*0.3 + 1.0*0.2 + 0.8*0.1
      // = 0.32 + 0.27 + 0.2 + 0.08 = 0.87
      expect(result).toBeCloseTo(0.87, 2);
    });

    it("should handle false author verification", () => {
      const result = service.combineScores(1.0, 1.0, false, 0.0);

      // 1.0*0.4 + 1.0*0.3 + 0.0*0.2 + 1.0*0.1 = 0.8
      expect(result).toBeCloseTo(0.8, 2);
    });

    it("should flag papers with low validation score", async () => {
      const article = {
        id: "test-low",
        title: "Low Quality Paper",
        source: "unknown-journal.com",
        domain: "unknown-journal.com",
        authors: [],
        full_text: "Some content",
        published_at: new Date(),
      };

      const config = {
        enable_scientific_validation: true,
        source_types: {
          "unknown-journal.com": "academic",
        },
      };

      const result = await service.validate(article, config);

      // Low quality should be flagged
      expect(result.flagged).toBeTruthy();
      expect(result.validation_score).toBeLessThan(0.6);
    });
  });

  describe("Source reputation", () => {
    it("should identify high-reputation sources", async () => {
      const highRep = await service.checkSourceReputation("nature.com/journal/article");
      expect(highRep).toBeGreaterThan(0.8);
    });

    it("should identify medium-reputation sources", async () => {
      const mediumRep = await service.checkSourceReputation("ieee.org/conference/papers");
      expect(mediumRep).toBeGreaterThan(0.6);
      expect(mediumRep).toBeLessThan(0.9);
    });

    it("should handle unknown sources", async () => {
      const unknownRep = await service.checkSourceReputation("random-journal.com");
      expect(unknownRep).toBe(0.5);
    });
  });

  describe("AI detection", () => {
    it("should detect suspicious AI patterns", async () => {
      const suspiciousText = `
        Introduction to the topic. In conclusion, it is worth noting that 
        furthermore, the results show. In summary, we can see.
      `;

      const aiProb = await service.estimateAIGenerated(suspiciousText);
      expect(aiProb).toBeGreaterThan(0);
      expect(aiProb).toBeLessThanOrEqual(1);
    });

    it("should have low AI probability for natural text", async () => {
      const naturalText = `
        We conducted experiments using standard protocols. 
        The results indicate significant improvements in performance metrics.
      `;

      const aiProb = await service.estimateAIGenerated(naturalText);
      expect(aiProb).toBeLessThan(0.5);
    });
  });

  describe("Citation verification", () => {
    it("should extract citations from text", async () => {
      const textWithCitations = `
        Previous work (DOI: 10.1234/example1, https://arxiv.org/abs/1234.5678)
        has shown significant results.
      `;

      const resolveRate = await service.checkCitations(textWithCitations);
      expect(resolveRate).toBeGreaterThanOrEqual(0);
      expect(resolveRate).toBeLessThanOrEqual(1);
    });

    it("should handle text without citations", async () => {
      const textWithoutCitations = "This is some content without citations.";

      const resolveRate = await service.checkCitations(textWithoutCitations);
      expect(resolveRate).toBe(0.5);
    });
  });

  describe("Author verification", () => {
    it("should verify valid authors", async () => {
      const authors = ["John Doe", "Jane Smith"];

      const verified = await service.verifyAuthors(authors);
      expect(verified).toBe(true);
    });

    it("should reject invalid authors", async () => {
      const authors: string[] = [];

      const verified = await service.verifyAuthors(authors);
      expect(verified).toBe(false);
    });

    it("should reject authors with invalid names", async () => {
      const authors = ["Ab", "C"];

      const verified = await service.verifyAuthors(authors);
      expect(verified).toBe(false);
    });
  });
});

