# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION} AS deps
WORKDIR /app

ARG APP_DIR
ARG NPM_CI_FLAGS=""
COPY ${APP_DIR}/package*.json ./
# Configurar npm para retry e timeout aumentado para lidar com problemas de rede
RUN npm config set fetch-retries 5 && \
    npm config set fetch-retry-mintimeout 20000 && \
    npm config set fetch-retry-maxtimeout 120000 && \
    npm config set fetch-timeout 300000 && \
    (npm ci ${NPM_CI_FLAGS} || (sleep 10 && npm ci ${NPM_CI_FLAGS}) || (sleep 20 && npm ci ${NPM_CI_FLAGS})) \
    && (npm install --no-save @rollup/rollup-linux-x64-gnu \
        || npm install --no-save @rollup/rollup-linux-x64-musl \
        || true) \
    && (npm install --no-save @swc/core-linux-x64-gnu \
        || npm install --no-save @swc/core-linux-x64-musl \
        || true)


FROM node:${NODE_VERSION} AS build
WORKDIR /app

ARG APP_DIR
ENV NODE_ENV=production

COPY --from=deps /app/node_modules ./node_modules
COPY ${APP_DIR} ./

RUN npm run build


FROM node:${NODE_VERSION} AS runtime
WORKDIR /app

ENV NODE_ENV=production \
    PORT=80

COPY --from=build /app/public ./public
COPY --from=build /app/.next/static ./.next/static
COPY --from=build /app/.next/standalone ./

EXPOSE 80
CMD ["node", "server.js"]


