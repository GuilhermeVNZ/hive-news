#!/bin/sh
set -eu

echo "🚀 Starting Dashboard..."

# Inject environment variables into Vite build (replace placeholders in index.html)
if [ -n "${VITE_API_URL:-}" ]; then
  echo "📝 Injecting VITE_API_URL: $VITE_API_URL"
  # Find and replace in all JS files in /usr/share/nginx/html/assets/
  find /usr/share/nginx/html/assets -type f -name "*.js" -exec sed -i \
    "s|http://localhost:3005|${VITE_API_URL}|g" {} \;
fi

echo "✅ Dashboard ready!"

# Start nginx in foreground
exec nginx -g 'daemon off;'

