# Hive-News Backup Script (PowerShell)
# Version: 1.0.0
# Last Updated: 2025-10-26

param(
    [string]$BackupDir = "./backups",
    [int]$RetentionDays = 7
)

# Configuration
$ErrorActionPreference = "Stop"
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupPath = Join-Path $BackupDir "backup_$timestamp"

# Create backup directory
New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
New-Item -ItemType Directory -Path $backupPath -Force | Out-Null

Write-Host "Starting Hive-News backup process..." -ForegroundColor Green

# Function to backup PostgreSQL
function Backup-PostgreSQL {
    Write-Host "Backing up PostgreSQL..." -ForegroundColor Yellow
    
    $pgBackupFile = Join-Path $backupPath "postgres_$timestamp.sql.gz"
    
    # Try Docker first
    $dockerRunning = docker ps 2>&1 | Select-String -Pattern "hivenews-postgres"
    
    if ($dockerRunning) {
        # Docker environment
        docker exec hivenews-postgres pg_dump -U hivenews hivenews | Out-File -FilePath "$backupPath/postgres_$timestamp.sql"
        Write-Host "PostgreSQL backup completed: postgres_$timestamp.sql" -ForegroundColor Green
    } else {
        # Direct connection
        Write-Host "Direct PostgreSQL backup not implemented. Please use Docker or manual backup." -ForegroundColor Red
    }
}

# Function to backup Redis
function Backup-Redis {
    Write-Host "Backing up Redis..." -ForegroundColor Yellow
    
    $dockerRunning = docker ps 2>&1 | Select-String -Pattern "hivenews-redis"
    
    if ($dockerRunning) {
        # Save Redis data
        docker exec hivenews-redis redis-cli SAVE
        
        # Copy dump file
        $dumpFile = Join-Path $backupPath "redis_$timestamp.rdb"
        docker cp hivenews-redis:/data/dump.rdb $dumpFile
        
        Write-Host "Redis backup completed: redis_$timestamp.rdb" -ForegroundColor Green
    } else {
        Write-Host "Redis backup not available (Docker not running)" -ForegroundColor Yellow
    }
}

# Function to backup configuration files
function Backup-Configuration {
    Write-Host "Backing up configuration files..." -ForegroundColor Yellow
    
    $configBackupFile = Join-Path $backupPath "configs_$timestamp.zip"
    
    $filesToBackup = @(
        "configs/",
        "k8s/",
        "docker/",
        ".env"
    )
    
    $existingFiles = $filesToBackup | Where-Object { Test-Path $_ }
    
    if ($existingFiles.Count -gt 0) {
        Compress-Archive -Path $existingFiles -DestinationPath $configBackupFile -Force
        Write-Host "Configuration backup completed: $configBackupFile" -ForegroundColor Green
    } else {
        Write-Host "No configuration files found to backup" -ForegroundColor Yellow
    }
}

# Function to clean old backups
function Cleanup-OldBackups {
    Write-Host "Cleaning up old backups (older than $RetentionDays days)..." -ForegroundColor Yellow
    
    $cutoffDate = (Get-Date).AddDays(-$RetentionDays)
    
    Get-ChildItem -Path $BackupDir -Recurse | Where-Object {
        $_.LastWriteTime -lt $cutoffDate
    } | Remove-Item -Force
    
    Write-Host "Cleanup completed" -ForegroundColor Green
}

# Main backup process
Write-Host "========================================" -ForegroundColor Green
Write-Host "Hive-News Backup Script" -ForegroundColor Green
Write-Host "Time: $timestamp" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

# Run backups
Backup-PostgreSQL
Backup-Redis
Backup-Configuration

# Cleanup old backups
Cleanup-OldBackups

Write-Host "========================================" -ForegroundColor Green
Write-Host "Backup completed successfully!" -ForegroundColor Green
Write-Host "Backup directory: $backupPath" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

# List recent backups
Write-Host "`nRecent backups:" -ForegroundColor Yellow
Get-ChildItem -Path $BackupDir -Directory | Sort-Object LastWriteTime -Descending | Select-Object -First 5 | Format-Table Name, LastWriteTime

