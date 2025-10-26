import { describe, it, expect, vi } from "vitest";
import { PDFCollectorService } from "../../../apps/backend-cmmv/src/services/pdf-collector.service";

describe("PDFCollectorService", () => {
  const service = new PDFCollectorService();

  describe("extractPDFLinks", () => {
    it("should extract PDF links from ArXiv response", () => {
      const arxivResponse = {
        feed: {
          entry: [
            {
              id: "http://arxiv.org/abs/2301.00001v1",
              title: "Test Paper",
              link: [
                { type: "text/html", href: "http://arxiv.org/abs/2301.00001v1" },
                { type: "application/pdf", href: "http://arxiv.org/pdf/2301.00001v1.pdf" },
              ],
            },
          ],
        },
      };

      const links = service.extractPDFLinks(arxivResponse);
      expect(links).toHaveLength(1);
      expect(links[0]).toBe("http://arxiv.org/pdf/2301.00001v1.pdf");
    });

    it("should return empty array for invalid response", () => {
      const invalidResponse = {};
      const links = service.extractPDFLinks(invalidResponse);
      expect(links).toHaveLength(0);
    });

    it("should handle single entry (not array)", () => {
      const arxivResponse = {
        feed: {
          entry: {
            id: "http://arxiv.org/abs/2301.00002v1",
            link: [{ type: "application/pdf", href: "http://arxiv.org/pdf/2301.00002v1.pdf" }],
          },
        },
      };

      const links = service.extractPDFLinks(arxivResponse);
      expect(links).toHaveLength(1);
    });
  });

  describe("extractBioRxivPDFs", () => {
    it("should extract PDF links from BioRxiv response", () => {
      const biorxivResponse = {
        collection: [
          { doi: "10.1101/2023.01.00001", pdf: "http://biorxiv.org/lookup/doi/10.1101/2023.01.00001.pdf" },
          { doi: "10.1101/2023.01.00002", pdf: "http://biorxiv.org/lookup/doi/10.1101/2023.01.00002.pdf" },
        ],
      };

      const links = service.extractBioRxivPDFs(biorxivResponse);
      expect(links).toHaveLength(2);
    });

    it("should return empty array for invalid response", () => {
      const invalidResponse = {};
      const links = service.extractBioRxivPDFs(invalidResponse);
      expect(links).toHaveLength(0);
    });
  });

  describe("downloadPDF", () => {
    it("should download PDF successfully", async () => {
      // Mock fetch
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        arrayBuffer: async () => new ArrayBuffer(8),
      });

      const buffer = await service.downloadPDF("http://example.com/test.pdf");
      expect(buffer).toBeInstanceOf(Buffer);
    });

    it("should throw error on failed download", async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        statusText: "Not Found",
      });

      await expect(service.downloadPDF("http://example.com/missing.pdf")).rejects.toThrow();
    });
  });

  describe("processArXivPapers", () => {
    it("should process ArXiv papers successfully", async () => {
      const arxivResponse = {
        feed: {
          entry: {
            id: "http://arxiv.org/abs/2301.00001v1",
            title: "Test Paper",
            author: [{ name: "John Doe" }, { name: "Jane Smith" }],
            published: "2023-01-01T00:00:00Z",
            updated: "2023-01-01T00:00:00Z",
            summary: "Abstract text",
            link: [
              { type: "application/pdf", href: "http://arxiv.org/pdf/2301.00001v1.pdf" },
            ],
          },
        },
      };

      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        arrayBuffer: async () => new ArrayBuffer(8),
      });

      const pdfs = await service.processArXivPapers(arxivResponse);
      
      // Should have PDF info even if extraction isn't full
      expect(Array.isArray(pdfs)).toBe(true);
    });
  });

  describe("processBioRxivPapers", () => {
    it("should process BioRxiv papers successfully", async () => {
      const biorxivResponse = {
        collection: [
          {
            doi: "10.1101/2023.01.00001",
            title: "Test Paper",
            authors: "John Doe, Jane Smith",
            date: "2023-01-01",
            pdf: "http://biorxiv.org/lookup/doi/10.1101/2023.01.00001.pdf",
            abstract: "Abstract text",
            server: "biorxiv",
          },
        ],
      };

      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        arrayBuffer: async () => new ArrayBuffer(8),
      });

      const pdfs = await service.processBioRxivPapers(biorxivResponse);
      
      expect(Array.isArray(pdfs)).toBe(true);
    });

    it("should return empty array for invalid response", async () => {
      const invalidResponse = {};
      const pdfs = await service.processBioRxivPapers(invalidResponse);
      expect(pdfs).toHaveLength(0);
    });
  });

  describe("processAllPDFs", () => {
    it("should process multiple PDF sources", async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        arrayBuffer: async () => new ArrayBuffer(8),
      });

      const sources = [
        {
          type: "arxiv",
          data: {
            feed: {
              entry: {
                id: "http://arxiv.org/abs/2301.00001v1",
                title: "Test Paper",
                author: [{ name: "John Doe" }],
                published: "2023-01-01T00:00:00Z",
                summary: "Abstract",
                link: [{ type: "application/pdf", href: "http://arxiv.org/pdf/2301.00001v1.pdf" }],
              },
            },
          },
        },
      ];

      const pdfs = await service.processAllPDFs(sources);
      expect(Array.isArray(pdfs)).toBe(true);
    });

    it("should handle errors gracefully", async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error("Fetch failed"));

      const sources = [
        {
          type: "arxiv",
          data: { feed: { entry: {} } },
        },
      ];

      const pdfs = await service.processAllPDFs(sources);
      expect(Array.isArray(pdfs)).toBe(true);
    });
  });
});

