FROM node:24-alpine AS build
WORKDIR /src
COPY package.json package-lock.json ./
# Install build dependencies only for npm install, then remove them
RUN apk add --no-cache --virtual .build-deps python3 make g++ py3-setuptools \
    && npm ci --legacy-peer-deps \
    && rm -rf node_modules/@opentelemetry/context-zone/node_modules/zone.js \
    && apk del .build-deps
COPY . .
RUN npm run build -- --configuration production

FROM nginx:1.27.4-alpine as final
# Remove default nginx config and static files
RUN rm -rf /usr/share/nginx/html/* \
    && rm /etc/nginx/conf.d/default.conf

COPY --from=build /src/dist/ /usr/share/nginx/html
COPY default.conf.template /etc/nginx/templates/default.conf.template

# Ensure the nginx user owns the necessary directories
RUN touch /var/run/nginx.pid \
    && chown -R nginx:nginx /var/run/nginx.pid /usr/share/nginx/html /var/cache/nginx /var/log/nginx /etc/nginx/conf.d

USER nginx
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://localhost:8080/ || exit 1
