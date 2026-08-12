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
# ca-certificates for the one outbound call (GitHub, at sign-in), and the
# sqlite3 CLI because the accounts table is the only thing here worth backing
# up — and `.backup` against a live database is the only correct way to do it.
# Copying the file out from under an open connection can catch a half-written
# page; a 1.5 MB binary is a cheap way not to find that out during an incident.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/hickory-relay-server /app/hickory-relay-server
ENV PORT=8080
EXPOSE 8080
CMD ["/app/hickory-relay-server"]
