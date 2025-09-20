FROM rust:1.75

# Set non-interactive mode to avoid prompts during build
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
        curl \
        wget \
        git \
        clang \
        pkg-config \
        libssl-dev \
        ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Add wasm target and rust-src
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src --toolchain stable

# Install cargo-contract v5.0.1
RUN cargo install --force --locked cargo-contract --version 5.0.1

# Install substrate-contracts-node (binary)
ARG NODE_VERSION=v0.42.0
RUN wget -q --show-progress \
    "https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz" \
    -O substrate-contracts-node-linux.tar.gz && \
    tar -xzf substrate-contracts-node-linux.tar.gz && \
    chmod +x substrate-contracts-node-linux/substrate-contracts-node && \
    mv substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/ && \
    rm -rf substrate-contracts-node-linux.tar.gz substrate-contracts-node-linux/

# Set working directory
WORKDIR /workspace

# Copy the entire project
COPY . .

# Build all contracts
RUN ./scripts/build-all.sh

# Verify installations
RUN cargo contract --version && \
    substrate-contracts-node --version && \
    rustc --version && \
    cargo --version

# Default command
CMD ["bash"]