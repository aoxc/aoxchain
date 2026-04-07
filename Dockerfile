# --- Stage 1: Builder ---
# Using the latest stable Rust version as of 2026
FROM rust:1.92-bookworm AS builder

# Set the working directory inside the container
WORKDIR /src/aoxchain

# Install necessary build-time dependencies
# pkg-config and libssl-dev are common for Rust projects interacting with OpenSSL
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the manifest files to leverage Docker's layer caching
# This prevents re-downloading dependencies if only source code changes
COPY Cargo.toml Cargo.lock ./

# Copy the entire source tree
COPY . .

# Build the specific binary in release mode
# --locked ensures the build uses the exact versions in Cargo.lock
RUN cargo build --release --locked -p aoxcmd --bin aoxc

# --- Stage 2: Runtime ---
# Using debian-slim for a minimal attack surface and smaller image size
FROM debian:bookworm-slim

# Standard OCI labels for better maintainability and CI/CD integration
LABEL org.opencontainers.image.title="AoxChain Node" \
      org.opencontainers.image.description="Official production image for AoxChain node" \
      org.opencontainers.image.vendor="AoxChain Foundation" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

# Install essential runtime libraries and security updates
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create a dedicated system user for the service (Security Best Practice)
# Avoids running the application as root
RUN groupadd -r aoxgroup && \
    useradd -r -g aoxgroup -d /var/lib/aoxchain -s /sbin/nologin aoxchain

# Create data directory with correct permissions
RUN mkdir -p /var/lib/aoxchain && chown aoxchain:aoxgroup /var/lib/aoxchain

# Set the working directory
WORKDIR /var/lib/aoxchain

# Copy the compiled binary from the builder stage
COPY --from=builder /src/aoxchain/target/release/aoxc /usr/local/bin/aoxc

# Set environment variables
ENV AOXC_HOME=/var/lib/aoxchain \
    RUST_LOG=info

# Expose necessary P2P and RPC ports
# 26656: P2P Network | 8545: JSON-RPC
EXPOSE 26656 8545

# Switch to the non-privileged user
USER aoxchain

# Final execution command
ENTRYPOINT ["aoxc"]
CMD ["node-bootstrap"]
