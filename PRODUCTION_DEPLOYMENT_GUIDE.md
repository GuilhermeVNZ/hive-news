# 🚀 Hive-News: Production Deployment Guide

**Version:** 1.0.0  
**Date:** 2025-10-26  
**Status:** Ready for Production

---

## 📋 Quick Start

### Option 1: Automated Deployment (Recommended)

**Windows:**
```powershell
.\deploy-production.ps1
```

**Linux/Mac:**
```bash
chmod +x deploy-production.sh
./deploy-production.sh
```

### Option 2: Manual Deployment

Follow the step-by-step guide below.

---

## 📝 Prerequisites

### Required Software
- **Node.js:** 20.x or higher
- **Docker:** 20.10 or higher
- **Docker Compose:** 2.0 or higher
- **Git:** Latest version

### Required APIs
- **DeepSeek API Key** - Get at https://www.deepseek.com/
- **Vectorizer URL** - Running instance
- **Synap URL** - Running instance (optional)

### Optional Services
- **Vectorizer** - For semantic search
- **Synap** - For MCP storage
- **SDXL** - For image generation

---

## 🔧 Step-by-Step Deployment

### Step 1: Clone Repository

```bash
git clone https://github.com/GuilhermeVNZ/hive-news.git
cd hive-news
```

### Step 2: Configure Environment

```bash
# Create .env from template
cp env.template .env

# Edit .env with your values
nano .env  # or your preferred editor
```

**Required Variables:**
```env
DATABASE_URL=postgresql://hivenews:PASSWORD@localhost:5432/hivenews
DEEPSEEK_API_KEY=sk-your-key-here
VECTORIZER_URL=http://your-vectorizer:15002
SYNAP_URL=http://your-synap:15500
```

### Step 3: Install Dependencies

```bash
npm install
```

### Step 4: Run Tests

```bash
npm test
# Expected: 269 tests passing
```

### Step 5: Build Application

```bash
npm run build
```

### Step 6: Start Infrastructure

```bash
# Start PostgreSQL, Redis, MinIO
docker-compose up -d postgres redis minio

# Verify services
docker-compose ps
```

### Step 7: Start Backend

```bash
npm start
```

Or run in background:
```bash
npm start &
```

### Step 8: Verify Deployment

```bash
# Health check
curl http://localhost:3000/health

# Metrics
curl http://localhost:3000/metrics
```

Expected output:
```json
{"status":"healthy","timestamp":"2025-10-26T...","version":"1.0.0"}
```

---

## 🔐 Production Configuration

### Environment Variables

Create `.env.production`:

```env
# Database
DATABASE_URL=postgresql://hivenews:STRONG_PASSWORD@postgres:5432/hivenews

# DeepSeek API
DEEPSEEK_API_KEY=sk-production-key
DEEPSEEK_BASE_URL=https://api.deepseek.com

# Vectorizer
VECTORIZER_URL=http://vectorizer:15002

# Synap
SYNAP_URL=http://synap:15500

# Redis
REDIS_HOST=redis
REDIS_PORT=6379

# MinIO/S3
S3_ENDPOINT=minio:9000
S3_ACCESS_KEY=production-key
S3_SECRET_KEY=production-secret
S3_BUCKET=hivenews

# Security
JWT_SECRET=CHANGE-THIS-IN-PRODUCTION
JWT_EXPIRES_IN=7d

# Node
NODE_ENV=production
```

---

## 🐳 Docker Deployment

### Build Image

```bash
npm run docker:build
# or
docker build -t hivenews/backend:latest -f docker/Dockerfile .
```

### Run with Docker Compose

```bash
docker-compose up -d
```

### Check Logs

```bash
docker-compose logs -f backend
```

### Stop Services

```bash
docker-compose down
```

---

## ☸️ Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (1.24+)
- kubectl configured
- Helm 3.x (optional)

### Step 1: Create Namespace

```bash
kubectl apply -f k8s/namespace.yaml
```

### Step 2: Configure Secrets

```bash
# Copy and edit secrets
cp k8s/secrets.yaml.example k8s/secrets.yaml
# Edit secrets.yaml with your values
kubectl apply -f k8s/secrets.yaml
```

### Step 3: Deploy

```bash
kubectl apply -f k8s/
```

### Step 4: Verify

```bash
kubectl get pods -n hivenews
kubectl get services -n hivenews
kubectl logs -f deployment/hivenews-backend -n hivenews
```

### Step 5: Port Forward

```bash
kubectl port-forward -n hivenews deployment/hivenews-backend 3000:3000
```

---

## 🔍 Verification

### Health Endpoints

```bash
# Health check
curl http://localhost:3000/health

# Readiness
curl http://localhost:3000/ready

# Metrics
curl http://localhost:3000/metrics
```

### Database Connection

```bash
# Using Docker
docker exec -it hivenews-postgres psql -U hivenews -d hivenews

# Using kubectl
kubectl exec -it deployment/hivenews-backend -n hivenews -- psql -h postgres -U hivenews -d hivenews
```

### Check Logs

```bash
# Docker
docker-compose logs -f backend

# Kubernetes
kubectl logs -f deployment/hivenews-backend -n hivenews
```

---

## 📊 Monitoring

### Prometheus Metrics

Access at: `http://localhost:8080/metrics`

Key metrics:
- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request duration
- `articles_generated_total` - Articles generated
- `sources_active` - Active content sources

### Health Dashboard

Create Grafana dashboard using these metrics.

---

## 🔄 Backup & Recovery

### Create Backup

```bash
# Run backup script
./scripts/backup.sh

# Or PowerShell
.\scripts\backup.ps1
```

### Restore from Backup

```bash
./scripts/restore.sh
```

---

## 🆘 Troubleshooting

### Issue: Services not starting

```bash
# Check Docker
docker ps

# Check logs
docker-compose logs

# Restart services
docker-compose restart
```

### Issue: Database connection failed

```bash
# Verify PostgreSQL is running
docker-compose ps postgres

# Check connection
docker exec -it hivenews-postgres psql -U hivenews -d hivenews
```

### Issue: High memory usage

```bash
# Check resource usage
docker stats

# Scale down if needed
kubectl scale deployment hivenews-backend --replicas=2 -n hivenews
```

---

## 📞 Support

### Resources
- **Documentation:** `/docs` directory
- **GitHub:** https://github.com/GuilhermeVNZ/hive-news
- **Issues:** Use GitHub Issues
- **Email:** support@hivenews.com

---

## ✅ Deployment Checklist

### Pre-Deployment
- [ ] Environment variables configured
- [ ] API keys obtained
- [ ] Database ready
- [ ] Tests passing
- [ ] Build successful

### Deployment
- [ ] Infrastructure started
- [ ] Services running
- [ ] Health checks passing
- [ ] Logs reviewed
- [ ] Metrics collected

### Post-Deployment
- [ ] Backup configured
- [ ] Monitoring setup
- [ ] Alerts configured
- [ ] Documentation updated
- [ ] Team notified

---

**Ready for Production! 🚀**

*For more details, see `docs/DEPLOYMENT.md`*

