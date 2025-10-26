import * as xml2js from "xml2js";

/**
 * RSS Parser Service
 * Parses RSS and Atom feeds to extract articles
 */

export interface FeedItem {
  title: string;
  link: string;
  description: string;
  pubDate: Date;
  guid: string;
  author?: string;
  categories?: string[];
}

export interface FeedMetadata {
  title: string;
  description: string;
  link: string;
  lastBuildDate?: Date;
}

export class RSSParserService {
  private parser: xml2js.Parser;

  constructor() {
    this.parser = new xml2js.Parser({
      explicitArray: false,
      mergeAttrs: true,
    });
  }

  /**
   * Parse RSS feed from XML string
   */
  async parseRSS(xmlString: string): Promise<{ metadata: FeedMetadata; items: FeedItem[] }> {
    try {
      const result = await this.parser.parseStringPromise(xmlString);
      const channel = result.rss?.channel || result.feed;

      const metadata: FeedMetadata = {
        title: channel.title || "",
        description: channel.description || "",
        link: channel.link || "",
        lastBuildDate: channel.lastBuildDate ? new Date(channel.lastBuildDate) : undefined,
      };

      const items: FeedItem[] = [];

      const itemsArray = Array.isArray(channel.item)
        ? channel.item
        : [channel.item].filter(Boolean);

      for (const item of itemsArray) {
        items.push({
          title: item.title || "",
          link: item.link || "",
          description: item.description || "",
          pubDate: item.pubDate ? new Date(item.pubDate) : new Date(),
          guid: item.guid?._ || item.guid || item.link || "",
          author: item.author || item["dc:creator"] || undefined,
          categories: item.category
            ? Array.isArray(item.category)
              ? item.category
              : [item.category]
            : undefined,
        });
      }

      return { metadata, items };
    } catch (error) {
      console.error("Failed to parse RSS feed:", error);
      throw new Error("Invalid RSS feed format");
    }
  }

  /**
   * Parse Atom feed from XML string
   */
  async parseAtom(xmlString: string): Promise<{ metadata: FeedMetadata; items: FeedItem[] }> {
    try {
      const result = await this.parser.parseStringPromise(xmlString);
      const feed = result.feed;

      const metadata: FeedMetadata = {
        title: feed.title?._ || feed.title || "",
        description: feed.subtitle?._ || feed.subtitle || "",
        link: feed.link?.[0]?.$.href || feed.link?.[0] || "",
        lastBuildDate: feed.updated ? new Date(feed.updated) : undefined,
      };

      const items: FeedItem[] = [];

      const entries = Array.isArray(feed.entry) ? feed.entry : [feed.entry].filter(Boolean);

      for (const entry of entries) {
        items.push({
          title: entry.title?._ || entry.title || "",
          link: entry.link?.[0]?.$.href || entry.link || "",
          description: entry.summary?._ || entry.content?._ || "",
          pubDate: entry.updated || entry.published || new Date().toISOString(),
          guid: entry.id || entry.link,
          author: entry.author?.name || undefined,
          categories: entry.category?.[0]?.$.term ? [entry.category[0].$.term] : undefined,
        });
      }

      return { metadata, items };
    } catch (error) {
      console.error("Failed to parse Atom feed:", error);
      throw new Error("Invalid Atom feed format");
    }
  }

  /**
   * Auto-detect and parse feed (RSS or Atom)
   */
  async parseFeed(xmlString: string): Promise<{ metadata: FeedMetadata; items: FeedItem[] }> {
    const lowerXml = xmlString.toLowerCase();

    if (lowerXml.includes("<rss")) {
      return this.parseRSS(xmlString);
    } else if (lowerXml.includes("<feed")) {
      return this.parseAtom(xmlString);
    } else {
      throw new Error("Unknown feed format");
    }
  }
}

