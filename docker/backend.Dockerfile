# syntax=docker/dockerfile:1.5

FROM rust:1.90-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project sources (heavy directories are pruned via .dockerignore)
COPY . ./

# Build the backend binary
RUN cargo build --release --manifest-path news-backend/Cargo.toml --bin news-backend

# Build compression-prompt binary
RUN cargo build --release --manifest-path compression-prompt-main/rust/Cargo.toml


FROM mcr.microsoft.com/playwright:v1.56.1-jammy AS runtime

ENV NODE_ENV=production \
    PORT=3005

# Install poppler-utils for pdftotext (PDF text extraction)
RUN apt-get update \
    && apt-get install -y --no-install-recommends poppler-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend sources and install Node dependencies for Playwright + sharp
COPY news-backend /app/news-backend
# Copy compression-prompt for PDF processing
COPY compression-prompt-main /app/compression-prompt-main
WORKDIR /app/news-backend
RUN npm ci --omit=dev

# Copy compiled binaries
COPY --from=builder /app/news-backend/target/release/news-backend /usr/local/bin/news-backend
COPY --from=builder /app/compression-prompt-main/rust/target/release/compress /usr/local/bin/compress

# Runtime directories for generated content
RUN mkdir -p downloads/raw downloads/cache downloads/temp output logs js \
    && chmod +x /usr/local/bin/news-backend \
    && chmod +x /usr/local/bin/compress

# Entrypoint ensures directories exist before booting the service
COPY docker/scripts/backend-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 3005
ENTRYPOINT ["/entrypoint.sh"]
CMD ["servers"]

