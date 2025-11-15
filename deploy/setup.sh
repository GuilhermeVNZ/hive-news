#!/bin/bash
# ====================================
# News Backend - Production Setup Script
# ====================================
# This script prepares the backend for production deployment
# Run this ONCE on the production server after uploading files

set -e  # Exit on any error

echo "========================================"
echo "News Backend - Production Setup"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${RED}ERROR: Do not run this script as root!${NC}"
    exit 1
fi

# Step 1: Check prerequisites
echo "Step 1: Checking prerequisites..."
echo ""

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}ERROR: Rust is not installed!${NC}"
    echo "Install Rust from: https://rustup.rs/"
    echo "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo -e "${GREEN}✓ Rust installed:${NC} $(rustc --version)"

# Check Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: Cargo is not installed!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Cargo installed:${NC} $(cargo --version)"

# Check Node.js (for Playwright)
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}WARNING: Node.js not found - Playwright may not work${NC}"
else
    echo -e "${GREEN}✓ Node.js installed:${NC} $(node --version)"
fi

# Check npm
if ! command -v npm &> /dev/null; then
    echo -e "${YELLOW}WARNING: npm not found - Playwright may not work${NC}"
else
    echo -e "${GREEN}✓ npm installed:${NC} $(npm --version)"
fi

echo ""

# Step 2: Setup directories
echo "Step 2: Creating necessary directories..."
echo ""

mkdir -p data
mkdir -p downloads/cache
mkdir -p downloads/temp
mkdir -p output/ScienceAI
mkdir -p output/AIResearch
mkdir -p logs

echo -e "${GREEN}✓ Directories created${NC}"
echo ""

# Step 3: Check .env file
echo "Step 3: Checking environment configuration..."
echo ""

if [ ! -f ".env" ]; then
    echo -e "${RED}ERROR: .env file not found!${NC}"
    echo ""
    echo "Create a .env file with the following required variables:"
    echo ""
    echo "# REQUIRED:"
    echo "JWT_SECRET=<generate-with: openssl rand -base64 32>"
    echo "DEFAULT_ADMIN_PASSWORD=<strong-password-min-16-chars>"
    echo ""
    echo "# OPTIONAL API KEYS:"
    echo "NATURE_API_KEY="
    echo "SCIENCE_API_KEY="
    echo "# ... etc"
    echo ""
    echo "See .env.example for full template"
    exit 1
fi

echo -e "${GREEN}✓ .env file found${NC}"

# Verify critical environment variables
source .env

if [ -z "$JWT_SECRET" ]; then
    echo -e "${RED}ERROR: JWT_SECRET is not set in .env!${NC}"
    echo "Generate with: openssl rand -base64 32"
    exit 1
fi

if [ ${#JWT_SECRET} -lt 32 ]; then
    echo -e "${RED}ERROR: JWT_SECRET is too short (minimum 32 characters)!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ JWT_SECRET is set${NC}"

if [ -z "$DEFAULT_ADMIN_PASSWORD" ]; then
    echo -e "${RED}ERROR: DEFAULT_ADMIN_PASSWORD is not set in .env!${NC}"
    exit 1
fi

if [ ${#DEFAULT_ADMIN_PASSWORD} -lt 16 ]; then
    echo -e "${RED}ERROR: DEFAULT_ADMIN_PASSWORD is too short (minimum 16 characters)!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ DEFAULT_ADMIN_PASSWORD is set${NC}"

echo ""

# Step 4: Install Playwright (if Node.js available)
if command -v npm &> /dev/null; then
    echo "Step 4: Installing Playwright..."
    echo ""
    
    cd news-backend
    npm install
    npx playwright install chromium
    cd ..
    
    echo -e "${GREEN}✓ Playwright installed${NC}"
    echo ""
else
    echo -e "${YELLOW}⚠️  Skipping Playwright installation (Node.js not available)${NC}"
    echo ""
fi

# Step 5: Build backend (release mode)
echo "Step 5: Building backend in release mode..."
echo "This may take 5-10 minutes on first run..."
echo ""

cd news-backend
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Backend built successfully${NC}"
else
    echo -e "${RED}ERROR: Backend build failed!${NC}"
    exit 1
fi
cd ..

echo ""

# Step 6: Setup systemd service (if systemd available)
if command -v systemctl &> /dev/null; then
    echo "Step 6: Setting up systemd service..."
    echo ""
    
    read -p "Do you want to create a systemd service? (y/n) " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        CURRENT_DIR=$(pwd)
        SERVICE_FILE="/etc/systemd/system/news-backend.service"
        
        echo "[Unit]
Description=News Backend Service
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$CURRENT_DIR/news-backend
Environment=\"RUST_LOG=info\"
ExecStart=$CURRENT_DIR/news-backend/target/release/news-backend servers
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target" | sudo tee $SERVICE_FILE > /dev/null
        
        sudo systemctl daemon-reload
        sudo systemctl enable news-backend
        
        echo -e "${GREEN}✓ Systemd service created${NC}"
        echo "Start service with: sudo systemctl start news-backend"
        echo "Check status with: sudo systemctl status news-backend"
    fi
else
    echo "Step 6: Systemd not available, skipping service setup"
fi

echo ""
echo "========================================"
echo "Setup Complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Start the backend:"
echo "   cd news-backend"
echo "   ./target/release/news-backend servers"
echo ""
echo "   OR (if systemd service created):"
echo "   sudo systemctl start news-backend"
echo ""
echo "2. Test the backend:"
echo "   curl http://localhost:3000/api/health"
echo ""
echo "3. Login to dashboard:"
echo "   http://your-domain.com/dashboard"
echo "   Username: admin"
echo "   Password: <DEFAULT_ADMIN_PASSWORD from .env>"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT: Change admin password immediately after first login!${NC}"
echo ""
echo "4. Setup automated collection (see deploy/scheduler-setup.md)"
echo ""
echo "For troubleshooting, check: logs/backend.log"
echo ""



























