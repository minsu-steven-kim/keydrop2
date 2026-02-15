# Build stage
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crypto-core ./crypto-core
COPY backend ./backend

# Create stub for desktop workspace member (not needed for backend build)
RUN mkdir -p desktop/src-tauri/src && \
    printf '[package]\nname = "keydrop-desktop"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n' > desktop/src-tauri/Cargo.toml && \
    printf 'fn main() {}\n' > desktop/src-tauri/src/main.rs

# Build release binary
RUN cargo build --release --package keydrop-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/keydrop-backend /usr/local/bin/

# Copy migrations
COPY --from=builder /app/backend/migrations /app/migrations

WORKDIR /app

# Create non-root user for security
RUN useradd -r -s /bin/false keydrop
USER keydrop

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

# Run the binary
CMD ["keydrop-backend"]
