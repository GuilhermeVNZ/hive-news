# Hive-News Training & Operations Manual

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📚 Table of Contents

- [Getting Started](#getting-started)
- [System Architecture](#system-architecture)
- [Daily Operations](#daily-operations)
- [Monitoring & Alerts](#monitoring--alerts)
- [Troubleshooting](#troubleshooting)
- [Maintenance Tasks](#maintenance-tasks)
- [Emergency Procedures](#emergency-procedures)

---

## Getting Started

### Prerequisites

- Basic Linux/Unix knowledge
- Docker fundamentals
- Kubernetes basics (optional)
- Understanding of REST APIs

### System Overview

Hive-News is an automated scientific content generation platform with:
- **Backend**: Node.js/TypeScript services
- **Database**: PostgreSQL
- **Cache**: Redis
- **Storage**: MinIO (S3-compatible)
- **Frontend**: Next.js portals
- **Deployment**: Docker/Kubernetes

---

## System Architecture

### Components

```
┌─────────────────┐
│   Load Balancer │
└────────┬────────┘
         │
    ┌────┴─────┐
    │ Ingress  │
    └────┬─────┘
         │
    ┌────┴────────────────────┐
    │                         │
┌───┴───┐              ┌──────┴───────┐
│       │              │              │
│ Backend│              │   Frontend    │
│   API  │              │   (Next.js)   │
└───┬───┘              └──────┬───────┘
    │                         │
    │    ┌─────────────┐      │
    └────┤ PostgreSQL   │──────┘
         │   Database  │
         └──────┬──────┘
                │
    ┌───────────┼───────────┐
    │           │           │
┌───┴───┐  ┌────┴────┐  ┌───┴────┐
│ Redis │  │  MinIO │  │Vectorizer│
│ Cache │  │Storage │  │   API   │
└───────┘  └────────┘  └─────────┘
```

### Service Dependencies

- Backend → PostgreSQL, Redis, Vectorizer, Synap
- Frontend → Backend API
- Publishers → External APIs (Twitter, LinkedIn, etc.)

---

## Daily Operations

### Morning Checklist

- [ ] Check system health dashboard
- [ ] Review error logs
- [ ] Verify backup completion
- [ ] Check article generation status
- [ ] Monitor API rate limits
- [ ] Review published articles

### Common Tasks

#### View Logs

```bash
# Docker
docker-compose logs -f backend

# Kubernetes
kubectl logs -f deployment/hivenews-backend -n hivenews
```

#### Check System Status

```bash
# Health check
curl http://localhost:3000/health

# Metrics
curl http://localhost:8080/metrics
```

#### View Database

```bash
# Connect to PostgreSQL
docker exec -it hivenews-postgres psql -U hivenews -d hivenews

# Check articles
SELECT COUNT(*) FROM articles WHERE published_at > NOW() - INTERVAL '24 hours';
```

---

## Monitoring & Alerts

### Key Metrics

| Metric | Threshold | Action |
|--------|-----------|--------|
| CPU Usage | > 80% | Scale up |
| Memory Usage | > 85% | Scale up |
| Error Rate | > 5% | Investigate |
| Response Time | > 1s | Optimize |
| Queue Depth | > 1000 | Scale workers |

### Dashboard Access

- **Grafana**: http://grafana.local/dashboard/hivenews
- **Prometheus**: http://prometheus.local/graph
- **MinIO**: http://minio.local:9001

### Setting Up Alerts

```yaml
# Example alert configuration
groups:
  - name: hivenews
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status="5xx"}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "Error rate exceeded threshold"
          description: "Error rate is {{ $value }}"
```

---

## Troubleshooting

### Common Issues

#### Issue: High Memory Usage

**Symptoms:**
- Pod restarts frequently
- Container OOMKilled

**Solutions:**
```bash
# Check memory usage
kubectl top pod -n hivenews

# Increase memory limit
kubectl edit deployment hivenews-backend -n hivenews
# Update resources.limits.memory

# Or scale down temporarily
kubectl scale deployment hivenews-backend --replicas=2 -n hivenews
```

#### Issue: Database Connection Errors

**Symptoms:**
- Connection refused errors
- Timeout errors

**Solutions:**
```bash
# Check PostgreSQL status
docker-compose ps postgres
kubectl get pods -n hivenews | grep postgres

# Check logs
docker-compose logs postgres
kubectl logs -n hivenews -l app=postgres

# Restart database
docker-compose restart postgres
kubectl rollout restart deployment/hivenews-postgres -n hivenews
```

#### Issue: API Rate Limits

**Symptoms:**
- 429 errors from external APIs
- Slow article generation

**Solutions:**
```bash
# Check rate limit status
curl http://localhost:3000/metrics | grep rate_limit

# Adjust rate limiting
# Edit .env or ConfigMap
K8S: kubectl edit configmap hivenews-config -n hivenews
```

---

## Maintenance Tasks

### Weekly Tasks

- [ ] Review and rotate logs
- [ ] Check disk usage
- [ ] Verify backup integrity
- [ ] Update dependencies (dev)
- [ ] Review security alerts

### Monthly Tasks

- [ ] Test disaster recovery
- [ ] Review performance metrics
- [ ] Update documentation
- [ ] Capacity planning review
- [ ] Security audit

### Quarterly Tasks

- [ ] Full system backup
- [ ] Disaster recovery drill
- [ ] Performance optimization
- [ ] Dependency updates
- [ ] Architecture review

---

## Emergency Procedures

### Service Down

1. **Diagnose**: Check health endpoints
2. **Check Logs**: Review recent errors
3. **Restart**: Attempt graceful restart
4. **Scale**: Scale up if needed
5. **Escalate**: Contact on-call engineer

### Data Loss

1. **Stop Services**: Prevent further data loss
2. **Assess**: Determine scope of loss
3. **Restore**: Use latest backup
4. **Verify**: Check data integrity
5. **Resume**: Start services

### Security Breach

1. **Isolate**: Quarantine affected systems
2. **Assess**: Determine attack vector
3. **Mitigate**: Block attack source
4. **Restore**: Rebuild from clean state
5. **Report**: Document incident

---

## Support

### Getting Help

- **Documentation**: `/docs` directory
- **GitHub Issues**: [Repository](#)
- **Email**: support@hivenews.com
- **Slack**: #hivenews-ops

### On-Call Rotation

- **Primary**: Primary engineer
- **Secondary**: Backup engineer
- **Escalation**: Team lead

---

## Resources

### Documentation

- [Deployment Guide](./DEPLOYMENT.md)
- [Monitoring Guide](../MONITORING.md)
- [Backup & Recovery](./BACKUP_RECOVERY.md)
- [Architecture](./ARCHITECTURE.md)

### External Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Redis Documentation](https://redis.io/documentation)
- [Docker Documentation](https://docs.docker.com/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)

---

**Last Updated:** 2025-10-26  
**Maintained by:** Hive-News Team

