#!/bin/bash
set -e

echo "🧪 Testing PowerGrid Network Contracts..."

# Test shared crate
echo "Testing shared crate..."
cargo test --manifest-path shared/Cargo.toml

# Test each contract
echo "Testing Resource Registry..."
cargo test --manifest-path contracts/resource-registry/Cargo.toml

echo "Testing Grid Service..."
cargo test --manifest-path contracts/grid-service/Cargo.toml

echo "Testing Token Contract..."
cargo test --manifest-path contracts/token/Cargo.toml

echo "Testing Governance Contract..."
cargo test --manifest-path contracts/governance/Cargo.toml

echo "✅ All tests passed!"