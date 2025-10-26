import { describe, it, expect, vi, beforeEach } from "vitest";
import { VectorizerClientService } from "../../../apps/backend-cmmv/src/services/vectorizer-client.service";

// Mock fetch
global.fetch = vi.fn();

describe("VectorizerClientService", () => {
  let service: VectorizerClientService;

  beforeEach(() => {
    service = new VectorizerClientService({
      baseURL: "http://localhost:15002",
      mcpEndpoint: "http://localhost:15002/mcp",
      apiKey: "test-key",
    });

    vi.clearAllMocks();
  });

  describe("transmuteDocument", () => {
    it("should transmute document successfully", async () => {
      const mockResponse = {
        text: "Extracted text content",
        vector_id: "vector-123",
        metadata: { file_type: "pdf" },
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const result = await service.transmuteDocument({
        filePath: "/path/to/document.pdf",
        collection: "news-content",
        metadata: { portal: "airesearch" },
      });

      expect(result.text).toBe("Extracted text content");
      expect(result.vectorId).toBe("vector-123");
      expect(result.metadata.file_type).toBe("pdf");
    });

    it("should handle transmutation errors", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
      });

      await expect(
        service.transmuteDocument({
          filePath: "/path/to/document.pdf",
          collection: "news-content",
        })
      ).rejects.toThrow("Failed to transmute document");
    });

    it("should include API key in headers", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ text: "test", vector_id: "id" }),
      });

      await service.transmuteDocument({
        filePath: "/path/to/document.pdf",
        collection: "news-content",
      });

      const call = (global.fetch as any).mock.calls[0][1];
      expect(call.headers["Authorization"]).toBe("Bearer test-key");
    });
  });

  describe("searchCollection", () => {
    it("should search collection and return results", async () => {
      const mockResults = {
        hits: [
          { id: "result-1", score: 0.95, metadata: { title: "Article 1" } },
          { id: "result-2", score: 0.88, metadata: { title: "Article 2" } },
        ],
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResults,
      });

      const results = await service.searchCollection("AI research", "news-content", 5);

      expect(results).toHaveLength(2);
      expect(results[0].score).toBe(0.95);
      expect(results[0].metadata.title).toBe("Article 1");
    });

    it("should respect limit parameter", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ results: [] }),
      });

      await service.searchCollection("query", "news-content", 10);

      const call = (global.fetch as any).mock.calls[0][1];
      const body = JSON.parse(call.body);
      expect(body.limit).toBe(10);
    });

    it("should handle search errors", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
      });

      await expect(service.searchCollection("query", "news-content")).rejects.toThrow(
        "Failed to search collection"
      );
    });
  });

  describe("indexText", () => {
    it("should index text successfully", async () => {
      const mockResponse = {
        vector_id: "vector-456",
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const vectorId = await service.indexText("Article content", "news-content", {
        article_id: "article-123",
      });

      expect(vectorId).toBe("vector-456");
    });

    it("should include metadata when indexing", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => ({ vector_id: "id" }),
      });

      await service.indexText("content", "news-content", { portal: "test" });

      const call = (global.fetch as any).mock.calls[0][1];
      const body = JSON.parse(call.body);
      expect(body.metadata.portal).toBe("test");
    });

    it("should handle indexing errors", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
      });

      await expect(service.indexText("content", "news-content")).rejects.toThrow(
        "Failed to index text"
      );
    });
  });

  describe("getDocumentChunks", () => {
    it("should get document chunks", async () => {
      const mockChunks = {
        chunks: [
          { index: 0, text: "Chunk 1" },
          { index: 1, text: "Chunk 2" },
        ],
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockChunks,
      });

      const result = await service.getDocumentChunks(
        "news-content",
        "/path/to/document.pdf",
        0,
        10
      );

      expect(result).toEqual(mockChunks);
    });

    it("should handle missing chunks", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
      });

      await expect(
        service.getDocumentChunks("news-content", "/path/to/document.pdf")
      ).rejects.toThrow("Failed to get document chunks");
    });
  });

  describe("Integration scenarios", () => {
    it("should handle full workflow: transmute -> index -> search", async () => {
      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          json: async () => ({ text: "Content", vector_id: "vec-1" }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          json: async () => ({ vector_id: "vec-2" }),
        })
        .mockResolvedValueOnce({
          ok: true,
          status: 200,
          json: async () => ({ results: [] }),
        });

      await service.transmuteDocument({
        filePath: "/test.pdf",
        collection: "news-content",
      });

      await service.indexText("Some content", "news-content");

      await service.searchCollection("query", "news-content");

      expect(global.fetch).toHaveBeenCalledTimes(3);
    });
  });
});

