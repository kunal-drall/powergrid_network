# PowerGrid Network — Local Setup, Testing, and Docker Guide

A practical walkthrough for installing dependencies, running the Substrate contracts node, and executing every build and test flow for the PowerGrid Network contracts. Instructions cover Linux (Ubuntu) and macOS environments, plus a ready-to-use Docker image.

---

## Quick Reference

| Task | Command |
| --- | --- |
| Clone repository | `git clone https://github.com/kunal-drall/powergrid_network.git` |
| Build everything | `cargo build --workspace` |
| Run all unit tests | `cargo test --workspace` |
| Launch local node | `substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe` |
| Run ink! e2e tests | `CARGO_TARGET_DIR=/tmp/cargo-target cargo test -p integration-tests --features e2e-tests -- --nocapture` |
| Helper scripts | `./scripts/build-all.sh`, `./scripts/test-all.sh`, `./scripts/deploy-local.sh` |

> ℹ️ The e2e suite requires a running `substrate-contracts-node`. Start it in a separate terminal before launching the tests.

---

## 1. Install Prerequisites

All environments need:

- Rust toolchain via `rustup` (stable channel)
- `wasm32-unknown-unknown` target + `rust-src` component
- `cargo-contract` **v5.0.1**
- `substrate-contracts-node` **v0.42.0**
- Standard build utilities (clang, pkg-config, libssl, protobuf)

### 1.1 Ubuntu 22.04+ (Linux)

#### Option A — One-liner helper script (recommended)

```bash
bash scripts/setup.sh
```

The script installs system packages, Rust, `cargo-contract`, and downloads the prebuilt `substrate-contracts-node` binary.

#### Option B — Manual steps

```bash
# System dependencies
sudo apt update
sudo apt install -y build-essential curl wget git clang pkg-config libssl-dev protobuf-compiler jq

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"

rustup target add wasm32-unknown-unknown
rustup component add rust-src

# ink! tooling
cargo install --locked cargo-contract --version 5.0.1

# Substrate contracts node v0.42.0
NODE_VERSION="v0.42.0"
wget https://github.com/paritytech/substrate-contracts-node/releases/download/${NODE_VERSION}/substrate-contracts-node-linux.tar.gz
mkdir substrate-contracts-node-linux
tar -xzf substrate-contracts-node-linux.tar.gz -C substrate-contracts-node-linux --strip-components=1
sudo install substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/substrate-contracts-node
rm -rf substrate-contracts-node-linux substrate-contracts-node-linux.tar.gz

substrate-contracts-node --version
```

> 💡 On low-storage systems (CI, Codespaces), set `export CARGO_TARGET_DIR=/tmp/cargo-target` before building to keep artifacts on the ephemeral filesystem.

### 1.2 macOS 13+ (Intel & Apple Silicon)

1. **Install system packages** (requires [Homebrew](https://brew.sh/)):
   ```bash
   brew update
   brew install rustup-init clang protobuf cmake pkg-config openssl@3 binaryen jq
   ``

2. **Install Rust**:
   ```bash
   rustup-init --default-toolchain stable --profile minimal -y
   source "$HOME/.cargo/env"
   rustup target add wasm32-unknown-unknown
   rustup component add rust-src
   ```

3. **Expose OpenSSL headers** (needed by some crates):
   ```bash
   echo 'export OPENSSL_DIR="$(brew --prefix openssl@3)"' >> ~/.zshrc
   echo 'export PKG_CONFIG_PATH="$(brew --prefix openssl@3)/lib/pkgconfig"' >> ~/.zshrc
   source ~/.zshrc
   ```

4. **Install ink! tooling**:
   ```bash
   cargo install --locked cargo-contract --version 5.0.1
   ```

5. **Install Substrate contracts node** (build from source on macOS):
   ```bash
   cargo install --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.42.0 --locked --force
   ```

6. **Verify**:
   ```bash
   cargo-contract --version
   substrate-contracts-node --version
   ```

---

## 2. Run the Local Contracts Node

Start the node in a dedicated terminal:

```bash
substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe
```

- `--dev` spins up an in-memory chain.
- `--tmp` cleans up state when the process exits.
- `--rpc-*` flags expose RPC for contract deployment and the E2E tests.

The process keeps the terminal busy; leave it running while executing contracts or tests.

---

## 3. Build and Test Workflow

All commands below assume you are in the repository root.

### 3.1 Standard build

```bash
cargo build --workspace
```

### 3.2 Unit & doc tests

```bash
cargo test --workspace
```

### 3.3 ink! end-to-end (E2E) tests

1. Ensure `substrate-contracts-node` is running.
2. Run the integration suite (optionally redirect build artifacts to `/tmp`):

   ```bash
   CARGO_TARGET_DIR=/tmp/cargo-target cargo test -p integration-tests --features e2e-tests -- --nocapture
   ```

The run compiles all Wasm artifacts, deploys each contract to the node, and exercises cross-contract flows.

### 3.4 Helper scripts

- `./scripts/build-all.sh` — Builds every contract package.
- `./scripts/test-all.sh` — Runs workspace tests plus ink! E2E tests.
- `./scripts/deploy-local.sh` — Deploys contracts to the node (expects RPC at `127.0.0.1:9944`).

Run scripts with `bash` on Linux/macOS:

```bash
bash ./scripts/test-all.sh
```

---

## 4. Docker-Based Workflow

A reproducible development container is provided via the project `Dockerfile`.

### 4.1 Build the image

```bash
docker build -t powergrid-dev .
```

### 4.2 Start an interactive container

```bash
docker run --rm -it \
  -v $(pwd):/workspace \
  -w /workspace \
  -p 9944:9944 \
  powergrid-dev
```

Inside the container you have Rust, `cargo-contract`, and `substrate-contracts-node` preinstalled. Typical workflow:

```bash
# Inside container shell
git status
substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe
```

Open a **second** terminal to reuse the same container (or start a new one) and run builds/tests:

```bash
# Example: run full e2e suite inside container
CARGO_TARGET_DIR=/tmp/cargo-target cargo test -p integration-tests --features e2e-tests -- --nocapture
```

To keep the node running in the background inside the container, use:

```bash
substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe &
```

### 4.3 Cleaning up

The container uses `/tmp/cargo-target` for build artifacts. When you exit, all intermediate files are discarded, leaving your host workspace untouched.

---

## 5. Troubleshooting & Tips

- **Slow builds on macOS** — Install `binaryen` (already in the brew list) to speed up Wasm optimization.
- **Port conflicts** — The node listens on `127.0.0.1:9944` by default. Stop other Substrate nodes or change the port using `--ws-port` / `--rpc-port`.
- **Disk pressure** — Redirect build artifacts: `export CARGO_TARGET_DIR=/tmp/cargo-target`.
- **Upgrade tooling** — Re-run the relevant install commands (e.g., `cargo install --force cargo-contract --version 5.0.1`).

---

## 6. Next Steps

Once setup is complete, explore:

- `docs/README.md` for governance and oracle integration notes.
- `contracts/integration-tests/src/real_e2e_tests.rs` for end-to-end scenarios.
- `shared/src/types.rs` for the list of governance proposal variants.

Happy hacking! 🚀
