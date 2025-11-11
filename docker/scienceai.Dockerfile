# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /app

COPY apps/frontend-next/ScienceAI/package*.json ./
RUN npm ci


FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app

ENV NODE_ENV=production

COPY --from=deps /app/node_modules ./node_modules
COPY apps/frontend-next/ScienceAI ./

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


