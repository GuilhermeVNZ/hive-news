/**
 * PDF Collector Service
 * Detects, downloads, and processes PDFs from academic paper sources
 */

export interface PDFInfo {
  url: string;
  title: string;
  authors: string[];
  publishedAt: Date;
  source: string;
  pdfUrl: string;
}

export interface PDFContent {
  pdfInfo: PDFInfo;
  fullText: string;
  abstract: string;
  metadata: Record<string, any>;
}

export class PDFCollectorService {
  /**
   * Detect PDF links from ArXiv API response
   */
  extractPDFLinks(arxivResponse: any): string[] {
    const pdfLinks: string[] = [];
    
    if (arxivResponse.feed?.entry) {
      const entries = Array.isArray(arxivResponse.feed.entry) 
        ? arxivResponse.feed.entry 
        : [arxivResponse.feed.entry];
      
      for (const entry of entries) {
        // ArXiv provides PDF links in the link array
        if (entry.link) {
          const links = Array.isArray(entry.link) ? entry.link : [entry.link];
          for (const link of links) {
            if (link.type === 'application/pdf' && link.href) {
              pdfLinks.push(link.href);
            }
          }
        }
      }
    }
    
    return pdfLinks;
  }

  /**
   * Detect PDF links from BioRxiv/medRxiv API response
   */
  extractBioRxivPDFs(biorxivResponse: any): string[] {
    const pdfLinks: string[] = [];
    
    if (biorxivResponse.collection?.length > 0) {
      for (const paper of biorxivResponse.collection) {
        if (paper.doi && paper.pdf) {
          pdfLinks.push(paper.pdf);
        }
      }
    }
    
    return pdfLinks;
  }

  /**
   * Download PDF from URL
   */
  async downloadPDF(url: string): Promise<Buffer> {
    try {
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`Failed to download PDF: ${response.status} ${response.statusText}`);
      }
      
      const arrayBuffer = await response.arrayBuffer();
      return Buffer.from(arrayBuffer);
    } catch (error) {
      console.error(`Error downloading PDF from ${url}:`, error);
      throw error;
    }
  }

  /**
   * Extract text from PDF buffer
   * Note: This is a basic implementation. For production, use a proper PDF parser
   */
  async extractTextFromPDF(pdfBuffer: Buffer): Promise<string> {
    // For now, return a placeholder
    // In production, use a library like pdf-parse or similar
    // This would require: npm install pdf-parse
    return "PDF text extraction not implemented. Install pdf-parse for full support.";
  }

  /**
   * Process ArXiv papers
   */
  async processArXivPapers(arxivResponse: any): Promise<PDFContent[]> {
    const pdfContents: PDFContent[] = [];
    const entries = Array.isArray(arxivResponse.feed?.entry) 
      ? arxivResponse.feed.entry 
      : arxivResponse.feed?.entry ? [arxivResponse.feed.entry] : [];
    
    for (const entry of entries) {
      if (!entry.id) continue;
      
      // Extract metadata
      const pdfInfo: PDFInfo = {
        url: entry.id,
        title: entry.title || 'Untitled',
        authors: this.extractAuthors(entry.author),
        publishedAt: new Date(entry.published || entry.updated),
        source: 'arxiv',
        pdfUrl: this.getPDFURL(entry),
      };
      
      // Get PDF link
      const pdfUrl = this.getPDFURL(entry);
      if (pdfUrl) {
        try {
          // Download PDF
          const pdfBuffer = await this.downloadPDF(pdfUrl);
          
          // Extract text
          const fullText = await this.extractTextFromPDF(pdfBuffer);
          
          pdfContents.push({
            pdfInfo,
            fullText,
            abstract: entry.summary || '',
            metadata: {
              arxivId: entry.id.split('/').pop(),
              doi: entry['arxiv:doi']?.['#'],
              categories: entry.category,
            },
          });
        } catch (error) {
          console.error(`Error processing PDF from ${pdfUrl}:`, error);
        }
      }
    }
    
    return pdfContents;
  }

  /**
   * Get PDF URL from ArXiv entry
   */
  private getPDFURL(entry: any): string | null {
    if (!entry.link) return null;
    
    const links = Array.isArray(entry.link) ? entry.link : [entry.link];
    const pdfLink = links.find((link: any) => link.type === 'application/pdf');
    
    return pdfLink?.href || null;
  }

  /**
   * Extract authors from ArXiv entry
   */
  private extractAuthors(author: any): string[] {
    if (!author) return [];
    
    const authors = Array.isArray(author) ? author : [author];
    return authors.map((a: any) => a.name || 'Unknown').filter(Boolean);
  }

  /**
   * Process BioRxiv papers
   */
  async processBioRxivPapers(biorxivResponse: any): Promise<PDFContent[]> {
    const pdfContents: PDFContent[] = [];
    
    if (!biorxivResponse.collection || !Array.isArray(biorxivResponse.collection)) {
      return pdfContents;
    }
    
    for (const paper of biorxivResponse.collection) {
      const pdfInfo: PDFInfo = {
        url: `https://doi.org/${paper.doi}`,
        title: paper.title || 'Untitled',
        authors: paper.authors?.split(',').map((a: string) => a.trim()) || [],
        publishedAt: new Date(paper.date),
        source: 'biorxiv',
        pdfUrl: paper.pdf || '',
      };
      
      if (paper.pdf) {
        try {
          const pdfBuffer = await this.downloadPDF(paper.pdf);
          const fullText = await this.extractTextFromPDF(pdfBuffer);
          
          pdfContents.push({
            pdfInfo,
            fullText,
            abstract: paper.abstract || '',
            metadata: {
              doi: paper.doi,
              server: paper.server,
            },
          });
        } catch (error) {
          console.error(`Error processing PDF from ${paper.pdf}:`, error);
        }
      }
    }
    
    return pdfContents;
  }

  /**
   * Process multiple PDF sources
   */
  async processAllPDFs(sources: Array<{ type: string; data: any }>): Promise<PDFContent[]> {
    const allPDFs: PDFContent[] = [];
    
    for (const source of sources) {
      try {
        if (source.type === 'arxiv') {
          const pdfs = await this.processArXivPapers(source.data);
          allPDFs.push(...pdfs);
        } else if (source.type === 'biorxiv' || source.type === 'medrxiv') {
          const pdfs = await this.processBioRxivPapers(source.data);
          allPDFs.push(...pdfs);
        }
      } catch (error) {
        console.error(`Error processing ${source.type} PDFs:`, error);
      }
    }
    
    return allPDFs;
  }
}

