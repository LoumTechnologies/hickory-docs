# Multi-stage build: web dist + server binary → slim runtime image.
# The server serves the web app as a static SPA fallback (WEB_DIST_DIR).

# --- web ---
FROM node:22-alpine AS web
WORKDIR /src
COPY apps/web/package.json apps/web/package-lock.json ./
RUN npm ci
COPY apps/web/ ./
# This image IS the hosted workspace: it has accounts and Stripe, so the client
# offers them. Every other build of the same bundle — the static marketing site,
# and the client `hickory serve` hands to collaborators — defaults to neither.
# See docs/specs/freeform/local-first.md.
ENV VITE_HOSTED=1
RUN npm run build

# --- server ---
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock plans.json ./
COPY crates/ crates/
COPY apps/server/ apps/server/
# The relay is a workspace member, so cargo needs its manifest present even to
# build a different package — without it the build dies on "failed to load
# manifest for workspace member". Its source comes along because a manifest
# pointing at absent code fails just as hard; the relay's own image is built
# from apps/relay/Dockerfile and this copy costs a few kilobytes.
COPY apps/relay/ apps/relay/
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
