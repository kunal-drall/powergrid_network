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

CMD ["bash"]FROM ubuntu:22.04# PowerGrid Network reproducible dev container# PowerGrid Network reproducible dev container# PowerGrid Network reproducible dev container


# Ensures Rust 1.86.0, ink! 5.0.1, cargo-contract, substrate-contracts-node, and all build tools

FROM ubuntu:22.04# Ensures Rust 1.86.0, ink! 5.0.1, cargo-contract, substrate-contracts-node, and all build tools# Ensures Rust 1.86.0, ink! 5.0.1, cargo-contract, substrate-contracts-node, and all build tools



ARG USER=developerFROM ubuntu:22.04

ARG UID=1000

ARG GID=1000FROM ubuntu:22.04

ARG NODE_VERSION=v0.42.0

ARG USER=developer

ENV DEBIAN_FRONTEND=noninteractive \

    RUSTUP_HOME=/usr/local/rustup \ARG USER=developerARG UID=1000

    CARGO_HOME=/usr/local/cargo \

    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH \ARG UID=1000ARG GID=1000

    CARGO_TARGET_DIR=/tmp/cargo-target \

    WASM_BUILD_TOOLCHAIN=1.86.0ARG GID=1000ARG NODE_VERSION=v0.42.0



RUN apt-get update && \ARG NODE_VERSION=v0.42.0

    apt-get install -y --no-install-recommends \

        build-essential \ENV DEBIAN_FRONTEND=noninteractive \

        clang \

        curl \ENV DEBIAN_FRONTEND=noninteractive \    RUSTUP_HOME=/usr/local/rustup \

        git \

        pkg-config \    RUSTUP_HOME=/usr/local/rustup \    CARGO_HOME=/usr/local/cargo \

        libssl-dev \

        protobuf-compiler \    CARGO_HOME=/usr/local/cargo \    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH \

        ca-certificates \

        wget \    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH \    CARGO_TARGET_DIR=/tmp/cargo-target \

        jq \

        cmake \    CARGO_TARGET_DIR=/tmp/cargo-target \    WASM_BUILD_TOOLCHAIN=1.86.0

        binaryen \

        sudo \    WASM_BUILD_TOOLCHAIN=1.86.0

    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && \

# Use bash for all RUN commands so cargo env is available

SHELL ["/bin/bash", "-lc"]RUN apt-get update && \    apt-get install -y --no-install-recommends \



# Install Rust 1.86.0, rust-src, clippy, wasm target, cargo-contract (ink! 5.0.1)    apt-get install -y --no-install-recommends \        build-essential \

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.86.0 --profile minimal && \

    source /usr/local/cargo/env && \        build-essential \        clang \

    rustup component add rust-src --toolchain 1.86.0 && \

    rustup component add clippy --toolchain 1.86.0 && \        clang \        curl \

    rustup target add wasm32-unknown-unknown --toolchain 1.86.0 && \

    cargo install --locked cargo-contract --version 5.0.1        curl \        git \



# Install substrate-contracts-node v0.42.0        git \        pkg-config \

RUN wget -q https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz && \

    tar -xzf substrate-contracts-node-linux.tar.gz && \        pkg-config \        libssl-dev \

    install -m 0755 substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/substrate-contracts-node && \

    rm -rf substrate-contracts-node-linux substrate-contracts-node-linux.tar.gz        libssl-dev \        protobuf-compiler \



# Create non-root user for interactive workflows        protobuf-compiler \        ca-certificates \

RUN groupadd -g ${GID} ${USER} && \

    useradd -m -u ${UID} -g ${GID} ${USER}        ca-certificates \        wget \



USER ${USER}        wget \        jq \

WORKDIR /workspace

        jq \        cmake \

# Expose Substrate node ports

EXPOSE 9944 9933 30333        cmake \        binaryen \



CMD ["bash"]        binaryen \        sudo \

        sudo \    && rm -rf /var/lib/apt/lists/*

    && rm -rf /var/lib/apt/lists/*

# Use bash for all RUN commands so cargo env is available

# Use bash for all RUN commands so cargo env is availableSHELL ["/bin/bash", "-lc"]

SHELL ["/bin/bash", "-lc"]

# Install Rust 1.86.0, rust-src, clippy, wasm target, cargo-contract (ink! 5.0.1)

# Install Rust 1.86.0, rust-src, clippy, wasm target, cargo-contract (ink! 5.0.1)RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.86.0 --profile minimal && \

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.86.0 --profile minimal && \    source /usr/local/cargo/env && \

    source /usr/local/cargo/env && \    rustup component add rust-src --toolchain 1.86.0 && \

    rustup component add rust-src --toolchain 1.86.0 && \    rustup component add clippy --toolchain 1.86.0 && \

    rustup component add clippy --toolchain 1.86.0 && \    rustup target add wasm32-unknown-unknown --toolchain 1.86.0 && \

    rustup target add wasm32-unknown-unknown --toolchain 1.86.0 && \    cargo install --locked cargo-contract --version 5.0.1

    cargo install --locked cargo-contract --version 5.0.1

# Install substrate-contracts-node v0.42.0

# Install substrate-contracts-node v0.42.0RUN wget -q https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz && \

RUN wget -q https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz && \    tar -xzf substrate-contracts-node-linux.tar.gz && \

    tar -xzf substrate-contracts-node-linux.tar.gz && \    install -m 0755 substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/substrate-contracts-node && \

    install -m 0755 substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/substrate-contracts-node && \    rm -rf substrate-contracts-node-linux substrate-contracts-node-linux.tar.gz

    rm -rf substrate-contracts-node-linux substrate-contracts-node-linux.tar.gz

# Create non-root user for interactive workflows

# Create non-root user for interactive workflowsRUN groupadd -g ${GID} ${USER} && \

RUN groupadd -g ${GID} ${USER} && \    useradd -m -u ${UID} -g ${GID} ${USER}

    useradd -m -u ${UID} -g ${GID} ${USER}

USER ${USER}

USER ${USER}WORKDIR /workspace

WORKDIR /workspace

# Expose Substrate node ports

# Expose Substrate node portsEXPOSE 9944 9933 30333

EXPOSE 9944 9933 30333

CMD ["bash"]
CMD ["bash"]