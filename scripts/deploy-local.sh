#!/bin/bash

echo "🚀 Deploying PowerGrid Network Locally..."

GREEN='\\033[0;32m'
BLUE='\\033[0;34m'
YELLOW='\\033[1;33m'
RED='\\033[0;31m'
NC='\\033[0m'

# Determine node URL based on environment
if [ -n "$NODE_URL" ]; then
    WS_URL="$NODE_URL"
elif [ -n "$DOCKER_NETWORK" ]; then
    WS_URL="ws://node:9944"
    NODE_CHECK_URL="http://node:9944"
else
    WS_URL="ws://localhost:9944"
    NODE_CHECK_URL="http://localhost:9944"
fi

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
    
    cd contracts/token || exit 1
    
    echo "📦 Building PowerGrid Token..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying PowerGrid Token..."
    
    # Use individual variables to avoid quoting issues
    NAME="PowerGrid Token"
    SYMBOL="PGT"
    DECIMALS=18
    SUPPLY=1000000000000000000000
    
    cargo contract instantiate --constructor new --args "\"$NAME\"" "\"$SYMBOL\"" "$DECIMALS" "$SUPPLY" --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ PowerGrid Token deployed successfully${NC}"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ PowerGrid Token deployment failed${NC}"
        cd ../..
        return 1
    fi
}

deploy_registry() {
    echo -e "${BLUE}🚀 Deploying Resource Registry...${NC}"
    
    cd contracts/resource_registry || exit 1
    
    echo "📦 Building Resource Registry..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying Resource Registry..."
    
    cargo contract instantiate --constructor new --args 1000000000000000000 --suri //Alice --url "$WS_URL" --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Resource Registry deployed successfully${NC}"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Resource Registry deployment failed${NC}"
        cd ../..
        return 1
    fi
}

main() {
    check_node
    mkdir -p deployment
    
    echo "📋 Deploying contracts in dependency order..."
    
    # Deploy Token Contract
    echo -e "${BLUE}Step 1: Deploying PowerGrid Token...${NC}"
    if deploy_token; then
        echo "✅ Token deployment completed"
    else
        echo "❌ Token deployment failed"
        exit 1
    fi
    
    # Deploy Resource Registry
    echo -e "${BLUE}Step 2: Deploying Resource Registry...${NC}"
    if deploy_registry; then
        echo "✅ Registry deployment completed"
    else
        echo "❌ Registry deployment failed"
        exit 1
    fi
    
    echo -e "${GREEN}🎉 Deployment completed successfully!${NC}"
    echo -e "${YELLOW}Contract addresses are shown in the output above${NC}"
    
    # Create a basic deployment file
    cat > deployment/local-addresses.json << 'DEPLOY_EOF'
{
  "contracts": {
    "deployment_note": "Addresses extracted from output above",
    "powergrid_token": "See deployment output",
    "resource_registry": "See deployment output"
  },
  "network": "local",
  "deployed_at": "TIMESTAMP_PLACEHOLDER",
  "deployer": "//Alice"
}
DEPLOY_EOF
    
    # Update timestamp
    sed -i "s/TIMESTAMP_PLACEHOLDER/$(date -u +%Y-%m-%dT%H:%M:%SZ)/g" deployment/local-addresses.json
    
    echo "📄 Basic deployment file created at deployment/local-addresses.json"
}

main "$@"
