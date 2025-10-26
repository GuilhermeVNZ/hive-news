# Hive-News Backup & Disaster Recovery Guide

**Version:** 1.0.0  
**Last Updated:** 2025-10-26

---

## 📋 Table of Contents

- [Overview](#overview)
- [Backup Strategy](#backup-strategy)
- [Backup Procedures](#backup-procedures)
- [Recovery Procedures](#recovery-procedures)
- [Disaster Recovery Plan](#disaster-recovery-plan)
- [Testing Backups](#testing-backups)
- [Best Practices](#best-practices)

---

## Overview

This guide covers backup and disaster recovery procedures for Hive-News production deployments.

### Components to Backup

1. **PostgreSQL Database** - Article data, sources, metrics
2. **Redis Cache** - Session data, temporary content
3. **Configuration Files** - Portal profiles, settings
4. **MinIO Storage** - Generated images, documents

---

## Backup Strategy

### Backup Frequency

| Component | Frequency | Retention |
|-----------|-----------|-----------|
| PostgreSQL | Daily | 30 days |
| Redis | Daily | 7 days |
| Configuration | Weekly | 90 days |
| MinIO | Daily | 30 days |

### Backup Storage

- **Local**: Primary backup location
- **Remote**: Cloud storage (S3-compatible)
- **Cold Storage**: Monthly archives

---

## Backup Procedures

### Automated Backups

#### Using cron (Linux)

```bash
# Add to crontab
0 2 * * * /path/to/scripts/backup.sh
```

#### Using Scheduled Tasks (Windows)

```powershell
# Create scheduled task
$action = New-ScheduledTaskAction -Execute "C:\path\to\scripts\backup.ps1"
$trigger = New-ScheduledTaskTrigger -Daily -At 2am
Register-ScheduledTask -TaskName "HiveNewsBackup" -Action $action -Trigger $trigger
```

### Manual Backups

#### PostgreSQL Backup

```bash
# Using Docker
docker exec hivenews-postgres pg_dump -U hivenews hivenews > backup_$(date +%Y%m%d).sql

# Direct connection
pg_dump -h localhost -U hivenews hivenews > backup_$(date +%Y%m%d).sql
```

#### Redis Backup

```bash
# Using Docker
docker exec hivenews-redis redis-cli SAVE
docker cp hivenews-redis:/data/dump.rdb ./redis_backup_$(date +%Y%m%d).rdb

# Direct connection
redis-cli SAVE
cp /var/lib/redis/dump.rdb ./redis_backup_$(date +%Y%m%d).rdb
```

#### Configuration Backup

```bash
# Backup configuration files
tar -czf configs_backup_$(date +%Y%m%d).tar.gz configs/ k8s/ docker/ .env
```

---

## Recovery Procedures

### PostgreSQL Recovery

```bash
# Stop application
docker-compose stop backend

# Drop existing database (if needed)
docker exec -i hivenews-postgres psql -U hivenews -c "DROP DATABASE hivenews;"

# Recreate database
docker exec -i hivenews-postgres psql -U hivenews -c "CREATE DATABASE hivenews;"

# Restore from backup
gunzip -c backup_20241026.sql.gz | docker exec -i hivenews-postgres psql -U hivenews -d hivenews

# Start application
docker-compose start backend
```

### Redis Recovery

```bash
# Stop Redis
docker-compose stop redis

# Copy backup file
docker cp redis_backup_20241026.rdb hivenews-redis:/data/dump.rdb

# Start Redis
docker-compose start redis
```

### Full System Recovery

```bash
# 1. Stop all services
docker-compose down

# 2. Restore PostgreSQL
gunzip -c backups/postgres_20241026.sql.gz | docker exec -i hivenews-postgres psql -U hivenews -d hivenews

# 3. Restore Redis
docker cp backups/redis_20241026.rdb hivenews-redis:/data/dump.rdb

# 4. Restore configuration
tar -xzf backups/configs_20241026.tar.gz

# 5. Start services
docker-compose up -d
```

---

## Disaster Recovery Plan

### Recovery Time Objectives (RTO)

| Priority | Component | RTO | RPO |
|----------|-----------|-----|-----|
| P0 | PostgreSQL | 1 hour | 1 day |
| P1 | Redis | 4 hours | 1 day |
| P2 | Configuration | 8 hours | 1 week |

### Recovery Point Objectives (RPO)

| Component | RPO | Backup Frequency |
|-----------|-----|------------------|
| PostgreSQL | 24 hours | Daily |
| Redis | 24 hours | Daily |
| Configuration | 7 days | Weekly |

### Disaster Scenarios

#### Scenario 1: Database Corruption

1. Stop services
2. Restore from latest backup
3. Verify data integrity
4. Resume services

#### Scenario 2: Server Failure

1. Provision new server
2. Install Docker
3. Restore backups
4. Configure networking
5. Start services

#### Scenario 3: Ransomware Attack

1. Isolate affected systems
2. Restore from clean backups
3. Verify no malware in backups
4. Update security measures
5. Resume operations

---

## Testing Backups

### Monthly Backup Test Procedure

```bash
# 1. Create test environment
docker-compose -f docker-compose.test.yml up -d

# 2. Restore latest backup
./scripts/restore.sh

# 3. Verify data integrity
docker exec hivenews-postgres psql -U hivenews -d hivenews -c "SELECT COUNT(*) FROM articles;"

# 4. Test application functionality
curl http://localhost:3000/health

# 5. Cleanup
docker-compose -f docker-compose.test.yml down -v
```

### Backup Verification Checklist

- [ ] PostgreSQL backup restores successfully
- [ ] Redis backup restores successfully
- [ ] Configuration files are valid
- [ ] Application starts correctly
- [ ] Data integrity verified
- [ ] Performance acceptable

---

## Best Practices

### Backup Best Practices

1. **Automate Everything**: Use cron/Task Scheduler
2. **Verify Backups**: Test restore monthly
3. **Offsite Storage**: Keep copies in different locations
4. **Encryption**: Encrypt sensitive backups
5. **Documentation**: Document all procedures
6. **Monitoring**: Alert on backup failures

### Recovery Best Practices

1. **Practice Regularly**: Test restore procedures
2. **Document Changes**: Update procedures when configurations change
3. **Multiple Copies**: Keep backups in multiple locations
4. **Version Control**: Track backup versions
5. **Quick Access**: Ensure backups are accessible quickly
6. **Team Training**: Train team on recovery procedures

---

## Monitoring

### Backup Monitoring

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'backup_monitor'
    static_configs:
      - targets: ['localhost:9091']  # backup-exporter
```

### Alerts

- Backup failed
- Backup older than 2 days
- Disk space below 20%
- Backup restore failed

---

## Support

For backup/recovery issues:

- **Documentation**: This guide
- **Scripts**: `scripts/backup.sh` and `scripts/restore.sh`
- **Email**: ops@hivenews.com

---

**Last Updated:** 2025-10-26  
**Maintained by:** Hive-News Team

