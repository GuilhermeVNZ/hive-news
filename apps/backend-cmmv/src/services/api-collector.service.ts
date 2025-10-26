/**
 * API Collector Service
 * Collects content from REST APIs with authentication and pagination
 */

export interface APIConfig {
  url: string;
  method?: "GET" | "POST" | "PUT" | "DELETE";
  headers?: Record<string, string>;
  auth?: {
    type: "bearer" | "basic" | "apikey";
    token?: string;
    apiKey?: string;
    apiKeyHeader?: string;
    username?: string;
    password?: string;
  };
  pagination?: {
    type: "cursor" | "offset" | "page";
    param: string;
    initialValue?: string | number;
  };
}

export interface APIResponse {
  data: any[];
  nextPage?: string | number | null;
  hasMore: boolean;
}

export class APICollectorService {
  private rateLimitMap: Map<string, number> = new Map();
  private readonly MIN_INTERVAL = 1000; // 1 second minimum between requests

  /**
   * Fetch data from API with authentication
   */
  async fetchAPI(config: APIConfig, params?: Record<string, any>): Promise<any> {
    const url = new URL(config.url);

    // Add query parameters
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        url.searchParams.append(key, String(value));
      });
    }

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...config.headers,
    };

    // Handle authentication
    if (config.auth) {
      switch (config.auth.type) {
        case "bearer":
          if (config.auth.token) {
            headers["Authorization"] = `Bearer ${config.auth.token}`;
          }
          break;
        case "basic":
          if (config.auth.username && config.auth.password) {
            const credentials = Buffer.from(
              `${config.auth.username}:${config.auth.password}`
            ).toString("base64");
            headers["Authorization"] = `Basic ${credentials}`;
          }
          break;
        case "apikey":
          if (config.auth.apiKey && config.auth.apiKeyHeader) {
            headers[config.auth.apiKeyHeader] = config.auth.apiKey;
          }
          break;
      }
    }

    const options: RequestInit = {
      method: config.method || "GET",
      headers,
    };

    try {
      const response = await fetch(url.toString(), options);

      if (!response.ok) {
        throw new Error(`API request failed: ${response.status} ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error("API request error:", error);
      throw error;
    }
  }

  /**
   * Fetch paginated data from API
   */
  async fetchPaginatedData(config: APIConfig, maxPages = 10): Promise<APIResponse> {
    const allData: any[] = [];
    let nextPage: string | number | null = null;
    let page = 1;

    // Apply rate limiting
    await this.waitForRateLimit(config.url);

    while (page <= maxPages) {
      const params: Record<string, any> = {};

      // Add pagination parameter
      if (config.pagination) {
        switch (config.pagination.type) {
          case "page":
            params[config.pagination.param] = page;
            break;
          case "cursor":
            params[config.pagination.param] = nextPage || config.pagination.initialValue || "";
            break;
          case "offset":
            const offset = page === 1 ? 0 : allData.length;
            params[config.pagination.param] = offset;
            break;
        }
      }

      const response = await this.fetchAPI(config, params);

      // Extract data from response (adjust based on API structure)
      const items = response.data || response.results || response.items || response;
      allData.push(...(Array.isArray(items) ? items : [items]));

      // Check if there's more data
      nextPage = response.next_page || response.nextCursor || response.next || null;

      if (!nextPage && page >= maxPages) {
        break;
      }

      page++;

      // Apply rate limiting
      if (page <= maxPages) {
        await this.waitForRateLimit(config.url);
      }
    }

    return {
      data: allData,
      nextPage,
      hasMore: !!nextPage,
    };
  }

  /**
   * Wait for rate limit cooldown
   */
  private async waitForRateLimit(url: string): Promise<void> {
    const lastRequest = this.rateLimitMap.get(url);
    const now = Date.now();

    if (lastRequest && now - lastRequest < this.MIN_INTERVAL) {
      const waitTime = this.MIN_INTERVAL - (now - lastRequest);
      await new Promise((resolve) => setTimeout(resolve, waitTime));
    }

    this.rateLimitMap.set(url, Date.now());
  }
}
