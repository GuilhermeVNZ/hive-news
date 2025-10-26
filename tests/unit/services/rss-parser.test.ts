import { describe, it, expect, beforeAll } from "vitest";
import { RSSParserService } from "../../../apps/backend-cmmv/src/services/rss-parser.service";

describe("RSSParserService", () => {
  let service: RSSParserService;

  beforeAll(() => {
    service = new RSSParserService();
  });

  it("should parse RSS feed", async () => {
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

    const result = await service.parseRSS(xml);
    expect(result.metadata.title).toBe("Test Feed");
    expect(result.items).toHaveLength(1);
    expect(result.items[0].title).toBe("Test Article");
  });

  it("should detect and parse Atom feeds", async () => {
    const xml = `<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Feed</title>
  <entry>
    <title>Atom Entry</title>
    <link href="http://test.com/atom-entry"/>
  </entry>
</feed>`;

    const result = await service.parseAtom(xml);
    expect(result.items).toHaveLength(1);
    expect(result.items[0].title).toBe("Atom Entry");
  });
});
