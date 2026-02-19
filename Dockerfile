# Multi-stage Docker build for Trello Local
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create dummy src for caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (caching layer)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY backend/src ./src

# Rebuild with actual code
RUN cargo build --release && rm -rf src

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create directories
RUN mkdir -p /app/data /app/frontend

# Copy binary from builder
COPY --from=builder /app/target/release/backend /app/backend

# Copy frontend
COPY frontend /app/frontend

# Create data directory
RUN mkdir -p /app/backend/data

# Set environment variables
ENV DATABASE_PATH=/app/backend/data/trello.db

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/boards || exit 1

# Run the application
CMD ["./backend"]
