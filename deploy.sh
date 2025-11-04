#!/bin/bash
# Script de deploy automático para Hostinger VPS
# Uso: ./deploy.sh

set -e  # Parar em caso de erro

PROJECT_DIR="/opt/news-system"
BACKEND_DIR="$PROJECT_DIR/news-backend"
FRONTEND_DIR="$PROJECT_DIR/apps/frontend-next/AIResearch"

echo "🚀 Starting deployment..."
echo "====================================="

# Verificar se está no diretório correto
if [ ! -d "$PROJECT_DIR" ]; then
    echo "❌ Error: Project directory not found: $PROJECT_DIR"
    exit 1
fi

cd $PROJECT_DIR

# 1. Parar pipeline temporariamente
echo ""
echo "⏸️  Stopping pipeline service..."
sudo systemctl stop news-pipeline || echo "⚠️  Pipeline service not running"

# 2. Fazer backup rápido do registry antes de atualizar
echo ""
echo "💾 Creating backup..."
BACKUP_DIR="$PROJECT_DIR/backups"
mkdir -p $BACKUP_DIR
if [ -f "$PROJECT_DIR/articles_registry.json" ]; then
    cp "$PROJECT_DIR/articles_registry.json" \
       "$BACKUP_DIR/registry_$(date +%Y%m%d_%H%M%S).json"
    echo "✅ Backup created"
fi

# 3. Pull do Git (se usar Git para deploy)
if [ -d ".git" ]; then
    echo ""
    echo "📥 Pulling latest changes from Git..."
    git pull origin main || git pull origin master || echo "⚠️  Git pull failed or no remote"
else
    echo ""
    echo "⚠️  Not a Git repository, skipping pull"
fi

# 4. Atualizar Backend
echo ""
echo "🔨 Building backend..."
cd $BACKEND_DIR

# Verificar se Cargo.toml existe
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found in $BACKEND_DIR"
    exit 1
fi

# Build em release mode
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Backend build successful"
else
    echo "❌ Backend build failed!"
    exit 1
fi

# 5. Atualizar Frontend
echo ""
echo "🏗️  Building frontend..."
cd $FRONTEND_DIR

# Verificar se package.json existe
if [ ! -f "package.json" ]; then
    echo "⚠️  Frontend not found, skipping..."
else
    # Instalar dependências (se necessário)
    if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules/.package-lock.json" ]; then
        echo "📦 Installing npm dependencies..."
        npm install
    fi

    # Build
    npm run build

    if [ $? -eq 0 ]; then
        echo "✅ Frontend build successful"
    else
        echo "❌ Frontend build failed!"
        exit 1
    fi
fi

# 6. Restart Pipeline
echo ""
echo "▶️  Restarting pipeline service..."
sudo systemctl start news-pipeline

# Verificar status
sleep 2
if sudo systemctl is-active --quiet news-pipeline; then
    echo "✅ Pipeline service is running"
else
    echo "⚠️  Pipeline service may not be running, check: sudo systemctl status news-pipeline"
fi

# 7. Reload Nginx (se configurado)
if systemctl is-active --quiet nginx; then
    echo ""
    echo "🔄 Reloading Nginx..."
    sudo systemctl reload nginx
    echo "✅ Nginx reloaded"
fi

# 8. Limpar builds antigos (opcional)
echo ""
echo "🧹 Cleaning old build artifacts..."
cd $BACKEND_DIR
cargo clean --target-dir target 2>/dev/null || true

echo ""
echo "====================================="
echo "✅ Deployment completed successfully!"
echo ""
echo "📊 Next steps:"
echo "   - Check pipeline status: sudo systemctl status news-pipeline"
echo "   - View logs: sudo journalctl -u news-pipeline -f"
echo "   - Verify articles: ls -la $PROJECT_DIR/output/AIResearch/ | head -5"
echo ""





























































