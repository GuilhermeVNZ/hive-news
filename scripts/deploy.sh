#!/bin/bash

# Hive-News Deployment Script

set -e

echo "🚀 Starting Hive-News deployment..."

# Check prerequisites
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found"
    exit 1
fi

if ! command -v kubectl &> /dev/null; then
    echo "⚠️  kubectl not found (skipping K8s deployment)"
fi

# Build Docker image
echo "📦 Building Docker image..."
docker build -t hivenews/backend:latest -f docker/Dockerfile .

# Run tests
echo "🧪 Running tests..."
npm test

# Deploy to Kubernetes (if configured)
if command -v kubectl &> /dev/null && kubectl cluster-info &> /dev/null; then
    echo "☸️  Deploying to Kubernetes..."
    kubectl apply -f k8s/deployment.yaml
    kubectl apply -f k8s/service.yaml
    kubectl rollout status deployment/hive-news-backend
else
    echo "⚠️  Kubernetes not configured, skipping K8s deployment"
fi

echo "✅ Deployment complete!"


