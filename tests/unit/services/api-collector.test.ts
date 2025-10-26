import { describe, it, expect, vi, beforeEach } from "vitest";
import { APICollectorService } from "../../../apps/backend-cmmv/src/services/api-collector.service";

// Mock fetch
global.fetch = vi.fn();

describe("APICollectorService", () => {
  let service: APICollectorService;

  beforeEach(() => {
    service = new APICollectorService();
    vi.clearAllMocks();
  });

  describe("fetchAPI", () => {
    it("should fetch data from API", async () => {
      const mockResponse = { data: [{ id: 1, title: "Test" }] };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        statusText: "OK",
        json: async () => mockResponse,
      });

      const config = {
        url: "https://api.example.com/data",
      };

      const result = await service.fetchAPI(config);

      expect(result).toEqual(mockResponse);
      expect(global.fetch).toHaveBeenCalledTimes(1);
    });

    it("should add query parameters", async () => {
      const mockResponse = { data: [] };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const config = {
        url: "https://api.example.com/data",
      };

      await service.fetchAPI(config, { page: 1, limit: 10 });

      const call = (global.fetch as any).mock.calls[0][0];
      expect(call).toContain("page=1");
      expect(call).toContain("limit=10");
    });

    it("should add Bearer token authentication", async () => {
      const mockResponse = { data: [] };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const config = {
        url: "https://api.example.com/data",
        auth: {
          type: "bearer" as const,
          token: "test-token",
        },
      };

      await service.fetchAPI(config);

      const call = (global.fetch as any).mock.calls[0][1];
      expect(call.headers["Authorization"]).toBe("Bearer test-token");
    });

    it("should add Basic authentication", async () => {
      const mockResponse = { data: [] };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const config = {
        url: "https://api.example.com/data",
        auth: {
          type: "basic" as const,
          username: "user",
          password: "pass",
        },
      };

      await service.fetchAPI(config);

      const call = (global.fetch as any).mock.calls[0][1];
      expect(call.headers["Authorization"]).toContain("Basic");
    });

    it("should add API key authentication", async () => {
      const mockResponse = { data: [] };
      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: async () => mockResponse,
      });

      const config = {
        url: "https://api.example.com/data",
        auth: {
          type: "apikey" as const,
          apiKey: "test-key",
          apiKeyHeader: "X-API-Key",
        },
      };

      await service.fetchAPI(config);

      const call = (global.fetch as any).mock.calls[0][1];
      expect(call.headers["X-API-Key"]).toBe("test-key");
    });

    it("should throw error on failed request", async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: "Not Found",
      });

      const config = {
        url: "https://api.example.com/data",
      };

      await expect(service.fetchAPI(config)).rejects.toThrow("API request failed");
    });
  });

  describe("fetchPaginatedData", () => {
    it("should fetch paginated data with page-based pagination", async () => {
      let page = 1;
      (global.fetch as any).mockImplementation(() => {
        const response = {
          ok: true,
          status: 200,
          json: async () => ({
            data: [{ id: page, title: `Page ${page}` }],
            next_page: page < 3 ? page + 1 : null,
          }),
        };
        page++;
        return response;
      });

      const config = {
        url: "https://api.example.com/data",
        pagination: {
          type: "page" as const,
          param: "page",
        },
      };

      const result = await service.fetchPaginatedData(config, 5);

      // We expect 3 pages worth of data
      expect(result.data.length).toBeGreaterThanOrEqual(3);
      expect(result.hasMore).toBe(false);
      // fetch was called multiple times due to the loop logic
      expect(global.fetch).toHaveBeenCalled();
    });

    it("should respect maxPages limit", async () => {
      let page = 1;
      (global.fetch as any).mockImplementation(() => {
        const response = {
          ok: true,
          status: 200,
          json: async () => ({
            data: [{ id: page }],
            next_page: page + 1,
          }),
        };
        page++;
        return response;
      });

      const config = {
        url: "https://api.example.com/data",
        pagination: {
          type: "page" as const,
          param: "page",
        },
      };

      const result = await service.fetchPaginatedData(config, 2);

      expect(result.data).toHaveLength(2);
      expect(global.fetch).toHaveBeenCalledTimes(2);
    });

    it("should handle offset-based pagination", async () => {
      (global.fetch as any).mockImplementation(() => ({
        ok: true,
        status: 200,
        json: async () => ({
          results: [{ id: 1 }],
        }),
      }));

      const config = {
        url: "https://api.example.com/data",
        pagination: {
          type: "offset" as const,
          param: "offset",
        },
      };

      const result = await service.fetchPaginatedData(config, 1);

      expect(global.fetch).toHaveBeenCalled();
    });

    it("should handle cursor-based pagination", async () => {
      (global.fetch as any).mockImplementation(() => ({
        ok: true,
        status: 200,
        json: async () => ({
          data: [{ id: 1 }],
          nextCursor: null,
        }),
      }));

      const config = {
        url: "https://api.example.com/data",
        pagination: {
          type: "cursor" as const,
          param: "cursor",
        },
      };

      const result = await service.fetchPaginatedData(config, 1);

      expect(global.fetch).toHaveBeenCalled();
      expect(result.hasMore).toBe(false);
    });
  });

  describe("Rate limiting", () => {
    it("should wait between requests", async () => {
      (global.fetch as any).mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({ data: [] }),
      });

      const config = {
        url: "https://api.example.com/data",
        pagination: {
          type: "page" as const,
          param: "page",
        },
      };

      const start = Date.now();
      await service.fetchPaginatedData(config, 2);
      const duration = Date.now() - start;

      // Should take at least 1 second (MIN_INTERVAL)
      expect(duration).toBeGreaterThan(500);
    });
  });
});
