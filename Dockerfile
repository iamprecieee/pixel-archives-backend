FROM rust:1.91-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency caching
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies only
RUN cargo build --release || true
RUN rm -rf target/release/.fingerprint/pixel-* \
    target/release/deps/pixel* \
    target/release/deps/libpixel*

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pixel_archives /app/pixel_archives

ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

EXPOSE 8080

ENTRYPOINT ["/app/pixel_archives"]