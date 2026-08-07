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
RUN cargo build --release -p hickory-server -p hick-lsp

# --- runtime ---
FROM debian:bookworm-slim
# docker-cli (not the daemon): HICKORY_EXECUTOR=docker drives a daemon
# reachable over DOCKER_HOST or a mounted socket. The image deliberately does
# not run a daemon of its own — nested containers need privileges this is not
# guaranteed to have, and a runtime that silently degrades is worse than one
# that fails on the first run with a clear message.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git docker.io \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/hickory-server /app/hickory-server
# The LSP bridge spawns hick-lsp next to the server binary (see src/lsp.rs).
COPY --from=build /src/target/release/hick-lsp /app/hick-lsp
COPY --from=web /src/dist /app/web-dist
ENV WEB_DIST_DIR=/app/web-dist \
    GIT_DATA_DIR=/data/git \
    PORT=8080
EXPOSE 8080
CMD ["/app/hickory-server"]
