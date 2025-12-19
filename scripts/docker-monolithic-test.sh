#!/bin/bash
# PowerGrid Network Monolithic Docker Test Script
# Runs node in background and deploys contracts in the same container

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🚀 PowerGrid Monolithic Container Test"
echo "======================================="

# Start substrate-contracts-node in background
echo -e "${BLUE}Starting substrate-contracts-node...${NC}"
substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all &
NODE_PID=$!

# Wait for node to be ready
echo "⏳ Waiting for node to be ready..."
for i in {1..30}; do
    if curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9944 2>/dev/null | grep -q '"result"'; then
        echo -e "${GREEN}✅ Node is ready!${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ Node failed to start${NC}"
        kill $NODE_PID 2>/dev/null || true
        exit 1
    fi
    sleep 2
done

# Set node URL for deploy script
export NODE_URL="ws://localhost:9944"
export NODE_CHECK_URL="http://localhost:9944"

# Run the deploy script
echo -e "${BLUE}Deploying contracts...${NC}"
cd /workspace
./scripts/deploy-local.sh

# If we get here, success!
echo ""
echo -e "${GREEN}🎉 Monolithic test completed successfully!${NC}"
echo "Node PID: $NODE_PID"
echo ""
echo "You can now:"
echo "1. Run contract tests: cargo test --workspace"
echo "2. Interact with contracts manually"
echo ""
echo "Press Ctrl+C to stop the node..."

# Keep the node running for interactive use
wait $NODE_PID
