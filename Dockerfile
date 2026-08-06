# Multi-stage build: web dist + server binary → slim runtime image.
# The server serves the web app as a static SPA fallback (WEB_DIST_DIR).

# --- web ---
FROM node:22-alpine AS web
WORKDIR /src
COPY apps/web/package.json apps/web/package-lock.json ./
RUN npm ci
COPY apps/web/ ./
RUN npm run build

# --- server ---
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock plans.json ./
COPY crates/ crates/
COPY apps/server/ apps/server/
# The workspace lists apps/server as its only app member needed here; prune
# nothing — a full copy keeps the build simple and correct.
RUN cargo build --release -p hickory-server

# --- runtime ---
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/hickory-server /app/hickory-server
COPY --from=web /src/dist /app/web-dist
ENV WEB_DIST_DIR=/app/web-dist \
    GIT_DATA_DIR=/data/git \
    PORT=8080
EXPOSE 8080
CMD ["/app/hickory-server"]
