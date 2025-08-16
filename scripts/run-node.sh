#!/usr/bin/env bash
set -e

echo "=== Starting substrate-contracts-node ==="

# Check if substrate-contracts-node is installed
if ! command -v substrate-contracts-node &> /dev/null; then
  echo "❌ substrate-contracts-node not found in PATH"
  echo "   Please install it first (see setup.sh)."
  exit 1
fi

# Run in dev mode with RPC enabled
substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe
