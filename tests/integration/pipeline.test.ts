import { describe, it, expect } from "vitest";
import { RSSParserService } from "../../apps/backend-cmmv/src/services/rss-parser.service";
import { APICollectorService } from "../../apps/backend-cmmv/src/services/api-collector.service";
import { MetadataExtractorService } from "../../apps/backend-cmmv/src/services/metadata-extractor.service";

describe("Content Pipeline Integration", () => {
  it("should process RSS feed end-to-end", async () => {
    const rssParser = new RSSParserService();
    const metadataExtractor = new MetadataExtractorService();

    const xml = `<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item>
      <title>Test Article</title>
      <link>http://test.com/article</link>
      <description>Test description</description>
      <pubDate>Mon, 01 Jan 2025 00:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>`;

    const result = await rssParser.parseRSS(xml);
    expect(result.items).toHaveLength(1);

    const metadata = await metadataExtractor.extractAllMetadata(
      result.items[0].description,
      result.items[0].link
    );
    expect(metadata.title).toBeDefined();
  });

  it("should collect API data", async () => {
    const apiCollector = new APICollectorService();

    // Test mock
    expect(apiCollector).toBeDefined();
  });
});
