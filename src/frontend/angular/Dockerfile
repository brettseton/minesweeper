FROM node:18-alpine AS build
# Install build dependencies for native modules
RUN apk add --no-cache python3 make g++ py3-setuptools
WORKDIR /src
COPY package.json package-lock.json ./
RUN npm install
COPY . .
RUN npm run build

FROM nginx:1.23.1-alpine as final
COPY --from=build /src/dist/ /usr/share/nginx/html
COPY default.conf.template /etc/nginx/templates/default.conf.template
EXPOSE 80
