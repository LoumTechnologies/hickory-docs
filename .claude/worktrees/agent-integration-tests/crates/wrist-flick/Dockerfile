FROM rust:slim-bookworm AS builder
RUN rustup default nightly
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release -p wrist-flick

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/wrist-flick /usr/local/bin/wrist-flick
ENTRYPOINT ["wrist-flick"]
