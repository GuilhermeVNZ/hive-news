# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /workspace

# Copy entire repository once so subsequent stages can reuse it without
# depending on host-side paths that may vary.
COPY . .

WORKDIR /workspace/apps/frontend-next/ScienceAI
# Use npm install because this project does not provide a package-lock.json yet.
# When a lockfile is added we can switch back to the faster `npm ci`.
RUN npm install


FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app

ENV NODE_ENV=production \
    NEWS_BASE_DIR=/data

# Bring in the prepared source (including node_modules) from the deps stage.
COPY --from=deps /workspace/apps/frontend-next/ScienceAI ./ 

RUN npm run build


FROM nginx:1.27-alpine AS runtime

# Copy custom nginx config with API proxy
COPY --from=deps /workspace/docker/nginx/scienceai.conf /etc/nginx/conf.d/default.conf

# Copy built app
COPY --from=build /app/dist /usr/share/nginx/html

# Copy entrypoint script for env var injection
COPY --from=deps /workspace/docker/scripts/dashboard-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]