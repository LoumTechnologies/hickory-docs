FROM rust:slim-bookworm AS builder
RUN rustup default nightly
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src &amp;&amp; echo "fn main() {}" > src/main.rs &amp;&amp; echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update &amp;&amp; apt-get install -y --no-install-recommends curl &amp;&amp; rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/blue-iron-cube /app/blue-iron-cube
COPY --from=builder /app/migrations /app/migrations
RUN mkdir -p /app/data
EXPOSE 3000
CMD ["/app/blue-iron-cube"]
