#!/bin/bash
# Hive-News Production Deployment Script
# Version: 1.0.0
# Date: 2025-10-26

set -e

echo "🚀 Hive-News Production Deployment"
echo "=================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Step 1: Verify Prerequisites
echo -e "${YELLOW}Step 1: Verifying prerequisites...${NC}"

# Check Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js not found${NC}"
    exit 1
fi

NODE_VERSION=$(node --version | sed 's/v//')
echo -e "${GREEN}✅ Node.js version: $NODE_VERSION${NC}"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker found${NC}"

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: Docker Compose not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker Compose found${NC}"

# Step 2: Environment Setup
echo -e "${YELLOW}Step 2: Setting up environment...${NC}"

if [ ! -f .env ]; then
    echo "Creating .env from template..."
    cp env.template .env
    echo -e "${GREEN}✅ Created .env file${NC}"
    echo -e "${YELLOW}⚠️  Please edit .env with your production values${NC}"
    exit 0
else
    echo -e "${GREEN}✅ .env file exists${NC}"
fi

# Step 3: Install Dependencies
echo -e "${YELLOW}Step 3: Installing dependencies...${NC}"

npm install

echo -e "${GREEN}✅ Dependencies installed${NC}"

# Step 4: Run Tests
echo -e "${YELLOW}Step 4: Running tests...${NC}"

npm test

echo -e "${GREEN}✅ All tests passing${NC}"

# Step 5: Build Application
echo -e "${YELLOW}Step 5: Building application...${NC}"

npm run build

echo -e "${GREEN}✅ Application built${NC}"

# Step 6: Start Infrastructure Services
echo -e "${YELLOW}Step 6: Starting infrastructure services...${NC}"

docker-compose up -d postgres redis minio

echo -e "${GREEN}✅ Infrastructure services started${NC}"
echo "Waiting for services to be ready..."

sleep 5

# Step 7: Verify Services
echo -e "${YELLOW}Step 7: Verifying services...${NC}"

docker-compose ps

echo -e "${GREEN}✅ Services verification complete${NC}"

# Step 8: Start Application
echo -e "${YELLOW}Step 8: Starting application...${NC}"

echo -e "${GREEN}Starting backend...${NC}"
npm start &

# Wait for app to start
sleep 10

# Step 9: Health Check
echo -e "${YELLOW}Step 9: Performing health check...${NC}"

if curl -f http://localhost:3000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    exit 1
fi

# Step 10: Display URLs
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Services available at:"
echo "  - Backend API: http://localhost:3000"
echo "  - PostgreSQL: localhost:5432"
echo "  - Redis: localhost:6379"
echo "  - MinIO: http://localhost:9001"
echo ""
echo "API Endpoints:"
echo "  - Health: http://localhost:3000/health"
echo "  - Metrics: http://localhost:3000/metrics"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Configure external APIs (DeepSeek, Vectorizer, etc.)"
echo "  2. Set up monitoring (Prometheus/Grafana)"
echo "  3. Configure backup schedule"
echo "  4. Review logs: docker-compose logs -f"
echo ""

