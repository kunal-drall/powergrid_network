# PowerGrid Network Docker Container
# Provides cross-platform development environment for ink! smart contracts

FROM ubuntu:24.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with latest stable
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install ink! contract tools
RUN cargo install --force --locked \
    cargo-contract@5.0.3 \
    substrate-contracts-node@0.42.0

# Add WebAssembly target
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build all contracts
RUN ./scripts/build-all.sh

# Expose substrate node port
EXPOSE 9944

# Default command
CMD ["bash"]