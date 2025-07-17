# this_file: Dockerfile
# Multi-stage build for fontgrep

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libfontconfig1 \
    libfreetype6 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/fontgrep /usr/local/bin/fontgrep

# Set up a non-root user
RUN useradd -m -s /bin/bash fontgrep
USER fontgrep

# Set working directory
WORKDIR /fonts

# Default command
ENTRYPOINT ["fontgrep"]
CMD ["--help"]

# Metadata
LABEL \
    org.opencontainers.image.title="fontgrep" \
    org.opencontainers.image.description="A command-line tool for searching and analyzing font files" \
    org.opencontainers.image.url="https://github.com/twardoch/fontgrep" \
    org.opencontainers.image.source="https://github.com/twardoch/fontgrep" \
    org.opencontainers.image.version="0.2.1" \
    org.opencontainers.image.authors="Simon Cozens <simon@simon-cozens.org>, Adam Twardoch <adam+github@twardoch.com>" \
    org.opencontainers.image.licenses="MIT"