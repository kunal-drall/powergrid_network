#!/bin/bash

echo "🚀 Deploying PowerGrid Network Locally..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Determine node URL based on environment
# Priority: CONTRACTS_NODE > NODE_URL > DOCKER_NETWORK > localhost
if [ -n "$CONTRACTS_NODE" ]; then
    WS_URL="$CONTRACTS_NODE"
    # Extract HTTP URL from WS URL for health check
    NODE_CHECK_URL=$(echo "$CONTRACTS_NODE" | sed 's/ws:/http:/g')
elif [ -n "$NODE_URL" ]; then
    WS_URL="$NODE_URL"
    NODE_CHECK_URL=$(echo "$NODE_URL" | sed 's/ws:/http:/g')
elif [ -n "$DOCKER_NETWORK" ]; then
    WS_URL="ws://node:9944"
    NODE_CHECK_URL="http://node:9944"
else
    WS_URL="ws://localhost:9944"
    NODE_CHECK_URL="http://localhost:9944"
fi

echo "🔗 Using node URL: $WS_URL"

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"

check_node() {
    echo "🔍 Checking if substrate-contracts-node is running on $WS_URL..."
    if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' "$NODE_CHECK_URL" 2>/dev/null | grep -q '"result"'; then
        echo -e "${GREEN}✅ substrate-contracts-node is running${NC}"
        return 0
    else
        echo -e "${RED}❌ substrate-contracts-node not responding${NC}"
        if [ -z "$DOCKER_NETWORK" ]; then
            echo -e "${YELLOW}💡 Please start it with: substrate-contracts-node --dev --tmp${NC}"
        else
            echo -e "${YELLOW}💡 Please ensure Docker node service is running${NC}"
        fi
        exit 1
    fi
}

deploy_token() {
    echo -e "${BLUE}🚀 Deploying PowerGrid Token...${NC}"
    
    cd "$ROOT_DIR/contracts/token" || exit 1
    
    echo "📦 Building PowerGrid Token..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying PowerGrid Token..."
    
    # Use individual variables to avoid quoting issues
    NAME="PowerGrid Token"
    SYMBOL="PWGD"
    DECIMALS=12
    SUPPLY=1000000000000000000000000  # 1 million tokens with 12 decimals
    
    cargo contract instantiate --constructor new --args "\"$NAME\"" "\"$SYMBOL\"" "$DECIMALS" "$SUPPLY" --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ PowerGrid Token deployed successfully${NC}"
        # Copy ABI
        cp target/ink/powergrid_token.json "$ROOT_DIR/backend/config/abis/" 2>/dev/null || true
        cd "$ROOT_DIR"
        return 0
    else
        echo -e "${RED}❌ PowerGrid Token deployment failed${NC}"
        cd "$ROOT_DIR"
        return 1
    fi
}

deploy_registry() {
    echo -e "${BLUE}🚀 Deploying Resource Registry...${NC}"
    
    cd "$ROOT_DIR/contracts/resource_registry" || exit 1
    
    echo "📦 Building Resource Registry..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying Resource Registry..."
    # Contract accepts min_stake in tokens (human-readable), converts to native units internally
    # 1 token minimum stake
    cargo contract instantiate --constructor new --args 1 --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Resource Registry deployed successfully${NC}"
        # Copy ABI
        cp target/ink/resource_registry.json "$ROOT_DIR/backend/config/abis/" 2>/dev/null || true
        cd "$ROOT_DIR"
        return 0
    else
        echo -e "${RED}❌ Resource Registry deployment failed${NC}"
        cd "$ROOT_DIR"
        return 1
    fi
}

deploy_grid_service() {
    local TOKEN_ADDR="$1"
    local REGISTRY_ADDR="$2"
    
    echo -e "${BLUE}🚀 Deploying Grid Service...${NC}"
    echo "   Token Address: $TOKEN_ADDR"
    echo "   Registry Address: $REGISTRY_ADDR"
    
    cd "$ROOT_DIR/contracts/grid_service" || exit 1
    
    echo "📦 Building Grid Service..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying Grid Service..."
    # Constructor: new(token_address: AccountId, registry_address: AccountId)
    cargo contract instantiate --constructor new --args "$TOKEN_ADDR" "$REGISTRY_ADDR" --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Grid Service deployed successfully${NC}"
        # Copy ABI
        cp target/ink/grid_service.json "$ROOT_DIR/backend/config/abis/" 2>/dev/null || true
        cd "$ROOT_DIR"
        return 0
    else
        echo -e "${RED}❌ Grid Service deployment failed${NC}"
        cd "$ROOT_DIR"
        return 1
    fi
}


deploy_governance() {
    echo -e "${BLUE}🚀 Deploying Governance...${NC}"
    
    cd "$ROOT_DIR/contracts/governance" || exit 1
    
    echo "📦 Building Governance..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying Governance..."
    # Constructor takes token_address as AccountId
    cargo contract instantiate --constructor new --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Governance deployed successfully${NC}"
        # Copy ABI
        cp target/ink/governance.json "$ROOT_DIR/backend/config/abis/" 2>/dev/null || true
        cd "$ROOT_DIR"
        return 0
    else
        echo -e "${RED}❌ Governance deployment failed${NC}"
        cd "$ROOT_DIR"
        return 1
    fi
}

main() {
    check_node
    mkdir -p "$ROOT_DIR/deployment"
    mkdir -p "$ROOT_DIR/backend/config/abis"
    
    echo "📋 Deploying contracts in dependency order..."
    
    # Deploy Token Contract
    echo -e "${BLUE}Step 1/4: Deploying PowerGrid Token...${NC}"
    if deploy_token; then
        echo "✅ Token deployment completed"
    else
        echo "❌ Token deployment failed"
        exit 1
    fi
    
    # Deploy Resource Registry
    echo -e "${BLUE}Step 2/4: Deploying Resource Registry...${NC}"
    if deploy_registry; then
        echo "✅ Registry deployment completed"
    else
        echo "❌ Registry deployment failed"
        exit 1
    fi
    
    # Deploy Grid Service
    echo -e "${BLUE}Step 3/4: Deploying Grid Service...${NC}"
    if deploy_grid_service; then
        echo "✅ Grid Service deployment completed"
    else
        echo "❌ Grid Service deployment failed"
        exit 1
    fi
    
    # Deploy Governance
    echo -e "${BLUE}Step 4/4: Deploying Governance...${NC}"
    if deploy_governance; then
        echo "✅ Governance deployment completed"
    else
        echo "❌ Governance deployment failed"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}🎉 All 4 contracts deployed successfully!${NC}"
    echo -e "${YELLOW}📝 Contract addresses are shown in the output above${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Copy the contract addresses from output above"
    echo "2. Update backend/.env with the addresses"
    echo "3. Run: cd backend && source venv/bin/activate && python src/oracle_service.py"
    echo ""
    
    # Create a basic deployment file
    cat > "$ROOT_DIR/deployment/local-addresses.json" << 'DEPLOY_EOF'
{
  "contracts": {
    "deployment_note": "Copy addresses from terminal output above",
    "powergrid_token": "COPY_FROM_OUTPUT",
    "resource_registry": "COPY_FROM_OUTPUT",
    "grid_service": "COPY_FROM_OUTPUT",
    "governance": "COPY_FROM_OUTPUT"
  },
  "network": "local",
  "deployer": "//Alice"
}
DEPLOY_EOF
    
    echo "📄 Deployment template created at deployment/local-addresses.json"
    echo "📁 ABIs copied to backend/config/abis/"
}

main "$@"

