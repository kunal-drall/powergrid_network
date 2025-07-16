# Use a recent stable Rust image as our base
FROM rust:1.78

# Install essential build tools and dependencies for WASM compilation
RUN apt-get update && apt-get install -y build-essential

# Set up the WASM toolchain
RUN rustup target add wasm32-unknown-unknown

# Install cargo-contract for building ink! smart contracts
RUN cargo install cargo-contract --version 4.0.1 --locked

# Install the Substrate Contracts Node for our local testnet
RUN cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.27.0 --force

# Set the working directory inside the container
WORKDIR /app

# Expose the ports for the Substrate node
# 9944: RPC endpoint for blockchain interaction
# 9615: Prometheus metrics endpoint
EXPOSE 9944 9615

# Default command to run when the container starts
# This will start a fresh dev node with a temporary state
CMD ["contracts-node", "--dev", "--unsafe-rpc-external", "--rpc-cors=all"]