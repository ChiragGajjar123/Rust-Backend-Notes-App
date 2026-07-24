# Multi-stage Dockerfile for high-performance Render Web Service deployment

# -------------------------------------------------------------
# Stage 1: Build binary
# -------------------------------------------------------------
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache compiled dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src target/release/deps/notes_backend*

# Copy actual source code and migrations
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build release binary (SQLx offline mode enabled or runtime queries)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# -------------------------------------------------------------
# Stage 2: Minimal Runtime Environment
# -------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (SSL certificates & CA bundle)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/notes_backend /app/notes_backend
COPY --from=builder /app/migrations /app/migrations

# Expose port (Render automatically sets PORT env var)
EXPOSE 10000

ENV RUST_LOG="info,tower_http=info"
ENV SERVER_HOST="0.0.0.0"

CMD ["/app/notes_backend"]
