import * as crypto from "crypto";
// import { SourceContract } from '../../contracts/source.contract';

interface SourceContract {
  id: string;
  portal_id: string;
  url: string;
  kind: "rss" | "api" | "html";
  last_fetch: Date;
}

/**
 * Source Manager Service
 * Manages source collection, deduplication, and scheduling
 */

export class SourceManagerService {
  private urlCache: Map<string, string> = new Map(); // URL hash -> portal_id

  /**
   * Generate hash for URL to detect duplicates
   */
  generateURLHash(url: string): string {
    return crypto.createHash("sha256").update(url).digest("hex");
  }

  /**
   * Check if source already exists
   */
  async checkDuplicate(portalId: string, url: string): Promise<boolean> {
    const hash = this.generateURLHash(url);
    const cacheKey = `${portalId}:${hash}`;

    return this.urlCache.has(cacheKey);
  }

  /**
   * Register source in cache
   */
  async registerSource(portalId: string, url: string): Promise<void> {
    const hash = this.generateURLHash(url);
    const cacheKey = `${portalId}:${hash}`;

    this.urlCache.set(cacheKey, portalId);
  }

  /**
   * CRUD operations for sources
   */
  async createSource(source: Partial<SourceContract>): Promise<SourceContract> {
    // Check for duplicates
    if (source.url && source.portal_id) {
      const isDuplicate = await this.checkDuplicate(source.portal_id, source.url);

      if (isDuplicate) {
        throw new Error(`Source already exists: ${source.url}`);
      }
    }

    const newSource: SourceContract = {
      id: source.id || crypto.randomUUID(),
      portal_id: source.portal_id || "",
      url: source.url || "",
      kind: source.kind || "rss",
      last_fetch: source.last_fetch || new Date(),
    };

    // Register in cache
    if (newSource.portal_id && newSource.url) {
      await this.registerSource(newSource.portal_id, newSource.url);
    }

    return newSource;
  }

  /**
   * Update last_fetch timestamp
   */
  updateLastFetch(source: SourceContract): SourceContract {
    return {
      ...source,
      last_fetch: new Date(),
    };
  }

  /**
   * Validate source configuration
   */
  validateSource(source: Partial<SourceContract>): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!source.portal_id) {
      errors.push("portal_id is required");
    }

    if (!source.url) {
      errors.push("url is required");
    } else {
      try {
        new URL(source.url);
      } catch {
        errors.push("url must be a valid URL");
      }
    }

    if (!source.kind || !["rss", "api", "html"].includes(source.kind)) {
      errors.push("kind must be rss, api, or html");
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Schedule fetch jobs for sources
   */
  async scheduleFetches(
    sources: SourceContract[]
  ): Promise<Array<{ source: SourceContract; scheduled: Date }>> {
    const schedule: Array<{ source: SourceContract; scheduled: Date }> = [];

    for (const source of sources) {
      // const now = new Date();
      const lastFetch = source.last_fetch || new Date(0);
      // const timeSinceLastFetch = now.getTime() - lastFetch.getTime();

      // Schedule based on refresh frequency
      let refreshInterval = 3600000; // Default: 1 hour (in milliseconds)

      if (source.kind === "rss") {
        refreshInterval = 1800000; // 30 minutes for RSS
      } else if (source.kind === "api") {
        refreshInterval = 3600000; // 1 hour for API
      } else if (source.kind === "html") {
        refreshInterval = 7200000; // 2 hours for HTML
      }

      const scheduledTime = new Date(lastFetch.getTime() + refreshInterval);

      schedule.push({ source, scheduled: scheduledTime });
    }

    return schedule;
  }

  /**
   * Cleanup old cache entries
   */
  cleanupCache(_maxAge = 86400000): void {
    // In production, implement proper TTL-based cache cleanup
    // const now = Date.now();

    // For now, just clear if cache gets too large
    if (this.urlCache.size > 100000) {
      this.urlCache.clear();
    }
  }
}
