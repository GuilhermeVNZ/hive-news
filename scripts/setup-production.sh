#!/bin/bash
# Script de setup inicial para produção na Hostinger
# Execute: bash scripts/setup-production.sh

set -e

echo "🚀 News System - Production Setup"
echo "====================================="
echo ""

# Verificar se está rodando como root
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  Please run as root or with sudo"
    exit 1
fi

# 1. Atualizar sistema
echo "📦 Step 1: Updating system packages..."
apt update && apt upgrade -y

# 2. Instalar dependências básicas
echo ""
echo "📦 Step 2: Installing dependencies..."
apt install -y \
    build-essential \
    curl \
    git \
    nginx \
    certbot \
    python3-certbot-nginx \
    postgresql \
    postgresql-contrib \
    libpq-dev

# 3. Instalar Rust
echo ""
echo "🦀 Step 3: Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "✅ Rust already installed"
    rustup update
fi

# 4. Instalar Node.js
echo ""
echo "📦 Step 4: Installing Node.js..."
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt install -y nodejs
else
    echo "✅ Node.js already installed"
fi

# 5. Criar diretórios
echo ""
echo "📁 Step 5: Creating directories..."
PROJECT_DIR="/opt/news-system"
mkdir -p $PROJECT_DIR/{downloads/{arxiv,filtered,rejected},output/{AIResearch,Promotional,ScienceAI},logs,backups}

# 6. Configurar permissões
echo ""
echo "🔐 Step 6: Setting permissions..."
chown -R $SUDO_USER:$SUDO_USER $PROJECT_DIR || chown -R root:root $PROJECT_DIR

# 7. Instalar systemd service
echo ""
echo "⚙️  Step 7: Setting up systemd service..."
cat > /etc/systemd/system/news-pipeline.service << EOF
[Unit]
Description=News System Pipeline - Automated Article Collection
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$PROJECT_DIR
Environment="PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
Environment="RUST_LOG=info"
ExecStart=/root/.cargo/bin/cargo run --release --manifest-path $PROJECT_DIR/Cargo.toml --bin start collector
Restart=always
RestartSec=10
StandardOutput=append:$PROJECT_DIR/logs/pipeline.log
StandardError=append:$PROJECT_DIR/logs/pipeline.error.log

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo "✅ Systemd service created (not started yet)"

# 8. Criar script de backup diário
echo ""
echo "💾 Step 8: Setting up backup script..."
cat > $PROJECT_DIR/backup.sh << 'BACKUP_EOF'
#!/bin/bash
BACKUP_DIR="/opt/news-system/backups"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup registry
if [ -f /opt/news-system/articles_registry.json ]; then
    cp /opt/news-system/articles_registry.json $BACKUP_DIR/registry_$DATE.json
    echo "✅ Registry backed up"
fi

# Backup output structure (não incluir PDFs)
if [ -d /opt/news-system/output ]; then
    tar -czf $BACKUP_DIR/output_$DATE.tar.gz /opt/news-system/output/ --exclude='*.pdf' 2>/dev/null || true
    echo "✅ Output structure backed up"
fi

# Manter apenas últimos 7 dias
find $BACKUP_DIR -name "registry_*.json" -mtime +7 -delete
find $BACKUP_DIR -name "output_*.tar.gz" -mtime +7 -delete

echo "✅ Backup completed: $DATE"
BACKUP_EOF

chmod +x $PROJECT_DIR/backup.sh

# 9. Configurar crontab para backup diário
echo ""
echo "⏰ Step 9: Setting up daily backups..."
(crontab -l 2>/dev/null | grep -v "backup.sh"; echo "0 2 * * * $PROJECT_DIR/backup.sh >> $PROJECT_DIR/logs/backup.log 2>&1") | crontab -
echo "✅ Daily backup scheduled at 2 AM"

# 10. Configurar firewall
echo ""
echo "🔥 Step 10: Configuring firewall..."
if command -v ufw &> /dev/null; then
    ufw allow 22/tcp    # SSH
    ufw allow 80/tcp    # HTTP
    ufw allow 443/tcp   # HTTPS
    ufw --force enable
    echo "✅ Firewall configured"
else
    echo "⚠️  UFW not installed, skipping firewall setup"
fi

# 11. Resumo
echo ""
echo "====================================="
echo "✅ Setup completed!"
echo ""
echo "📋 Next steps:"
echo "   1. Clone your repository to $PROJECT_DIR"
echo "   2. Configure .env file in $PROJECT_DIR/news-backend/.env"
echo "   3. Build backend: cd $PROJECT_DIR/news-backend && cargo build --release"
echo "   4. Build frontend: cd $PROJECT_DIR/apps/frontend-next/AIResearch && npm install && npm run build"
echo "   5. Configure Nginx (see docs/DEPLOY_HOSTINGER.md)"
echo "   6. Start pipeline: systemctl start news-pipeline"
echo "   7. Enable pipeline: systemctl enable news-pipeline"
echo ""
echo "📚 Full documentation: docs/DEPLOY_HOSTINGER.md"





























































