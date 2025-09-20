# PowerGrid Network Scripts

This directory contains the core scripts for building, testing, and deploying the PowerGrid Network smart contracts.

## Scripts Overview

### `setup.sh`
**Purpose**: Sets up the complete development environment for ink! smart contracts.

**What it does**:
- Installs and updates system packages (build-essential, curl, wget, git, clang, etc.)
- Installs Rust if not already present
- Installs cargo-contract v5.0.1 (specific version for compatibility)
- Adds wasm32-unknown-unknown target for WebAssembly compilation
- Installs rust-src component
- Downloads and installs substrate-contracts-node v0.42.0
- Verifies all installations

**Usage**:
```bash
./scripts/setup.sh
```

**Prerequisites**: sudo access for package installation

---

### `run-node.sh`
**Purpose**: Starts the substrate-contracts-node for local development and testing.

**What it does**:
- Checks if substrate-contracts-node is installed
- Starts the node in development mode with:
  - `--dev` flag for development chain
  - `--tmp` flag for temporary storage (clean state on restart)
  - `--rpc-cors all` to allow all CORS requests
  - `--rpc-methods=unsafe` for testing purposes

**Usage**:
```bash
./scripts/run-node.sh
```

**Prerequisites**: substrate-contracts-node must be installed (run `setup.sh` first)

**Note**: This command runs in the foreground. Keep the terminal open while testing contracts.

---

### `build-all.sh`
**Purpose**: Builds all smart contracts in the project.

**What it does**:
- Builds each contract: governance, grid_service, resource_registry, token
- Runs `cargo clippy` for linting
- Runs `cargo contract build --release` for optimized WASM builds
- Reports success/failure for each contract

**Usage**:
```bash
./scripts/build-all.sh
```

**Prerequisites**: Rust toolchain and cargo-contract installed

---

### `test-all.sh`
**Purpose**: Runs unit tests for all smart contracts.

**What it does**:
- Discovers all contract directories in `contracts/`
- Runs `cargo test` in each contract directory
- Reports test results for each contract

**Usage**:
```bash
./scripts/test-all.sh
```

**Prerequisites**: Contracts must be buildable

---

### `deploy-and-run-e2e.sh`
**Purpose**: Deploys all contracts to local node and runs end-to-end tests.

**What it does**:
- Checks if substrate-contracts-node is running on port 9944
- Deploys contracts in dependency order:
  1. PowerGrid Token (PGT)
  2. Resource Registry
  3. Grid Service
  4. Governance
- Creates deployment record in `deployment/local-addresses.json`
- Performs basic deployment verification
- Provides guidance for further testing

**Usage**:
```bash
./scripts/deploy-and-run-e2e.sh
```

**Prerequisites**: 
- substrate-contracts-node running (`./scripts/run-node.sh`)
- All contracts built (`./scripts/build-all.sh`)

## Complete Workflow

For a complete development workflow, run these scripts in order:

1. **Initial Setup** (one-time):
   ```bash
   ./scripts/setup.sh
   ```

2. **Build Contracts**:
   ```bash
   ./scripts/build-all.sh
   ```

3. **Run Tests**:
   ```bash
   ./scripts/test-all.sh
   ```

4. **Start Local Node** (in separate terminal):
   ```bash
   ./scripts/run-node.sh
   ```

5. **Deploy and Test**:
   ```bash
   ./scripts/deploy-and-run-e2e.sh
   ```

## Troubleshooting

- If `setup.sh` fails, check internet connection and sudo permissions
- If `run-node.sh` fails, ensure substrate-contracts-node is in PATH
- If `build-all.sh` fails, check Rust toolchain and dependencies
- If `deploy-and-run-e2e.sh` fails, ensure the node is running on port 9944

## File Locations

- Contract source code: `contracts/*/src/`
- Build artifacts: `contracts/*/target/`
- Deployment records: `deployment/`
- Documentation: `docs/`