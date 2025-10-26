#!/bin/bash
# Hive-News Restore Script
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

echo -e "${RED}========================================${NC}"
echo -e "${RED}Hive-News Restore Script${NC}"
echo -e "${RED}WARNING: This will overwrite existing data!${NC}"
echo -e "${RED}========================================${NC}"

# Function to list available backups
list_backups() {
    echo -e "${YELLOW}Available backups:${NC}"
    
    echo -e "${GREEN}PostgreSQL backups:${NC}"
    ls -lh "$BACKUP_DIR"/*.sql.gz 2>/dev/null || echo "No PostgreSQL backups found"
    
    echo -e "${GREEN}Redis backups:${NC}"
    ls -lh "$BACKUP_DIR"/*.rdb 2>/dev/null || echo "No Redis backups found"
    
    echo -e "${GREEN}Configuration backups:${NC}"
    ls -lh "$BACKUP_DIR"/configs_*.tar.gz 2>/dev/null || echo "No config backups found"
}

# Function to restore PostgreSQL
restore_postgres() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        echo -e "${RED}Backup file not found: $backup_file${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Restoring PostgreSQL from: $backup_file${NC}"
    
    read -p "Are you sure you want to restore? This will overwrite existing data (yes/no): " confirmation
    
    if [ "$confirmation" != "yes" ]; then
        echo -e "${YELLOW}Restore cancelled${NC}"
        exit 0
    fi
    
    # Drop and recreate database (optional - adjust based on your needs)
    # docker exec -i hivenews-postgres psql -U hivenews -c "DROP DATABASE IF EXISTS hivenews;"
    # docker exec -i hivenews-postgres psql -U hivenews -c "CREATE DATABASE hivenews;"
    
    # Restore
    gunzip -c "$backup_file" | docker exec -i hivenews-postgres psql -U hivenews -d hivenews
    
    echo -e "${GREEN}PostgreSQL restored successfully${NC}"
}

# Function to restore Redis
restore_redis() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        echo -e "${RED}Backup file not found: $backup_file${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Restoring Redis from: $backup_file${NC}"
    
    read -p "Are you sure you want to restore? This will overwrite existing data (yes/no): " confirmation
    
    if [ "$confirmation" != "yes" ]; then
        echo -e "${YELLOW}Restore cancelled${NC}"
        exit 0
    fi
    
    # Copy backup to container
    docker cp "$backup_file" hivenews-redis:/data/dump.rdb
    
    echo -e "${GREEN}Redis restored successfully. Restart Redis container to apply changes.${NC}"
}

# Function to restore configuration
restore_configs() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        echo -e "${RED}Backup file not found: $backup_file${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Restoring configuration from: $backup_file${NC}"
    
    read -p "Are you sure you want to restore? This will overwrite existing configs (yes/no): " confirmation
    
    if [ "$confirmation" != "yes" ]; then
        echo -e "${YELLOW}Restore cancelled${NC}"
        exit 0
    fi
    
    # Extract to temporary directory first
    TEMP_DIR=$(mktemp -d)
    tar -xzf "$backup_file" -C "$TEMP_DIR"
    
    echo -e "${GREEN}Configuration files extracted to: $TEMP_DIR${NC}"
    echo -e "${YELLOW}Please manually copy files from $TEMP_DIR to the appropriate locations${NC}"
}

# Main function
main() {
    echo -e "${YELLOW}Select restore option:${NC}"
    echo "1) List available backups"
    echo "2) Restore PostgreSQL"
    echo "3) Restore Redis"
    echo "4) Restore Configuration"
    echo "5) Exit"
    
    read -p "Enter choice [1-5]: " choice
    
    case $choice in
        1)
            list_backups
            ;;
        2)
            read -p "Enter PostgreSQL backup file path: " backup_file
            restore_postgres "$backup_file"
            ;;
        3)
            read -p "Enter Redis backup file path: " backup_file
            restore_redis "$backup_file"
            ;;
        4)
            read -p "Enter Configuration backup file path: " backup_file
            restore_configs "$backup_file"
            ;;
        5)
            echo "Exiting..."
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            exit 1
            ;;
    esac
}

# Run main function
main

