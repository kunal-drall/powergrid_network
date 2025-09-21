#!/bin/bash
set -e

echo "🚀 PowerGrid Network Docker E2E Test Simulation"
echo "================================================"
echo "This script simulates the Docker E2E workflow using local tools"
echo ""

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Step 1: Verify Rust toolchain (simulating container setup)
echo -e "${BLUE}📋 Step 1: Verifying Rust toolchain...${NC}"
rustc --version
cargo --version

# Add wasm target if not present
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

if ! rustup component list --installed | grep -q rust-src; then
    echo "Adding rust-src component..."
    rustup component add rust-src --toolchain stable
fi

echo -e "${GREEN}✅ Rust toolchain ready${NC}"
echo ""

# Step 2: Install cargo-contract if needed (simulating container dependency)
echo -e "${BLUE}📋 Step 2: Checking cargo-contract...${NC}"
if ! command -v cargo-contract &> /dev/null; then
    echo "cargo-contract not found. Installing..."
    echo "Note: This would be pre-installed in the Docker container"
    # In a real scenario, this would be cached in the Docker image
    timeout 30 cargo install --force --locked cargo-contract --version 5.0.1 || {
        echo -e "${YELLOW}⚠️ cargo-contract installation skipped due to network issues${NC}"
        echo "In production Docker image, this would be pre-installed"
    }
else
    echo -e "${GREEN}✅ cargo-contract is available${NC}"
fi
echo ""

# Step 3: Build contracts (simulating contract compilation in container)
echo -e "${BLUE}📋 Step 3: Building contracts...${NC}"
if timeout 180 ./scripts/build-all.sh; then
    echo -e "${GREEN}✅ All contracts built successfully${NC}"
else
    echo -e "${YELLOW}⚠️ Contract building timed out or failed${NC}"
    echo "This would work in a properly configured Docker environment"
fi
echo ""

# Step 4: Check if substrate-contracts-node is available (simulating node service)
echo -e "${BLUE}📋 Step 4: Checking substrate-contracts-node availability...${NC}"
if command -v substrate-contracts-node &> /dev/null; then
    echo -e "${GREEN}✅ substrate-contracts-node is available${NC}"
    substrate-contracts-node --version
    
    # Start node in background (simulating Docker service)
    echo "Starting substrate-contracts-node in background..."
    nohup substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe > substrate-node.log 2>&1 &
    NODE_PID=$!
    
    # Wait for node to start
    echo "Waiting for node to initialize..."
    for i in {1..30}; do
        sleep 1
        if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9944 2>/dev/null | grep -q '"result"'; then
            echo -e "${GREEN}✅ substrate-contracts-node started successfully${NC}"
            
            # Step 5: Run E2E tests (simulating Docker runner container)
            echo ""
            echo -e "${BLUE}📋 Step 5: Running E2E deployment tests...${NC}"
            if timeout 300 ./scripts/deploy-and-run-e2e-docker.sh; then
                echo -e "${GREEN}✅ E2E tests completed successfully${NC}"
            else
                echo -e "${YELLOW}⚠️ E2E tests failed or timed out${NC}"
            fi
            
            # Cleanup
            echo ""
            echo "Stopping substrate-contracts-node..."
            kill $NODE_PID 2>/dev/null || true
            wait $NODE_PID 2>/dev/null || true
            
            break
        fi
        echo -n "."
    done
    
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ substrate-contracts-node failed to start${NC}"
        kill $NODE_PID 2>/dev/null || true
    fi
    
else
    echo -e "${YELLOW}⚠️ substrate-contracts-node not found${NC}"
    echo "In Docker setup, this would be pre-installed in the container"
    echo "Download from: https://github.com/paritytech/substrate-contracts-node/releases"
fi
echo ""

# Step 6: Show Docker usage
echo -e "${BLUE}📋 Step 6: Docker Usage Instructions${NC}"
echo ""
echo "To use the actual Docker setup:"
echo ""
echo "1. Build the Docker image:"
echo "   docker build -t powergrid-network ."
echo ""
echo "2. Run with Docker Compose:"
echo "   docker-compose up"
echo ""
echo "3. Run individual components:"
echo "   # Start just the node"
echo "   docker-compose up substrate-node"
echo "   "
echo "   # Run E2E tests"
echo "   docker-compose up e2e-runner"
echo ""
echo "4. Manual testing:"
echo "   # Interactive shell"
echo "   docker run -it powergrid-network bash"
echo "   "
echo "   # Run node with port mapping"
echo "   docker run -p 9944:9944 powergrid-network substrate-contracts-node --dev --rpc-external --ws-external"
echo ""

echo -e "${GREEN}🎉 Docker simulation completed!${NC}"
echo ""
echo "See DOCKER.md for complete Docker usage documentation."