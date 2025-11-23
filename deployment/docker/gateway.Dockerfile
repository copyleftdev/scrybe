# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies (cached layer)
RUN cargo build --release --bin scrybe-gateway

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 scrybe

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/scrybe-gateway /app/scrybe-gateway

# Set ownership
RUN chown -R scrybe:scrybe /app

USER scrybe

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/scrybe-gateway"]
