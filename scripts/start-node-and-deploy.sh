#!/usr/bin/env bash
set -e

echo "🚀 Starting substrate-contracts-node and deploying contracts..."

# Kill any existing node
pkill -9 -f substrate-contracts-node 2>/dev/null || true
sleep 2

# Start node
echo "📡 Starting node..."
~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all --unsafe-rpc-external > /tmp/substrate-node.log 2>&1 &
NODE_PID=$!
echo "Node started with PID: $NODE_PID"

# Wait for node to be ready
echo "⏳ Waiting for node to be ready..."
for i in {1..30}; do
    sleep 2
    if curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9944 > /dev/null 2>&1; then
        echo "✅ Node is ready!"
        break
    else
        echo "   Waiting... ($i/30)"
    fi
done

# Deploy contracts
echo ""
echo "🚀 Deploying contracts..."
export PATH="$HOME/.local/bin:$PATH"
cd "$(dirname "$0")/.."
./scripts/deploy-local.sh

echo ""
echo "✅ Done! Check /tmp/contract-deployment.log for contract addresses"

