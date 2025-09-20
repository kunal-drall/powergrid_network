#!/usr/bin/env bash
set -e

echo "🚀 PowerGrid Network - Deploy and Run E2E Tests (Docker Version)"
echo "================================================================"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Set default node URL - can be overridden by environment variable
NODE_URL=${NODE_URL:-"ws://localhost:9944"}
NODE_HTTP_URL=$(echo "$NODE_URL" | sed 's/ws:/http:/' | sed 's/:9944/:9944/')

echo "Using node URL: $NODE_URL"

# Check if substrate-contracts-node is running
check_node() {
    echo "🔍 Checking if substrate-contracts-node is running..."
    local node_host=$(echo "$NODE_HTTP_URL" | sed -E 's|.*://([^:/]+).*|\1|')
    local node_port=$(echo "$NODE_HTTP_URL" | sed -E 's|.*:([0-9]+).*|\1|')
    
    if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' "$NODE_HTTP_URL" 2>/dev/null | grep -q '"result"'; then
        echo -e "${GREEN}✅ substrate-contracts-node is running at $NODE_HTTP_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ substrate-contracts-node not responding at $NODE_HTTP_URL${NC}"
        
        # In Docker, we don't try to start the node ourselves
        if [ -n "$DOCKER_CONTAINER" ]; then
            echo -e "${RED}Please ensure the substrate-contracts-node service is running${NC}"
            exit 1
        fi
        
        echo -e "${YELLOW}💡 Starting substrate-contracts-node in dev mode...${NC}"
        
        # Try to start substrate-contracts-node if available
        if command -v substrate-contracts-node &> /dev/null; then
            echo "Starting substrate-contracts-node in background..."
            nohup substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe > substrate-node.log 2>&1 &
            SUBSTRATE_PID=$!
            
            # Wait for node to start
            echo "Waiting for node to initialize..."
            for i in {1..30}; do
                sleep 1
                if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' "$NODE_HTTP_URL" 2>/dev/null | grep -q '"result"'; then
                    echo -e "${GREEN}✅ substrate-contracts-node started successfully${NC}"
                    return 0
                fi
                echo -n "."
            done
            echo ""
            echo -e "${RED}❌ Failed to start substrate-contracts-node${NC}"
            kill $SUBSTRATE_PID 2>/dev/null || true
        else
            echo -e "${YELLOW}💡 substrate-contracts-node not found. Please install it:${NC}"
            echo "cargo install contracts-node --force"
            echo "Then start it manually with: substrate-contracts-node --dev"
        fi
        exit 1
    fi
}

# Deploy a contract and capture its address
deploy_contract() {
    local contract_dir=$1
    local constructor=$2
    local args="$3"
    local contract_name=$4
    local output_var=$5
    
    echo -e "${BLUE}🚀 Deploying $contract_name...${NC}"
    
    cd "contracts/$contract_dir" || exit 1
    
    echo "📦 Building $contract_name..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying $contract_name..."
    
    local cmd="cargo contract instantiate --constructor $constructor"
    if [ -n "$args" ]; then
        cmd="$cmd --args $args"
    fi
    cmd="$cmd --suri //Alice --url $NODE_URL --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0"
    
    # Capture deployment output
    local deploy_output
    if deploy_output=$(eval "$cmd" 2>&1); then
        echo -e "${GREEN}✅ $contract_name deployed successfully${NC}"
        
        # Extract contract address from output
        local contract_address
        contract_address=$(echo "$deploy_output" | grep -E "Contract [a-zA-Z0-9]{48}" | head -1 | sed -E 's/.*Contract ([a-zA-Z0-9]{48}).*/\1/')
        
        if [ -z "$contract_address" ]; then
            # Try alternative pattern for contract address extraction
            contract_address=$(echo "$deploy_output" | grep -oE '[15][a-km-zA-HJ-NP-Z1-9]{47}' | head -1)
        fi
        
        if [ -n "$contract_address" ] && [ -n "$output_var" ]; then
            eval "$output_var=\"$contract_address\""
            echo "📍 Contract address: $contract_address"
        fi
        
        echo "$deploy_output"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ $contract_name deployment failed${NC}"
        echo "$deploy_output"
        cd ../..
        return 1
    fi
}

# Test contract interaction and capture output
test_contract_call() {
    local contract_dir=$1
    local contract_addr=$2
    local message=$3
    local args_string="$4"
    local signer=$5
    local value=${6:-"0"}
    local execute_flag=${7:-""}
    local description="$8"
    
    echo -e "${BLUE}📞 Testing: $description${NC}"
    
    cd "contracts/$contract_dir" || exit 1
    
    local cmd="cargo contract call --contract $contract_addr --message $message"
    
    if [ -n "$args_string" ]; then
        cmd="$cmd --args $args_string"
    fi
    
    if [ -n "$signer" ]; then
        cmd="$cmd --suri $signer"
    else
        cmd="$cmd --suri //Alice"
    fi
    
    cmd="$cmd --url $NODE_URL"
    
    if [ -n "$value" ] && [ "$value" != "0" ]; then
        cmd="$cmd --value $value"
    fi
    
    if [ "$execute_flag" = "--execute" ]; then
        cmd="$cmd --execute --skip-confirm"
    else
        cmd="$cmd --dry-run"
    fi
    
    echo "Executing: $cmd"
    
    if eval "$cmd"; then
        echo -e "${GREEN}✅ Test passed: $description${NC}"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Test failed: $description${NC}"
        cd ../..
        return 1
    fi
}

# Main deployment and testing function
main() {
    echo "Starting PowerGrid Network E2E deployment and testing..."
    
    # Check if node is running
    check_node
    
    # Create deployment directory
    mkdir -p deployment
    
    echo ""
    echo -e "${BLUE}📦 Deploying contracts in dependency order...${NC}"
    echo ""
    
    # 1. Deploy PowerGrid Token (PGT)
    echo "=== Deploying PowerGrid Token ==="
    if deploy_contract "token" "new" "1000000000000000000000000" "PowerGrid Token" "token_address"; then
        echo -e "${GREEN}✅ PowerGrid Token deployed at: $token_address${NC}"
    else
        echo -e "${RED}❌ Failed to deploy PowerGrid Token${NC}"
        exit 1
    fi
    
    echo ""
    
    # 2. Deploy Resource Registry
    echo "=== Deploying Resource Registry ==="
    if deploy_contract "resource_registry" "new" "$token_address" "Resource Registry" "registry_address"; then
        echo -e "${GREEN}✅ Resource Registry deployed at: $registry_address${NC}"
    else
        echo -e "${RED}❌ Failed to deploy Resource Registry${NC}"
        exit 1
    fi
    
    echo ""
    
    # 3. Deploy Grid Service
    echo "=== Deploying Grid Service ==="
    if deploy_contract "grid_service" "new" "$token_address $registry_address" "Grid Service" "grid_service_address"; then
        echo -e "${GREEN}✅ Grid Service deployed at: $grid_service_address${NC}"
    else
        echo -e "${RED}❌ Failed to deploy Grid Service${NC}"
        exit 1
    fi
    
    echo ""
    
    # 4. Deploy Governance
    echo "=== Deploying Governance ==="
    if deploy_contract "governance" "new" "$token_address" "Governance" "governance_address"; then
        echo -e "${GREEN}✅ Governance deployed at: $governance_address${NC}"
    else
        echo -e "${RED}❌ Failed to deploy Governance${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}🎉 All contracts deployed successfully!${NC}"
    echo ""
    
    # Save deployment addresses
    cat > deployment/local-addresses.json << EOF
{
    "token": "$token_address",
    "resource_registry": "$registry_address", 
    "grid_service": "$grid_service_address",
    "governance": "$governance_address",
    "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "node_url": "$NODE_URL"
}
EOF
    
    echo -e "${BLUE}📋 Deployment Summary:${NC}"
    echo "Token: $token_address"
    echo "Resource Registry: $registry_address"
    echo "Grid Service: $grid_service_address"
    echo "Governance: $governance_address"
    echo ""
    echo -e "${YELLOW}📋 Next Steps:${NC}"
    echo "1. Contract addresses saved to deployment/local-addresses.json"
    echo "2. Run unit tests: ./scripts/test-all.sh"
    echo "3. Run integration tests: cd contracts/integration-tests && cargo test --features e2e-tests"
    echo "4. Use addresses above for manual contract interactions"
}

# Set Docker environment flag
export DOCKER_CONTAINER=1

main "$@"