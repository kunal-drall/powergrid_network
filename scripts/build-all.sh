#!/bin/bash
set -e

echo "🔨 Building PowerGrid Network Contracts..."

# Install cargo-contract if not present
if ! command -v cargo-contract &> /dev/null; then
    echo "Installing cargo-contract..."
    cargo install --force --locked cargo-contract
fi

# Build each contract
echo "Building Resource Registry..."
cargo contract build --manifest-path contracts/resource_registry/Cargo.toml --release

echo "Building Grid Service..."
cargo contract build --manifest-path contracts/grid_service/Cargo.toml --release

echo "Building Token Contract..."
cargo contract build --manifest-path contracts/token/Cargo.toml --release

echo "Building Governance Contract..."
cargo contract build --manifest-path contracts/governance/Cargo.toml --release

# Copy artifacts
echo "Copying artifacts..."
mkdir -p artifacts

cp contracts/resource_registry/target/ink/resource_registry/resource_registry.wasm artifacts/ 2>/dev/null || echo "Resource registry WASM not found"
cp contracts/resource_registry/target/ink/resource_registry/resource_registry.json artifacts/ 2>/dev/null || echo "Resource registry metadata not found"

cp contracts/grid_service/target/ink/grid_service/grid_service.wasm artifacts/ 2>/dev/null || echo "Grid service WASM not found"
cp contracts/grid_service/target/ink/grid_service/grid_service.json artifacts/ 2>/dev/null || echo "Grid service metadata not found"

cp contracts/token/target/ink/powergrid_token/powergrid_token.wasm artifacts/ 2>/dev/null || echo "Token WASM not found"
cp contracts/token/target/ink/powergrid_token/powergrid_token.json artifacts/ 2>/dev/null || echo "Token metadata not found"

cp contracts/governance/target/ink/governance/governance.wasm artifacts/ 2>/dev/null || echo "Governance WASM not found"
cp contracts/governance/target/ink/governance/governance.json artifacts/ 2>/dev/null || echo "Governance metadata not found"

echo "✅ All contracts built successfully!"
echo "📁 Artifacts copied to ./artifacts/"