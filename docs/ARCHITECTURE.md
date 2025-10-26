# Hive-News Architecture

**Version:** 1.1.0  
**Last Updated:** 2025-10-26  
**Status:** Production Ready

---

## 📋 Overview

Hive-News is a **scientific news automation platform** that collects, validates, ranks, generates, and publishes AI-powered articles across multiple languages and channels.

**🧬 Scientific Validation Module** ensures authenticity and integrity of academic papers before news generation.

### Core Components

1. **Content Collection** - RSS feeds, APIs, web scraping
2. **Scientific Validation** 🧬 - Academic paper verification (NEW)
3. **Vector Search** - Semantic indexing with 512D embeddings
4. **Content Ranking** - Dynamic ranking with QA feedback loop
5. **AI Generation** - DeepSeek-powered article writing
6. **Multi-language** - 5 languages (pt-BR, es-ES, fr-FR, de-DE, zh-CN)
7. **Image Generation** - SDXL-powered AI images
8. **Automated Publishing** - Websites and social media (X.com, LinkedIn)

---

## 🏗️ System Architecture

### High-Level Flow

```
RSS/API → Collector → Metadata Extractor
                              ↓
                         Vectorizer (512D)
                              ↓
🧬 ScientificValidationService (NEW)
    - Reputation checks
    - Citation verification
    - Author verification
    - AI detection
                              ↓
                         Ranker (with QA feedback)
    - Freshness (40%)
    - Relevance (30%)
    - Trend (20%)
    - Social Signal (10%)
    - QA Penalty (applied)
                              ↓
                        DeepSeek (Writer)
                              ↓
                        Translator (5 langs)
                              ↓
                        SDXL (Images)
                              ↓
                        Publisher
```

---

## 🧬 Scientific Validation Module (NEW)

### Position in Pipeline

**Location:** After Vectorizer, Before Ranker  
**Purpose:** Verify authenticity of academic papers

### Components

1. **ScientificValidationService**
   - Conditional execution (only for academic sources)
   - Portal-specific configuration
   - Multi-factor scoring

2. **Validation Factors**
   - **Reputation Score (40%)**: Journal/conference reputation
   - **Citation Rate (30%)**: Citation accessibility verification
   - **Author Verification (20%)**: ORCID/profile checks
   - **AI Detection (10%)**: AI-generated content probability

3. **Integration with Ranker**
   ```typescript
   finalRank = (
     freshness * 0.35 +
     relevance * 0.25 +
     trend * 0.20 +
     socialSignal * 0.10 +
     validationScore * 0.10  // NEW: Scientific validation
   ) * qaPenalty
   ```

### Configuration

**Per-Portal YAML:**
```yaml
enable_scientific_validation: true
source_types:
  - arxiv.org: "academic"    # Validated
  - nature.com: "academic"   # Validated
  - techcrunch.com: "news"   # Skipped
```

---

## 📦 Module Overview

### 19 Modules Implemented

1. **Core/Application** - CMMV integration, YAML profiles, ORM
2. **Editorial** - Style presets, language config, cadence
3. **Source/Collector** - RSS, API, HTML scraping
4. **Metadata Extractor** - Title, authors, dates, abstracts
5. **Vectorizer** - 512D embeddings, semantic search
6. **Ranker** - Dynamic ranking with QA feedback loop
7. **Writer** - DeepSeek-powered article generation
8. **Translator** - Multi-language translation
9. **Image Generator** - SDXL image generation
10. **Publisher** - Website and social media publishing
11. **Scheduler** - Cron-based job scheduling
12. **Metrics** - Engagement tracking
13. **SEO/i18n** - SEO optimization, hreflang
14. **QA/Factuality** - Content validation
15. **Scientific Validation** 🧬 - Academic paper verification (NEW)
16. **Observability** - Logging, monitoring
17. **Security** - Authentication, encryption
18. **Frontend** - Next.js 15 portals
19. **GUI** - Electron desktop app

---

## 🔄 Data Flow

### 1. Collection Phase

```
Sources (RSS/API/HTML)
    ↓
Collector Service
    ↓
Metadata Extraction
    ↓
Vectorizer (embedding)
    ↓
[Stored in PostgreSQL]
```

### 2. Validation Phase (NEW) 🧬

```
Academic Papers Only
    ↓
ScientificValidationService
    ↓
Check: Reputation + Citations + Authors + AI
    ↓
validation_score (0-1)
    ↓
flagged (boolean)
```

### 3. Ranking Phase

```
Articles (all sources)
    ↓
RankerService
    ↓
Calculate: Freshness + Relevance + Trend + Social
    ↓
Apply: QA Penalty + Validation Score
    ↓
Ranked Articles
```

### 4. Generation Phase

```
Top-Ranked Articles
    ↓
DeepSeek Writer
    ↓
Generated Content
    ↓
Translator (5 languages)
    ↓
SDXL Image Generator
    ↓
Published Content
```

---

## 📊 CMMV Contracts

### 12 Contracts (including NEW Scientific Validation)

1. **EditorialContract** - Portal configurations
2. **SourceContract** - Content sources
3. **DocumentContract** - Document metadata
4. **VectorContract** - Vector embeddings (512D)
5. **ArticleContract** - Generated articles
6. **TranslationContract** - Multi-language translations
7. **ImageContract** - Generated images
8. **PublishContract** - Publishing records
9. **JobContract** - Scheduled jobs
10. **MetricContract** - Engagement metrics
11. **ValidationContract** - QA validation
12. **ScientificValidationContract** 🧬 - Academic validation (NEW)

---

## 🔌 Protocol Support

### Multi-Protocol Architecture

1. **MCP** - Model Context Protocol (full support)
   - 32 tools exposed
   - Synap integration (13 tools)
   - Vectorizer integration (19 tools)

2. **UMICP** - Universal Micro-ICP
   - Native JSON types
   - Auto documentation

3. **StreamableHTTP** - Server-Sent Events
   - Real-time metrics
   - Live search results

4. **WebSocket** - Binary RPC
   - Protobuf encoding
   - High performance

---

## 🧮 Ranking Algorithm (Enhanced)

### Formula

```typescript
finalRank = (
  freshness * 0.35 +        // Freshness (NEW: reduced from 0.4)
  relevance * 0.25 +        // Relevance (NEW: reduced from 0.3)
  trend * 0.20 +
  socialSignal * 0.10 +
  validationScore * 0.10    // NEW: Scientific validation
) * qaPenalty
```

### QA Feedback Loop

- **Rejected**: -50% penalty
- **Pending**: -10% penalty
- **Non-factual**: -30% penalty
- **Non-neutral tone**: -20% penalty
- Applied to final rank calculation

---

## 🎯 Performance Targets

- **Processing Speed**: < 5 min/article
- **API Latency**: < 500ms per request
- **Vector Search**: < 100ms per query
- **Image Generation**: < 30s per image
- **System Uptime**: 99.9%

---

## 🔐 Security

### Authentication

- JWT tokens
- OAuth2 support
- 2FA ready

### Encryption

- Vault for secrets
- AES-256-GCM
- Encrypted API keys

### API Security

- Rate limiting
- Input validation
- SQL injection prevention
- XSS prevention

---

## 📈 Monitoring

### Observability Stack

- **Logs**: Loki
- **Metrics**: Grafana + Prometheus
- **Traces**: Distributed tracing
- **Alerts**: PagerDuty integration

### Key Metrics

- Article generation rate
- Validation success rate
- API response times
- Error rates
- Engagement metrics

---

## 🚀 Deployment

### Infrastructure

- **Backend**: Docker containers
- **Database**: PostgreSQL 15+
- **Cache**: Redis
- **Storage**: MinIO/S3
- **Frontend**: Vercel/Cloudflare

### Scaling

- Horizontal scaling supported
- Load balancing ready
- Auto-scaling configured
- CDN integration

---

**Maintained by:** Hive-News Team  
**Last Updated:** 2025-10-26
