import { describe, it, expect, vi, beforeAll, beforeEach, afterEach } from "vitest";
import { DeepSeekClientService } from "../../../apps/backend-cmmv/src/services/deepseek-client.service";

// Mock fetch
global.fetch = vi.fn();

describe("DeepSeekClientService", () => {
  let service: DeepSeekClientService;

  beforeAll(() => {
    service = new DeepSeekClientService({
      apiKey: "test-key",
      baseURL: "https://api.deepseek.com",
    });
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("buildArticlePrompt", () => {
    it("should build article prompt correctly", () => {
      const prompt = service["buildArticlePrompt"]({
        documentText: "Sample document text",
        title: "Test Article",
        style: "scientific",
        baseLanguage: "en",
        references: ["ref1", "ref2"],
      });

      expect(prompt).toContain("Test Article");
      expect(prompt).toContain("scientific");
      expect(prompt).toContain("ref1, ref2");
    });

    it("should handle requests without references", () => {
      const prompt = service["buildArticlePrompt"]({
        documentText: "Document",
        title: "Title",
        style: "tech",
        baseLanguage: "en",
      });

      expect(prompt).toContain("N/A");
    });

    it("should truncate long document text", () => {
      const longText = "x".repeat(2000);
      const prompt = service["buildArticlePrompt"]({
        documentText: longText,
        title: "Test",
        style: "scientific",
        baseLanguage: "en",
      });

      // Should be truncated to 1000 chars
      expect(prompt.length).toBeLessThan(1500);
    });
  });

  describe("extractDek", () => {
    it("should extract dek from article text", () => {
      const text = "First sentence. Second sentence. More content here.";
      const dek = service["extractDek"](text);
      expect(dek).toBe("First sentence");
    });

    it("should truncate long first sentence", () => {
      const longSentence = "x".repeat(250) + ".";
      const dek = service["extractDek"](longSentence);
      expect(dek.length).toBeLessThanOrEqual(200);
    });

    it("should handle text without periods", () => {
      const text = "Single sentence text";
      const dek = service["extractDek"](text);
      expect(dek).toBe("Single sentence text");
    });
  });

  describe("generateArticle", () => {
    it("should generate article successfully", async () => {
      const mockResponse = {
        choices: [
          {
            message: {
              content:
                "This is the generated article content. It contains multiple sentences and paragraphs.",
            },
          },
        ],
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const result = await service.generateArticle({
        documentText: "Document content",
        title: "Article Title",
        style: "scientific",
        baseLanguage: "en",
        references: ["ref1"],
      });

      expect(result.title).toBe("Article Title");
      expect(result.body).toBeTruthy();
      expect(result.dek).toBeTruthy();
    });

    it("should handle API errors", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 429,
      });

      await expect(
        service.generateArticle({
          documentText: "Content",
          title: "Title",
          style: "scientific",
          baseLanguage: "en",
        })
      ).rejects.toThrow("Failed to generate article");
    });

    it("should apply rate limiting", async () => {
      const mockResponse = {
        choices: [{ message: { content: "Article" } }],
      };

      (global.fetch as any).mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const start = Date.now();
      await service.generateArticle({
        documentText: "Content",
        title: "Title",
        style: "scientific",
        baseLanguage: "en",
      });

      await service.generateArticle({
        documentText: "Content2",
        title: "Title2",
        style: "scientific",
        baseLanguage: "en",
      });

      const duration = Date.now() - start;
      // Should have rate limiting delay
      expect(duration).toBeGreaterThan(1000);
    });
  });

  describe("translate", () => {
    it("should translate text successfully", async () => {
      const mockResponse = {
        choices: [
          {
            message: {
              content: "Texto traducido",
            },
          },
        ],
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const result = await service.translate({
        text: "Translated text",
        sourceLang: "en",
        targetLang: "es",
      });

      expect(result).toBe("Texto traducido");
    });

    it("should handle translation errors", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
      });

      await expect(
        service.translate({
          text: "Text",
          sourceLang: "en",
          targetLang: "es",
        })
      ).rejects.toThrow("Failed to translate text");
    });
  });
});
