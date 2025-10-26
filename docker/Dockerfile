FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY apps/backend-cmmv/package*.json ./apps/backend-cmmv/

# Install dependencies
RUN npm ci

# Copy source code
COPY apps/backend-cmmv ./apps/backend-cmmv
COPY contracts ./contracts
COPY tsconfig.json ./

# Build
RUN npm run build

# Production stage
FROM node:20-alpine

WORKDIR /app

# Copy built files
COPY --from=builder /app/apps/backend-cmmv/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/contracts ./contracts
COPY apps/backend-cmmv/.env.example ./.env

EXPOSE 3000

CMD ["node", "dist/main.js"]



