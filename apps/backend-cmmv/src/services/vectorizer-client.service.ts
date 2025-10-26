/**
 * Vectorizer Client Service
 * Integrates with Vectorizer MCP for document indexing and search
 */

export interface VectorizerConfig {
  baseURL: string;
  mcpEndpoint: string;
  apiKey?: string;
}

export interface TransmutationRequest {
  filePath: string;
  collection: string;
  metadata?: Record<string, any>;
}

export interface TransmutationResponse {
  text: string;
  vectorId: string;
  metadata: Record<string, any>;
}

export class VectorizerClientService {
  private config: VectorizerConfig;

  constructor(config: VectorizerConfig) {
    this.config = config;
  }

  /**
   * Send document to Vectorizer for transmutation
   */
  async transmuteDocument(request: TransmutationRequest): Promise<TransmutationResponse> {
    const url = `${this.config.baseURL}/api/transmute`;

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
        },
        body: JSON.stringify({
          file_path: request.filePath,
          collection: request.collection,
          metadata: request.metadata || {},
        }),
      });

      if (!response.ok) {
        throw new Error(`Vectorizer transmutation failed: ${response.status}`);
      }

      const result = (await response.json()) as {
        text?: string;
        vector_id?: string;
        id?: string;
        metadata?: Record<string, unknown>;
      };

      return {
        text: result.text || "",
        vectorId: result.vector_id || result.id || "",
        metadata: result.metadata || {},
      };
    } catch (error) {
      console.error("Vectorizer transmutation error:", error);
      throw new Error("Failed to transmute document");
    }
  }

  /**
   * Search documents in Vectorizer collection
   */
  async searchCollection(
    query: string,
    collection: string,
    limit = 10
  ): Promise<Array<{ id: string; score: number; metadata: any }>> {
    const url = `${this.config.baseURL}/api/search`;

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
        },
        body: JSON.stringify({
          query,
          collection,
          limit,
        }),
      });

      if (!response.ok) {
        throw new Error(`Vectorizer search failed: ${response.status}`);
      }

      const results = (await response.json()) as {
        hits?: Array<{ id: string; score: number; metadata: any }>;
        results?: Array<{ id: string; score: number; metadata: any }>;
      };

      return results.hits || results.results || [];
    } catch (error) {
      console.error("Vectorizer search error:", error);
      throw new Error("Failed to search collection");
    }
  }

  /**
   * Send text to Vectorizer for indexing
   */
  async indexText(
    text: string,
    collection: string,
    metadata?: Record<string, any>
  ): Promise<string> {
    const url = `${this.config.mcpEndpoint}/insert_text`;

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify({
          collection_name: collection,
          text,
          metadata: metadata || {},
        }),
      });

      if (!response.ok) {
        throw new Error(`Vectorizer indexing failed: ${response.status}`);
      }

      const result = (await response.json()) as { vector_id?: string; id?: string };

      return result.vector_id || result.id || "";
    } catch (error) {
      console.error("Vectorizer indexing error:", error);
      throw new Error("Failed to index text");
    }
  }

  /**
   * Get document chunks from Vectorizer
   */
  async getDocumentChunks(
    _collection: string,
    filePath: string,
    _startChunk = 0,
    _limit = 10
  ): Promise<any> {
    const url = `${this.config.baseURL}/api/files/${encodeURIComponent(filePath)}/chunks`;

    try {
      const response = await fetch(url, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to get document chunks: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Error getting document chunks:", error);
      throw new Error("Failed to get document chunks");
    }
  }
}
