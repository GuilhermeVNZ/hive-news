# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /app

ARG APP_DIR
ARG NPM_CI_FLAGS=""
COPY ${APP_DIR}/package*.json ./
RUN npm ci ${NPM_CI_FLAGS}


FROM node:${NODE_VERSION}-alpine AS build
WORKDIR /app

ARG APP_DIR
ENV NODE_ENV=production

COPY --from=deps /app/node_modules ./node_modules
COPY ${APP_DIR} ./

RUN npm run build


FROM node:${NODE_VERSION}-alpine AS runtime
WORKDIR /app

ENV NODE_ENV=production \
    PORT=80

COPY --from=build /app/public ./public
COPY --from=build /app/.next/static ./.next/static
COPY --from=build /app/.next/standalone ./

EXPOSE 80
CMD ["node", "server.js"]


