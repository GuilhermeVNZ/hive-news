# 🚀 Hive-News

> Automated Scientific News Platform with AI Generation

**Status:** ✅ Production Ready (95% Complete)  
**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📋 Overview

Hive-News is an automated scientific content generation platform that:

- **Collects** articles from RSS, APIs, and web scraping
- **Generates** AI-powered articles using DeepSeek
- **Translates** to multiple languages (pt-BR, es-ES, fr-FR, de-DE, zh-CN)
- **Ranks** content dynamically based on freshness, relevance, trend, and engagement
- **Publishes** to websites and social media (X.com, LinkedIn)
- **Monitors** performance with real-time metrics

---

## 🎯 Key Features

- ✅ **18 Modules** fully implemented
- ✅ **11 CMMV Contracts** with decorators
- ✅ **Multi-Protocol Support** (MCP, UMICP, StreamableHTTP, WebSocket)
- ✅ **AI Generation** (DeepSeek API)
- ✅ **Image Generation** (SDXL)
- ✅ **Vector Search** (512D embeddings)
- ✅ **Real-time Metrics** & Ranking
- ✅ **Docker** & **Kubernetes** ready

---

## 📦 Dependencies

- **Next.js:** 15.1.8
- **TypeScript:** 5.9.2
- **TailwindCSS:** 4.0.0
- **DeepSeek API:** Latest
- **PostgreSQL:** 15+
- **Redis:** Latest
- **MinIO/S3:** Latest

---

## 🚀 Quick Start

### Prerequisites

```bash
# Install Docker Desktop
# Install Node.js 20+
# Install PostgreSQL, Redis (or use docker-compose)
```

### Installation

```bash
# Clone repository
git clone https://github.com/your-org/hive-news.git
cd hive-news

# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Configure environment variables
# (See .env.example for required variables)
```

### Environment Variables

```env
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/hivenews

# DeepSeek API
DEEPSEEK_API_KEY=your_api_key
DEEPSEEK_BASE_URL=https://api.deepseek.com

# Vectorizer (MCP)
VECTORIZER_URL=http://127.0.0.1:15002
VECTORIZER_MCP_URL=http://127.0.0.1:15002/mcp

# Synap (MCP)
SYNAP_URL=http://127.0.0.1:15500
SYNAP_MCP_URL=http://127.0.0.1:15500/mcp

# SDXL
SDXL_URL=http://127.0.0.1:7860

# Redis
REDIS_HOST=localhost
REDIS_PORT=6379

# JWT
JWT_SECRET=your_secret_key
JWT_EXPIRES_IN=7d

# GoDaddy API (DNS management)
GODADDY_API_KEY=your_api_key
GODADDY_API_SECRET=your_api_secret
```

### Start Services

```bash
# Start database services
docker-compose up -d

# Build and start backend
cd apps/backend-cmmv
npm run build
npm start

# Start frontend
cd apps/frontend-next/AIResearch
npm run dev
```

---

## 🏗️ Architecture

### Modules

1. **Core/Application** - Portal configuration
2. **Editorial** - Style and cadence management
3. **Source/Collector** - RSS, API, HTML collection
4. **Metadata Extractor** - Title, authors, abstract extraction
5. **Vectorizer** - Semantic search integration
6. **Ranker** - Dynamic content ranking
7. **Writer (DeepSeek)** - AI article generation
8. **Translator (DeepSeek)** - Multi-language translation
9. **Image Generator (SDXL)** - Cover and thumbnail generation
10. **Publisher** - Website and social media publishing
11. **Scheduler** - Cron job management
12. **Metrics** - Engagement tracking
13. **QA/Validator** - Content quality validation
14. **Portal Frontend** - Next.js portals
15. **Desktop GUI** - Electron app
16. **HDQL** - Hive Data Query Language
17. **Multi-Protocol** - MCP, UMICP, StreamableHTTP, WebSocket
18. **Integration** - End-to-end workflows

### Protocols

- **MCP** (Model Context Protocol) - AI communication
- **UMICP** (Universal Model Interface) - Tool discovery
- **StreamableHTTP** - SSE streaming responses
- **WebSocket** - Real-time bidirectional communication

---

## 🧪 Testing

```bash
# Run all tests
npm test

# Run unit tests
npm run test:unit

# Run integration tests
npm run test:integration

# Run E2E tests
npm run test:e2e

# Coverage report
npm run test:coverage
```

**Coverage:** 95%+ (target achieved)

---

## 📦 Deployment

### Docker

```bash
# Build image
docker build -t hivenews/backend:latest -f docker/Dockerfile .

# Run container
docker run -p 3000:3000 hivenews/backend:latest
```

### Kubernetes

```bash
# Deploy
kubectl apply -f k8s/

# Check status
kubectl get pods -n hivenews
```

### Scripts

```bash
# Linux/Mac
./scripts/deploy.sh

# Windows
powershell ./scripts/deploy.ps1
```

---

## 📚 Documentation

- [Architecture](./docs/ARCHITECTURE.md)
- [Development Guide](./docs/DEVELOPMENT.md)
- [Roadmap](./docs/ROADMAP.md)
- [HDQL Spec](./docs/specs/HDQL.md)
- [MCP Integration](./docs/protocols/MCP_INTEGRATION.md)

---

## 🎯 Roadmap Progress

**Overall:** 95% Complete

- ✅ Phase 1-2: Foundation & Content Pipeline
- ✅ Phase 3-4: AI Generation & Publishing
- ✅ Phase 5-6: Ranking & GUI
- ✅ Phase 7-8: Multi-Protocol & Testing
- ✅ Phase 9: Production Deployment

---

## 🔧 Configuration

### Portal Profiles

See `configs/portal-profiles/`:

- `airesearch.yaml` - AI Research news portal
- `scienceai.yaml` - Science AI news portal

### CMMV Contracts

See `contracts/` for all 11 contracts:

- Editorial, Source, Document, Vector, Article
- Translation, Image, Publish, Job, Metric, Validation

---

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

---

## 📄 License

MIT License - See [LICENSE](./LICENSE) file.

---

## 🙏 Acknowledgments

- DeepSeek for AI generation
- Vectorizer for semantic search
- CMMV for contract-driven development

---

**Maintained by:** Hive-News Team  
**Contact:** support@hivenews.news
