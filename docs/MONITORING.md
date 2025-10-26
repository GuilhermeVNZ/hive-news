# Hive-News Monitoring Guide

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📊 Overview

Hive-News includes comprehensive monitoring capabilities to track system health, performance, and business metrics.

---

## 🔍 Metrics Exposed

### Application Metrics

- **HTTP Requests**: Request count, duration, status codes
- **Error Rate**: 4xx, 5xx error counts
- **Database**: Connection pool, query duration
- **Cache**: Hit/miss rates, eviction counts
- **Queue**: Job count, processing time

### Business Metrics

- **Articles Generated**: Total articles created
- **Sources Active**: Active content sources
- **Publishing Rate**: Articles published per hour
- **Engagement**: Views, clicks, time on page
- **Rank**: Average rank score

### System Metrics

- **CPU Usage**: Per service
- **Memory Usage**: Heap, RSS
- **Disk I/O**: Read/write operations
- **Network**: Bandwidth, latency

---

## 🎯 Monitoring Setup

### Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'hivenews'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana Dashboards

1. **System Overview**: CPU, memory, network
2. **Application Metrics**: Requests, errors, throughput
3. **Business Metrics**: Articles, sources, engagement
4. **Database**: Query performance, connections

### Alerts

```yaml
# alert rules
groups:
  - name: hivenews_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status="5xx"}[5m]) > 0.05
        annotations:
          summary: "High error rate detected"
      
      - alert: DatabaseConnectionFailure
        expr: up{job="postgres"} == 0
        annotations:
          summary: "Database connection lost"
      
      - alert: HighMemoryUsage
        expr: container_memory_usage_bytes / 2Gi > 0.9
        annotations:
          summary: "Memory usage exceeded 90%"
```

---

## 📈 Key Performance Indicators (KPIs)

### Service Availability

- **Target**: 99.9% uptime
- **Metric**: `up` (1 = healthy, 0 = down)
- **Alert**: < 99% for 5 minutes

### Response Time

- **Target**: P95 < 500ms
- **Metric**: `http_request_duration_seconds`
- **Alert**: P95 > 1s

### Error Rate

- **Target**: < 1%
- **Metric**: `rate(http_requests_total{status="5xx"}[5m])`
- **Alert**: > 5%

### Article Generation

- **Target**: 100 articles/hour
- **Metric**: `articles_generated_total`
- **Alert**: < 50 articles/hour

---

## 🔔 Alerting Rules

### Critical Alerts (PagerDuty)

- Database down
- > 10% error rate
- Memory usage > 95%
- Disk space < 10%

### Warning Alerts (Slack)

- Response time > 1s
- Queue depth > 1000
- Low article generation rate

### Info Alerts (Email)

- Deployment completed
- Scheduled backup completed
- Certificate renewal

---

## 🛠️ Debugging

### View Logs

```bash
# Docker
docker-compose logs -f backend

# Kubernetes
kubectl logs -f deployment/hivenews-backend -n hivenews

# Follow specific service
kubectl logs -f deployment/hivenews-backend -n hivenews -c backend
```

### Query Metrics

```bash
# Using curl
curl http://localhost:8080/metrics

# Using PromQL
curl -G 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=rate(http_requests_total[5m])'
```

---

## 📊 Dashboard Examples

### System Health Dashboard

```
CPU Usage:      [Gauge]
Memory Usage:   [Gauge]
Disk I/O:       [Gauge]
Network Traffic: [Graph]
```

### Application Dashboard

```
Request Rate:     [Graph]
Error Rate:       [Graph]
Response Time:    [Graph]
Active Jobs:      [Counter]
```

### Business Dashboard

```
Articles/Hour:    [Graph]
Sources Active:   [Gauge]
Engagement Rate:  [Graph]
Rank Distribution: [Histogram]
```

---

## 📞 Support

For monitoring issues:

- **Documentation**: This file
- **GitHub Issues**: [Repository](#)
- **Email**: ops@hivenews.com

---

**Last Updated:** 2025-10-26

