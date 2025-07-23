#!/bin/bash

# Test contract interactions locally
set -e

echo "🧪 Testing PowerGrid Network Contract Interactions..."

# Load contract addresses
if [ ! -f "deployment/local-addresses.json" ]; then
    echo "❌ Local deployment not found. Run ./scripts/deploy-local.sh first"
    exit 1
fi

REGISTRY_ADDR=$(jq -r '.contracts.resource_registry' deployment/local-addresses.json)
TOKEN_ADDR=$(jq -r '.contracts.powergrid_token' deployment/local-addresses.json)
GRID_ADDR=$(jq -r '.contracts.grid_service' deployment/local-addresses.json)

echo "📋 Testing Resource Registry..."
cd contracts/resource_registry

# Test device registration
echo "  - Registering device..."
cargo contract call \
    --contract "$REGISTRY_ADDR" \
    --message register_device \
    --args '{"SmartPlug": null}' 1000 "Living Room" "Tesla" "Model-X" "1.0.0" \
    --value 1000000000000000000 \
    --suri //Bob \
    --execute

echo "🪙 Testing PowerGrid Token..."
cd ../powergrid_token

# Test token operations
echo "  - Checking token supply..."
cargo contract call \
    --contract "$TOKEN_ADDR" \
    --message total_supply \
    --suri //Alice

echo "⚡ Testing Grid Service..."
cd ../grid_service

# Test grid event creation
echo "  - Creating grid event..."
cargo contract call \
    --contract "$GRID_ADDR" \
    --message create_grid_event \
    --args "DemandResponse" 60 750 100 \
    --suri //Alice \
    --execute

cd ../..

echo "✅ All interaction tests completed!"
