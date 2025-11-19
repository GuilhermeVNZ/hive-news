# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /app

COPY apps/frontend-next/scienceai/package.json ./package.json
# Use npm install because this project does not provide a package-lock.json yet.
# When a lockfile is added we can switch back to the faster `npm ci`.
# Configurar npm para retry e timeout aumentado para lidar com problemas de rede
RUN npm config set fetch-retries 5 && \
    npm config set fetch-retry-mintimeout 20000 && \
    npm config set fetch-retry-maxtimeout 120000 && \
    npm config set fetch-timeout 300000 && \
    (npm install || (sleep 10 && npm install) || (sleep 20 && npm install))


FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app

ENV NODE_ENV=production \
    NEWS_BASE_DIR=/data

# Bring in the prepared source (including node_modules) from the deps stage.
COPY --from=deps /app/node_modules ./node_modules
COPY apps/frontend-next/scienceai ./

RUN npm run build


FROM nginx:1.27-alpine AS runtime

# Copy custom nginx config with API proxy
COPY docker/nginx/scienceai.conf /etc/nginx/conf.d/default.conf

# Copy built app
COPY --from=build /app/dist /usr/share/nginx/html

# Copy entrypoint script for env var injection
COPY docker/scripts/dashboard-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]