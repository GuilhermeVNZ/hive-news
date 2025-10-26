# Hive-News Deployment Guide

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [CI/CD Pipeline](#cicd-pipeline)
- [Environment Variables](#environment-variables)
- [Monitoring](#monitoring)
- [Backup & Recovery](#backup--recovery)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required
- **Node.js:** 20.x or higher
- **Docker:** 20.10 or higher
- **Docker Compose:** 2.0 or higher
- **Kubernetes:** 1.24 or higher (optional)
- **PostgreSQL:** 15 or higher
- **Redis:** 7 or higher

### Optional
- **MinIO** (for S3-compatible storage)
- **Vectorizer** (for semantic search)
- **Synap** (for MCP storage)

---

## Quick Start

### Development Environment

```bash
# Clone repository
git clone https://github.com/your-org/hivenews.git
cd hivenews

# Install dependencies
npm install

# Start services with Docker Compose
docker-compose up -d

# Run migrations
npm run migrate

# Start development server
npm run dev
```

---

## Docker Deployment

### Build Docker Images

```bash
# Build backend image
docker build -t hivenews/backend:latest -f docker/Dockerfile .

# Build frontend image (if applicable)
docker build -t hivenews/frontend:latest -f apps/frontend-next/Dockerfile .
```

### Run with Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Environment Variables

Create `.env` file:

```env
# Database
DATABASE_URL=postgresql://user:password@postgres:5432/hivenews

# DeepSeek API
DEEPSEEK_API_KEY=your-api-key

# Vectorizer
VECTORIZER_URL=http://vectorizer:15002
VECTORIZER_API_KEY=your-api-key

# Synap
SYNAP_URL=http://synap:15500

# Redis
REDIS_URL=redis://redis:6379

# MinIO
MINIO_URL=http://minio:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
```

---

## Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (1.24+)
- kubectl configured
- Helm 3.x (optional)

### Step 1: Create Namespace

```bash
kubectl apply -f k8s/namespace.yaml
```

### Step 2: Create Secrets

```bash
# Copy example secrets
cp k8s/secrets.yaml.example k8s/secrets.yaml

# Edit secrets.yaml with your values
nano k8s/secrets.yaml

# Apply secrets
kubectl apply -f k8s/secrets.yaml
```

### Step 3: Deploy Applications

```bash
# Deploy backend
kubectl apply -f k8s/deployment.yaml

# Deploy services
kubectl apply -f k8s/service.yaml

# Deploy ingress (if using)
kubectl apply -f k8s/ingress.yaml

# Check status
kubectl get pods -n hivenews
kubectl get services -n hivenews
```

### Step 4: Verify Deployment

```bash
# Check pod status
kubectl get pods -n hivenews

# Check logs
kubectl logs -f deployment/hivenews-backend -n hivenews

# Port forward for testing
kubectl port-forward -n hivenews deployment/hivenews-backend 3000:3000
```

---

## CI/CD Pipeline

### GitHub Actions

The project includes automated CI/CD pipelines:

- **`.github/workflows/ci.yml`** - Continuous Integration
- **`.github/workflows/cd.yml`** - Continuous Deployment
- **`.github/workflows/lint.yml`** - Code Quality Checks

### Setup Secrets

In GitHub repository settings, add:

- `DOCKER_USERNAME`
- `DOCKER_PASSWORD`
- `DOCKER_HUB_TOKEN`
- `SLACK_WEBHOOK` (optional)

### Workflow Overview

1. **CI on Push/PR**: Runs tests, lint, type-check
2. **Build on Success**: Creates Docker images
3. **Deploy on Tag**: Deploys to production

### Manual Deployment

```bash
# Tag release
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# This triggers CD pipeline
```

---

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/db` |
| `DEEPSEEK_API_KEY` | DeepSeek API key | `sk-...` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_ENV` | Environment | `development` |
| `PORT` | Server port | `3000` |
| `LOG_LEVEL` | Logging level | `info` |
| `VECTORIZER_URL` | Vectorizer endpoint | - |
| `SYNAP_URL` | Synap endpoint | - |
| `REDIS_URL` | Redis connection | - |

---

## Monitoring

### Health Endpoints

- **Health Check**: `GET /health`
- **Readiness**: `GET /ready`
- **Metrics**: `GET /metrics`

### Prometheus Metrics

```yaml
# Exposed on port 8080
scrape_configs:
  - job_name: 'hivenews'
    static_configs:
      - targets: ['localhost:8080']
```

### Logging

```bash
# View logs
docker-compose logs -f backend

# View specific service
kubectl logs -f deployment/hivenews-backend -n hivenews
```

---

## Backup & Recovery

### Database Backup

```bash
# Backup PostgreSQL
docker exec hivenews-postgres pg_dump -U hivenews hivenews > backup.sql

# Restore
docker exec -i hivenews-postgres psql -U hivenews hivenews < backup.sql
```

### Volume Backup

```bash
# Backup volumes
docker run --rm -v hivenews_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres.tar.gz /data
```

---

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL status
docker-compose ps postgres
docker-compose logs postgres

# Test connection
docker exec -it hivenews-postgres psql -U hivenews -d hivenews
```

### Pod Not Starting

```bash
# Check pod events
kubectl describe pod -n hivenews <pod-name>

# Check logs
kubectl logs -n hivenews <pod-name>

# Restart deployment
kubectl rollout restart deployment/hivenews-backend -n hivenews
```

### High Memory Usage

```bash
# Check resource usage
kubectl top pods -n hivenews

# Scale down
kubectl scale deployment/hivenews-backend --replicas=1 -n hivenews
```

---

## Production Checklist

- [ ] Environment variables configured
- [ ] Secrets properly secured
- [ ] Database migrations applied
- [ ] Health checks configured
- [ ] Logging configured
- [ ] Monitoring set up
- [ ] Backup strategy implemented
- [ ] Disaster recovery plan documented
- [ ] SSL/TLS certificates configured
- [ ] Rate limiting enabled
- [ ] DDoS protection configured
- [ ] CDN configured (if using)
- [ ] Load balancer configured

---

## Support

For issues or questions:

- **Documentation**: `/docs` directory
- **GitHub Issues**: [Repository Issues](#)
- **Email**: support@hivenews.com

---

**Last Updated:** 2025-10-26  
**Maintained by:** Hive-News Team

