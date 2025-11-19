# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /app

ARG APP_DIR
COPY ${APP_DIR}/package*.json ./
# Configurar npm para retry e timeout aumentado para lidar com problemas de rede
RUN npm config set fetch-retries 5 && \
    npm config set fetch-retry-mintimeout 20000 && \
    npm config set fetch-retry-maxtimeout 120000 && \
    npm config set fetch-timeout 300000 && \
    npm ci || (sleep 10 && npm ci) || (sleep 20 && npm ci)


FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app

ARG APP_DIR
ENV NODE_ENV=production

COPY --from=deps /app/node_modules ./node_modules
COPY ${APP_DIR} ./

RUN npm run build


FROM nginx:1.27-alpine AS runtime

COPY docker/nginx/spa.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
COPY docker/scripts/dashboard-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]

