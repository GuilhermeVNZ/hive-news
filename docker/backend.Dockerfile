# syntax=docker/dockerfile:1.5

FROM rust:1.85-bullseye AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project sources (heavy directories are pruned via .dockerignore)
COPY . ./

# Build the backend binary
RUN cargo build --release --manifest-path news-backend/Cargo.toml --bin servers


FROM mcr.microsoft.com/playwright:v1.47.0-jammy AS runtime

ENV NODE_ENV=production \
    PORT=3005

WORKDIR /app

# Copy backend sources and install Node dependencies for Playwright + sharp
COPY news-backend /app/news-backend
WORKDIR /app/news-backend
RUN npm ci --omit=dev

# Copy compiled binary
COPY --from=builder /app/news-backend/target/release/servers /usr/local/bin/news-backend

# Runtime directories for generated content
RUN mkdir -p downloads/raw downloads/cache downloads/temp output logs js \
    && chmod +x /usr/local/bin/news-backend

# Entrypoint ensures directories exist before booting the service
COPY docker/scripts/backend-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 3005
ENTRYPOINT ["/entrypoint.sh"]
CMD ["servers"]

