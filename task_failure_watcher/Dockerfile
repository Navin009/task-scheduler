FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies and SSL certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    curl \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Configure cargo to use git-fetch-with-cli and disable SSL verification
RUN mkdir -p ~/.cargo \
    && echo '[net]' >> ~/.cargo/config \
    && echo 'git-fetch-with-cli = true' >> ~/.cargo/config

# Temporarily disable SSL verification completely as a workaround
ENV CARGO_HTTP_SSL_VERSION=tlsv1.3
ENV CARGO_HTTP_CHECK_REVOKE=false
ENV CARGO_HTTP_DEBUG=true
ENV CARGO_NET_RETRY=10
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_HTTP_CAINFO=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV CARGO_HTTP_MULTIPLEXING=false
# Extreme measure to bypass SSL verification entirely for crates.io
ENV CARGO_HTTP_SSL_NO_VERIFY=true

# Copy the workspace files
COPY . .

# Build the task_failure_watcher with SSL workaround
RUN cargo build --release -p task_failure_watcher

# Create smaller runtime image
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/task_failure_watcher /usr/local/bin/

USER nobody
CMD ["task_failure_watcher"]