# Dockerfile for PowerGrid Network
FROM rust:1.75

# Set non-interactive mode to avoid prompts during build
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies (minimal set)
RUN apt-get update && \
    apt-get install -y \
        curl \
        wget \
        ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Copy project files
COPY . .

# Install substrate-contracts-node binary
ARG NODE_VERSION=v0.42.0
RUN wget -q --show-progress \
    "https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz" \
    -O substrate-contracts-node-linux.tar.gz && \
    tar -xzf substrate-contracts-node-linux.tar.gz && \
    chmod +x substrate-contracts-node-linux/substrate-contracts-node && \
    mv substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/ && \
    rm -rf substrate-contracts-node-linux.tar.gz substrate-contracts-node-linux/

# Create setup script that will be run at container startup
RUN cat > /workspace/setup-in-container.sh << 'EOF'
#!/bin/bash
set -e

echo "Setting up Rust environment for ink! contracts..."

# Add wasm target and rust-src if not already present
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

if ! rustup component list --installed | grep -q rust-src; then
    echo "Adding rust-src component..."
    rustup component add rust-src --toolchain stable
fi

# Install cargo-contract if not present
if ! command -v cargo-contract &> /dev/null; then
    echo "Installing cargo-contract v5.0.1..."
    cargo install --force --locked cargo-contract --version 5.0.1
fi

echo "Setup complete!"
EOF

RUN chmod +x /workspace/setup-in-container.sh

# Verify substrate-contracts-node installation
RUN substrate-contracts-node --version

# Copy and set up entrypoint
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

# Default command
CMD ["bash"]