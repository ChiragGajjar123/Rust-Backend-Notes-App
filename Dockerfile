# Multi-stage Dockerfile for high-performance AWS Web Service deployment

# -------------------------------------------------------------
# Stage 1: Build binary
# -------------------------------------------------------------
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy source code and manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build release binary and clean intermediate build target to save disk space
RUN cargo build --release && \
    cp target/release/notes_backend /notes_backend_bin && \
    rm -rf /app/target

# -------------------------------------------------------------
# Stage 2: Minimal Runtime Environment
# -------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (SSL certificates & CA bundle)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy compiled binary and migrations from builder stage
COPY --from=builder /notes_backend_bin /app/notes_backend
COPY --from=builder /app/migrations /app/migrations

# Expose HTTP port
EXPOSE 8080

ENV RUST_LOG="info,tower_http=info"
ENV SERVER_HOST="0.0.0.0"

CMD ["/app/notes_backend"]
