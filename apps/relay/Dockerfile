# The relay: one small binary that forwards bytes.
#
# Deliberately separate from the root Dockerfile. That one builds the hosted
# workspace — a web bundle, a language server, a docker CLI for executing
# documents. The relay needs none of it: it runs no documents, serves no web
# app, and the smaller its image is, the less there is to be wrong about a
# process that sits on the public internet holding other people's traffic.

# --- build ---
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock plans.json ./
COPY crates/ crates/
COPY apps/ apps/
RUN cargo build --release -p hickory-relay-server

# --- runtime ---
FROM debian:bookworm-slim
# ca-certificates only: the relay's one outbound call is to GitHub, at sign-in.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/hickory-relay-server /app/hickory-relay-server
ENV PORT=8080
EXPOSE 8080
CMD ["/app/hickory-relay-server"]
