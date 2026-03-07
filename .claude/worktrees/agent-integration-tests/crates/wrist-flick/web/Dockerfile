FROM node:22-slim AS builder
WORKDIR /app
COPY crates/wrist-flick/web/package.json crates/wrist-flick/web/package-lock.json ./
RUN npm ci
COPY crates/wrist-flick/web/ ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY crates/wrist-flick/web/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
