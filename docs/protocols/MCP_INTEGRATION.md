# MCP Integration Guide

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## Overview

Hive-News integrates with **Model Context Protocol (MCP)** for AI assistant integration and automated content processing.

### MCP Servers

| Server             | Port  | Purpose                           | Status    |
| ------------------ | ----- | --------------------------------- | --------- |
| **Synap MCP**      | 15500 | Cache, queues, pub/sub, streaming | ✅ Active |
| **Vectorizer MCP** | 15002 | Document search and indexing      | ✅ Active |

---

## Synap MCP Integration

### Configuration

```json
{
  "mcpServers": {
    "synap": {
      "type": "http",
      "url": "http://127.0.0.1:15500/mcp"
    }
  }
}
```

### Available Tools (13)

#### Key-Value Store (4)

- `synap_kv_get` - Retrieve value by key
- `synap_kv_set` - Store value with TTL
- `synap_kv_delete` - Delete key
- `synap_kv_scan` - Scan keys by prefix

#### Message Queues (2)

- `synap_queue_publish` - Publish to queue
- `synap_queue_consume` - Consume from queue

#### Event Streams (1)

- `synap_stream_publish` - Publish event to stream

#### Pub/Sub (1)

- `synap_pubsub_publish` - Publish to topic

#### Utilities (5)

- `synap_health_check` - Check server health
- `synap_get_stats` - Get statistics
- `synap_list_keys` - List all keys
- `synap_flush_db` - Clear database
- `synap_ping` - Ping server

### Usage Examples

#### Store Article Metrics

```typescript
await mcp.synap_kv_set({
  key: "article:123:metrics",
  value: JSON.stringify({
    views: 150,
    clicks: 45,
    engagement_rate: 0.3,
    updated_at: new Date().toISOString(),
  }),
  ttl: 3600, // 1 hour TTL
});
```

#### Queue Translation Job

```typescript
await mcp.synap_queue_publish({
  queue: "translation",
  message: JSON.stringify({
    article_id: "123",
    source_lang: "en",
    target_langs: ["pt", "es"],
  }),
  priority: 7,
});
```

#### Real-Time Updates

```typescript
await mcp.synap_stream_publish({
  room: "article-updates",
  event: "published",
  data: {
    article_id: "123",
    portal_id: "airesearch",
    title: "New Article Published",
  },
});
```

---

## Vectorizer MCP Integration

### Configuration

```json
{
  "mcpServers": {
    "vectorizer-main": {
      "type": "http",
      "url": "http://127.0.0.1:15002/mcp"
    }
  }
}
```

### Available Tools

- `list_collections` - List all collections
- `search_intelligent` - AI-powered search
- `search_semantic` - Semantic search with reranking
- `insert_text` - Index document
- `get_file_content` - Retrieve file content
- `list_files` - List indexed files
- `get_related_files` - Find related files

### Usage Examples

#### Search for Related Content

```typescript
const results = await mcp.vectorizer_search_intelligent({
  collections: ["airesearch-docs", "scienceai-docs"],
  query: "machine learning breakthroughs",
  max_results: 10,
  similarity_threshold: 0.7,
});
```

#### Index New Document

```typescript
await mcp.vectorizer_insert_text({
  collection_name: "airesearch-docs",
  text: "Article content...",
  metadata: {
    article_id: "123",
    title: "New Article",
    published_at: new Date().toISOString(),
  },
});
```

---

## Protocol Implementation

### StreamableHTTP

Hive-News uses Server-Sent Events for real-time updates:

```typescript
const stream = new EventSource("http://localhost:3000/stream/live");

stream.addEventListener("message", (event) => {
  const data = JSON.parse(event.data);
  console.log("Update:", data);
});
```

### WebSocket RPC

Binary protobuf communication for high-performance:

```typescript
const client = new RPCClient({
  url: "ws://localhost:3000/rpc",
  protocol: "protobuf",
});

const result = await client.call("ArticleService.create", {
  title: "New Article",
  body: "Content...",
});
```

---

## Best Practices

1. **Use TTL wisely:** Set appropriate expiration for cache data
2. **Priority queues:** Use higher priority for urgent jobs
3. **Streaming:** Subscribe to streams for real-time updates
4. **Batch operations:** Combine multiple operations when possible
5. **Error handling:** Always handle connection failures

---

## Troubleshooting

### Connection Issues

```bash
# Check if Synap is running
curl http://localhost:15500/health

# Check if Vectorizer is running
curl http://localhost:15002/health

# Check processes
ps aux | grep synap-server
ps aux | grep vectorizer
```

### Common Errors

**404 Not Found**

- Server not running
- Wrong port
- Incorrect URL path

**401 Unauthorized**

- Missing API key
- Invalid credentials

**500 Internal Server Error**

- Server configuration issue
- Check server logs

---

**Authors:** Hive-News Protocol Team
