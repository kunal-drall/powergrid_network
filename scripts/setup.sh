#!/usr/bin/env bash
set -e

echo "=== Setting up environment for ink! contracts ==="

# 1. Update system packages
echo "--- Updating system packages ---"
sudo apt-get update -y
sudo apt-get upgrade -y
sudo apt-get install -y build-essential curl wget git clang pkg-config libssl-dev protobuf-compiler

# 2. Install Rust (if not already installed)
if ! command -v rustc &> /dev/null; then
  echo "--- Installing Rust ---"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source $HOME/.cargo/env
else
  echo "Rust is already installed"
fi

# 3. Verify Rust
rustc --version
cargo --version

# 4. Install correct cargo-contract version (5.0.1)
echo "--- Installing cargo-contract v5.0.1 ---"
cargo install --force --locked cargo-contract --version 5.0.1

# 5. Add wasm target + rust-src
echo "--- Adding wasm target and rust-src ---"
rustup target add wasm32-unknown-unknown
rustup component add rust-src --toolchain stable
rustup component list | grep rust-src

# 6. Install substrate-contracts-node (binary)
NODE_VERSION="v0.42.0"   # Latest version
NODE_URL="https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz"

echo "--- Downloading substrate-contracts-node ${NODE_VERSION} ---"
wget -q --show-progress $NODE_URL -O substrate-contracts-node-linux.tar.gz

echo "--- Extracting substrate-contracts-node ---"
tar -xzf substrate-contracts-node-linux.tar.gz

# The binary is extracted to substrate-contracts-node-linux/ directory
echo "--- Installing substrate-contracts-node to /usr/local/bin/ ---"
chmod +x substrate-contracts-node-linux/substrate-contracts-node
sudo mv substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/

# Clean up
rm -rf substrate-contracts-node-linux.tar.gz substrate-contracts-node-linux/

# 7. Verify installation
echo "--- Verifying installations ---"
cargo-contract --version
substrate-contracts-node --version

echo ""
echo "✅ Setup complete! You can now run:"
echo "   substrate-contracts-node --dev --tmp &"
echo "   cargo test --features e2e-tests"