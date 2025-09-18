# Multi-stage build for production-ready Rust application
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY intelligence-core/ ./intelligence-core/
COPY intelligence-security/ ./intelligence-security/
COPY intelligence-analytics/ ./intelligence-analytics/
COPY intelligence-monitoring/ ./intelligence-monitoring/
COPY intelligence-api/ ./intelligence-api/

# Build the application
RUN cargo build --release --bin intelligence-api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r intelligence && useradd -r -g intelligence intelligence

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/intelligence-api /app/intelligence-api

# Copy configuration files
COPY --from=builder /app/intelligence-api/config/ ./config/

# Create directories for models and data
RUN mkdir -p /models /data /tmp && \
    chown -R intelligence:intelligence /app /models /data /tmp

# Switch to non-root user
USER intelligence

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
CMD ["./intelligence-api"]
