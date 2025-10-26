#!/bin/bash
# Hive-News Backup Script
# Version: 1.0.0
# Last Updated: 2025-10-26

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS="${RETENTION_DAYS:-7}"

# Create backup directory
mkdir -p "$BACKUP_DIR"

echo -e "${GREEN}Starting Hive-News backup process...${NC}"

# Function to backup PostgreSQL
backup_postgres() {
    echo -e "${YELLOW}Backing up PostgreSQL...${NC}"
    
    local pg_backup_file="$BACKUP_DIR/postgres_$TIMESTAMP.sql.gz"
    
    if [ "$DOCKER_ENV" = "true" ]; then
        # Docker environment
        docker exec hivenews-postgres pg_dump -U hivenews hivenews | gzip > "$pg_backup_file"
    else
        # Direct connection
        pg_dump -h localhost -U hivenews hivenews | gzip > "$pg_backup_file"
    fi
    
    echo -e "${GREEN}PostgreSQL backup completed: $pg_backup_file${NC}"
}

# Function to backup Redis
backup_redis() {
    echo -e "${YELLOW}Backing up Redis...${NC}"
    
    local redis_backup_file="$BACKUP_DIR/redis_$TIMESTAMP.rdb"
    
    if [ "$DOCKER_ENV" = "true" ]; then
        # Docker environment
        docker exec hivenews-redis redis-cli SAVE
        docker cp hivenews-redis:/data/dump.rdb "$redis_backup_file"
    else
        # Direct connection
        redis-cli SAVE
        cp /var/lib/redis/dump.rdb "$redis_backup_file"
    fi
    
    echo -e "${GREEN}Redis backup completed: $redis_backup_file${NC}"
}

# Function to backup configuration files
backup_configs() {
    echo -e "${YELLOW}Backing up configuration files...${NC}"
    
    local config_backup_file="$BACKUP_DIR/configs_$TIMESTAMP.tar.gz"
    
    tar -czf "$config_backup_file" \
        configs/ \
        .env \
        k8s/ \
        docker/ \
        2>/dev/null || true
    
    echo -e "${GREEN}Configuration backup completed: $config_backup_file${NC}"
}

# Function to clean old backups
cleanup_old_backups() {
    echo -e "${YELLOW}Cleaning up old backups (older than $RETENTION_DAYS days)...${NC}"
    
    find "$BACKUP_DIR" -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -name "*.rdb" -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -name "configs_*.tar.gz" -mtime +$RETENTION_DAYS -delete
    
    echo -e "${GREEN}Cleanup completed${NC}"
}

# Main backup process
main() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Hive-News Backup Script${NC}"
    echo -e "${GREEN}Time: $TIMESTAMP${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    # Detect environment
    if docker ps | grep -q hivenews-postgres; then
        DOCKER_ENV=true
        echo -e "${YELLOW}Detected Docker environment${NC}"
    else
        DOCKER_ENV=false
        echo -e "${YELLOW}Detected direct environment${NC}"
    fi
    
    # Run backups
    backup_postgres
    backup_redis
    backup_configs
    
    # Cleanup old backups
    cleanup_old_backups
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Backup completed successfully!${NC}"
    echo -e "${GREEN}Backup directory: $BACKUP_DIR${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    # List backups
    echo -e "${YELLOW}Recent backups:${NC}"
    ls -lh "$BACKUP_DIR" | tail -5
}

# Run main function
main

