#!/bin/sh
set -eu

WORKDIR=/app/news-backend

mkdir -p "$WORKDIR/downloads/raw" \
    "$WORKDIR/downloads/cache" \
    "$WORKDIR/downloads/temp" \
    "$WORKDIR/output" \
    "$WORKDIR/logs"

cd "$WORKDIR"

exec /usr/local/bin/news-backend "$@"

