#!/bin/bash

# PowerGrid Network Local Deployment Script
set -e

echo "🚀 Deploying PowerGrid Network Locally..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if ink-node is running
check_node() {
    if ! curl -s http://localhost:9933/health >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  ink-node not responding. Make sure it's running:${NC}"
        echo "   ink-node --dev"
        exit 1
    fi
    echo -e "${GREEN}✅ ink-node is running${NC}"
}

# Deploy a contract
deploy_contract() {
    local CONTRACT_NAME=$1
    local ARGS=$2
    
    echo -e "${BLUE}🚀 Deploying $CONTRACT_NAME...${NC}"
    cd contracts/$CONTRACT_NAME
    
    # Deploy and capture output
    OUTPUT=$(cargo contract instantiate \
        --constructor new \
        --args "$ARGS" \
        --suri //Alice \
        --url ws://localhost:9944 \
        --execute 2>&1)
    
    # Extract contract address
    ADDRESS=$(echo "$OUTPUT" | grep -o "Contract [A-Za-z0-9]*" | cut -d' ' -f2 | head -1)
    
    if [ -z "$ADDRESS" ]; then
        echo "❌ Deployment failed for $CONTRACT_NAME"
        echo "$OUTPUT"
        exit 1
    fi
    
    echo -e "${GREEN}✅ $CONTRACT_NAME: $ADDRESS${NC}"
    cd ../..
    echo "$ADDRESS"
}

# Main deployment
main() {
    check_node
    
    mkdir -p deployment
    
    echo "📋 Deploying contracts in order..."
    
    REGISTRY=$(deploy_contract "resource_registry" "1000000000000000000")
    TOKEN=$(deploy_contract "powergrid_token" '"PowerGrid Token" "PGT" 18 1000000000000000000000')
    GRID=$(deploy_contract "grid_service" "\"$REGISTRY\" \"$TOKEN\" 750")
    GOV=$(deploy_contract "governance" "\"$TOKEN\" \"$REGISTRY\" \"$GRID\" 100000000000000000000 100 51")
    
    # Save addresses
    cat > deployment/local-addresses.json << JSON
{
  "resource_registry": "$REGISTRY",
  "powergrid_token": "$TOKEN", 
  "grid_service": "$GRID",
  "governance": "$GOV"
}
JSON
    
    echo ""
    echo -e "${GREEN}🎉 All contracts deployed!${NC}"
    echo "📄 Addresses saved to: deployment/local-addresses.json"
}

main "$@"
