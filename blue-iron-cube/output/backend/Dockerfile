FROM rust:slim-bookworm AS builder
RUN rustup default nightly
WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build deps
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy real source
COPY src/ src/
COPY migrations/ migrations/

# Touch to invalidate cache on source change
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/blue-iron-cube /app/blue-iron-cube
COPY --from=builder /app/migrations /app/migrations

RUN mkdir -p /app/data

EXPOSE 3000

CMD ["/app/blue-iron-cube"]
