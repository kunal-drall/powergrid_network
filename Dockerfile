# PowerGrid Network Docker Container
# Provides cross-platform development environment for ink! smart contracts

# Stage 1: Builder - Install Rust, tools, and build contracts
FROM rust:1.86-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libssl-dev \
    pkg-config \
    curl \
    wget \
    git \
    protobuf-compiler \
    binaryen \
    wabt \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-contract (locked to 5.0.1 for ink! 5.1.1 compatibility)
RUN cargo install --force --locked cargo-contract --version 5.0.1

# Install substrate-contracts-node v0.42.0
RUN wget https://github.com/paritytech/substrate-contracts-node/releases/download/v0.42.0/substrate-contracts-node-linux.tar.gz && \
    tar -xzf substrate-contracts-node-linux.tar.gz && \
    mv substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/ && \
    chmod +x /usr/local/bin/substrate-contracts-node && \
    rm -rf substrate-contracts-node-linux.tar.gz substrate-contracts-node-linux

# Add WASM target
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

# Set working directory
WORKDIR /app

# Copy project files
COPY . /app

# Build all contracts
RUN ./scripts/build-all.sh

# Stage 2: Runtime - Lightweight image for running node and tests
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy substrate-contracts-node from builder
COPY --from=builder /usr/local/bin/substrate-contracts-node /usr/local/bin/

# Copy built artifacts and scripts from builder
COPY --from=builder /app /app

# Set working directory
WORKDIR /app

# Expose ports for node (RPC, WS, P2P)
EXPOSE 9944 9933 30333

# Default command (interactive shell for tester)
CMD ["bash"]