# PowerGrid Network reproducible dev container
# Ensures Rust 1.86.0, ink! 5.0.1, cargo-contract, substrate-contracts-node, and all build tools
FROM ubuntu:22.04

ARG USER=developer
ARG UID=1000
ARG GID=1000
ARG NODE_VERSION=v0.42.0

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH \
    CARGO_TARGET_DIR=/tmp/cargo-target \
    WASM_BUILD_TOOLCHAIN=1.86.0

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        curl \
        git \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
        ca-certificates \
        wget \
        jq \
        cmake \
        binaryen \
        sudo \
    && rm -rf /var/lib/apt/lists/*

# Use bash for all RUN commands so cargo env is available
SHELL ["/bin/bash", "-lc"]

# Install Rust 1.86.0, rust-src, clippy, wasm target, cargo-contract (ink! 5.0.1)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.86.0 --profile minimal && \
    source /usr/local/cargo/env && \
    rustup component add rust-src --toolchain 1.86.0 && \
    rustup component add clippy --toolchain 1.86.0 && \
    rustup target add wasm32-unknown-unknown --toolchain 1.86.0 && \
    cargo install --locked cargo-contract --version 5.0.1

# Install substrate-contracts-node v0.42.0
RUN wget -q https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz && \
    tar -xzf substrate-contracts-node-linux.tar.gz && \
    install -m 0755 substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/substrate-contracts-node && \
    rm -rf substrate-contracts-node-linux substrate-contracts-node-linux.tar.gz

# Create non-root user for interactive workflows
RUN groupadd -g ${GID} ${USER} && \
    useradd -m -u ${UID} -g ${GID} ${USER}

USER ${USER}
WORKDIR /workspace

# Expose Substrate node ports
EXPOSE 9944 9933 30333

CMD ["bash"]