# syntax=docker/dockerfile:1.5

ARG NODE_VERSION=20

FROM node:${NODE_VERSION}-alpine AS deps
WORKDIR /app

ARG APP_DIR
COPY ${APP_DIR}/package*.json ./
RUN npm ci


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

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

