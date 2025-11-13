#!/bin/sh
set -eu

BASE_DIR="${NEWS_BASE_DIR:-/app/news-backend}"

mkdir -p "$BASE_DIR/downloads/raw" \
    "$BASE_DIR/downloads/cache" \
    "$BASE_DIR/downloads/temp" \
    "$BASE_DIR/output" \
    "$BASE_DIR/logs"

cd "$BASE_DIR"

exec /usr/local/bin/news-backend "$@"

