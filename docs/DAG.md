# Hive-News Dependency Graph (DAG)

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📊 Module Dependencies

```
┌──────────────────────────────────────────────────────────────┐
│                     Application Core                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │   @cmmv/core - Contract Engine                        │  │
│  │   @cmmv/repository - TypeORM                           │  │
│  │   @cmmv/cache - Redis/Synap                           │  │
│  │   @cmmv/vault - Encryption                            │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Editorial   │    │     Source    │    │   Scheduler   │
│   Contract    │    │   Collector   │    │     Module    │
└───────────────┘    └───────────────┘    └───────────────┘
        │                    │                    │
        │                    ▼                    │
        │            ┌───────────────┐           │
        │            │   Metadata    │           │
        │            │   Extractor   │           │
        │            └───────────────┘           │
        │                    │                   │
        │                    ▼                   │
        │            ┌───────────────┐           │
        │            │  Vectorizer   │           │
        │            │   Module      │           │
        │            └───────────────┘           │
        │                    │                   │
        │                    ▼                   │
        │            ┌───────────────┐           │
        │            │    Ranker     │           │
        │            └───────────────┘           │
        │                    │                   │
        └────────────────────┼───────────────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
                ▼                         ▼
        ┌───────────────┐         ┌───────────────┐
        │    Writer     │         │  Translator  │
        │  (DeepSeek)   │         │  (DeepSeek)  │
        └───────────────┘         └───────────────┘
                │                         │
                └─────────────┬───────────┘
                              │
                              ▼
                    ┌───────────────┐
                    │   Image Gen   │
                    │    (SDXL)     │
                    └───────────────┘
                              │
                              ▼
                    ┌───────────────┐
                    │   Publisher   │
                    └───────────────┘
```

---

## 🔗 Contract Dependencies

### Editorial Contract → Uses:

```
@Contract('Editorial')
  ├─→ Sources: Portal configuration
  ├─→ Ranker: Ranking preferences
  └─→ Publisher: Publishing settings
```

### Source Contract → Uses:

```
@Contract('Source')
  ├─→ Editorial: Portal reference
  ├─→ Vectorizer: Document processing
  └─→ Metadata Extractor: Content parsing
```

### Article Contract → Uses:

```
@Contract('Article')
  ├─→ Editorial: Style configuration
  ├─→ Writer: Content generation
  ├─→ Ranker: Ranking score
  ├─→ Publisher: Publishing status
  └─→ Metrics: Engagement tracking
```

### Translation Contract → Uses:

```
@Contract('Translation')
  ├─→ Article: Source article
  ├─→ Translator: Translation service
  └─→ SEO: i18n optimization
```

---

## 🗄️ Storage Dependencies

### PostgreSQL Dependencies

```
PostgreSQL
  ├─→ TypeORM Entities
  │   ├─→ Editorial Entity
  │   ├─→ Article Entity
  │   ├─→ Translation Entity
  │   ├─→ Metric Entity
  │   └─→ Job Entity
  │
  └─→ @cmmv/repository
      ├─→ Read operations
      ├─→ Write operations
      └─→ Relation management
```

### Vectorizer Dependencies

```
Vectorizer
  ├─→ Document storage
  │   ├─→ Document Contract
  │   ├─→ Vector Contract
  │   └─→ Collection management
  │
  ├─→ Search operations
  │   ├─→ Source Collector
  │   ├─→ Ranker
  │   └─→ HDQL queries
  │
  └─→ Transmutation
      ├─→ PDF processing
      ├─→ DOCX processing
      ├─→ Image OCR
      └─→ HTML parsing
```

### Synap Dependencies

```
Synap
  ├─→ Cache layer
  │   ├─→ Article cache
  │   ├─→ Rank cache
  │   └─→ Metadata cache
  │
  ├─→ Task queues
  │   ├─→ Collector jobs
  │   ├─→ Translation jobs
  │   └─→ Publishing jobs
  │
  └─→ Real-time events
      ├─→ Update streams
      └─→ Metric broadcasts
```

---

## 🔄 Process Flow Dependencies

### Collection Flow

```
1. Scheduler triggers collection
   │
   ├─→ Source Collector
   │   │
   │   ├─→ Fetch from RSS/API/HTML
   │   │
   │   └─→ Send to Vectorizer
   │       │
   │       ├─→ Transmutation
   │       │
   │       └─→ Metadata Extract
   │
   └─→ Store in PostgreSQL
```

### Generation Flow

```
1. Ranker selects articles
   │
   ├─→ Writer (DeepSeek)
   │   │
   │   ├─→ Generate article
   │   │
   │   └─→ Store Article Contract
   │
   ├─→ Translator (DeepSeek)
   │   │
   │   ├─→ Translate to target languages
   │   │
   │   └─→ Store Translation Contracts
   │
   └─→ Image Generator (SDXL)
       │
       ├─→ Generate images
       │
       └─→ Store Image Contracts
```

### Publishing Flow

```
1. Publisher triggered
   │
   ├─→ SEO Module
   │   │
   │   ├─→ Generate metadata
   │   │
   │   └─→ Create hreflang tags
   │
   ├─→ Social Formatter
   │   │
   │   ├─→ X.com format
   │   │
   │   └─→ LinkedIn format
   │
   └─→ Publish
       │
       ├─→ Website
       ├─→ X.com
       └─→ LinkedIn
```

---

## 🧩 Module Isolation Levels

### Completely Independent

- ✅ **Editorial Module**: No runtime dependencies
- ✅ **Scheduler**: Independent job orchestration
- ✅ **Observability**: Logging and metrics only

### Core Dependencies Only

- ⚠️ **Source Collector**: Requires Editorial config
- ⚠️ **Metadata Extractor**: Requires Vectorizer output
- ⚠️ **Ranker**: Requires Metrics data

### Multiple Dependencies

- 🔴 **Writer**: Needs Ranker + Editorial
- 🔴 **Translator**: Needs Article + Writer
- 🔴 **Publisher**: Needs Article + Translation + Images
- 🔴 **Metrics**: Needs Article + Publisher data

---

## 📊 Integration Points

### MCP Integration

```
MCP Server (Port 15500)
  ├─→ Synap operations
  ├─→ AI assistant tools
  └─→ Real-time updates
```

### UMICP Integration

```
UMICP Server (Port 15002)
  ├─→ Vectorizer operations
  ├─→ Tool discovery
  └─→ Native JSON types
```

### StreamableHTTP

```
Streaming Endpoints
  ├─→ /stream/live
  ├─→ /stream/metrics
  └─→ /stream/results
```

---

## 🔧 Build Order

### Phase 1: Core

1. @cmmv/core
2. @cmmv/repository
3. @cmmv/cache
4. Application Core

### Phase 2: Storage

1. PostgreSQL setup
2. Vectorizer setup
3. Synap setup
4. Integration tests

### Phase 3: Modules

1. Editorial
2. Source Collector
3. Metadata Extractor
4. Ranker

### Phase 4: AI

1. Writer
2. Translator
3. Image Generator

### Phase 5: Publishing

1. Publisher
2. SEO Module
3. Social Integration

### Phase 6: UI

1. Electron GUI
2. HDQL Parser
3. Query Builder

---

## ⚠️ Breaking Changes

When modifying contracts:

1. Check all dependent modules
2. Update related tests
3. Update documentation
4. Generate migration scripts
5. Run integration tests

---

**Maintained by:** Hive-News Architecture Team
