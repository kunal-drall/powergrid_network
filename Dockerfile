# syntax=docker/dockerfile:1.4
# PowerGrid Network reproducible dev container
# Works across Linux and macOS hosts (x86_64 / arm64)
FROM --platform=$BUILDPLATFORM ubuntu:22.04

ARG USER=developer
ARG UID=1000
ARG GID=1000
ARG RUST_VERSION=1.86.0
ARG CARGO_CONTRACT_VERSION=5.0.1
ARG NODE_VERSION=v0.42.0

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH \
    CARGO_TARGET_DIR=/tmp/cargo-target

# Install system dependencies (no arch-specific binaries)
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
        binaryen && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal --component rust-src

# Install rust components and tooling using full path to avoid sourcing
RUN /usr/local/cargo/bin/rustup component add clippy --toolchain ${RUST_VERSION} && \
    /usr/local/cargo/bin/rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION}

# Install cargo-contract (ink! tooling)
RUN /usr/local/cargo/bin/cargo install --locked cargo-contract --version ${CARGO_CONTRACT_VERSION}

# Build substrate-contracts-node from source for current architecture
RUN /usr/local/cargo/bin/cargo install contracts-node \
        --git https://github.com/paritytech/substrate-contracts-node.git \
        --tag ${NODE_VERSION} \
        --force

# Create non-root user for interactive workflows
RUN groupadd -g ${GID} ${USER} && \
    useradd -m -u ${UID} -g ${GID} ${USER}

# Ensure developer owns cargo caches and target directory
RUN mkdir -p ${CARGO_TARGET_DIR} && \
    chown -R ${UID}:${GID} /usr/local/cargo /usr/local/rustup ${CARGO_TARGET_DIR}

USER ${USER}
WORKDIR /workspace

# Expose Substrate node ports
EXPOSE 9944 9933 30333

CMD ["bash"]
