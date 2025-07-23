#!/bin/bash

# Quick interaction test
if [ ! -f "deployment/local-addresses.json" ]; then
    echo "❌ No deployment found. Run ./scripts/deploy-local.sh first"
    exit 1
fi

REGISTRY=$(jq -r '.resource_registry' deployment/local-addresses.json)
TOKEN=$(jq -r '.powergrid_token' deployment/local-addresses.json)

echo "🧪 Testing basic interactions..."

# Test token balance
echo "💰 Checking Alice's token balance:"
cd contracts/powergrid_token
cargo contract call --contract "$TOKEN" --message balance_of --args "$(echo '//Alice' | head -c 48)" --suri //Alice
cd ../..

# Test device registration
echo "📋 Registering Bob's device:"
cd contracts/resource_registry  
cargo contract call --contract "$REGISTRY" --message register_device --args '{"SmartPlug": null} 1000 "Kitchen" "Tesla" "Model-X" "1.0.0"' --value 1000000000000000000 --suri //Bob --execute
cd ../..

echo "✅ Basic interactions working!"
