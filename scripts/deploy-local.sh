#!/bin/bash

echo "🚀 Deploying PowerGrid Network Locally..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

check_node() {
    if curl -s http://localhost:9944 2>/dev/null | grep -q "Method is not allowed\|POST is required"; then
        echo -e "${GREEN}✅ ink-node is running on port 9944${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  ink-node not responding${NC}"
        exit 1
    fi
}

deploy_contract() {
    local CONTRACT_DIR=$1
    local CONTRACT_NAME=$2
    local ARGS=$3
    
    echo -e "${BLUE}🚀 Deploying $CONTRACT_NAME...${NC}"
    
    cd contracts/$CONTRACT_DIR
    
    OUTPUT=$(cargo contract instantiate \
        --constructor new \
        --args $ARGS \
        --suri //Alice \
        --url ws://localhost:9944 \
        --execute \
        --gas 1000000000000 \
        --proof-size 1000000 2>&1)
    
    echo "$OUTPUT"
    
    ADDRESS=$(echo "$OUTPUT" | grep -E "Contract [A-Za-z0-9]{48}" | head -1 | grep -oE "[A-Za-z0-9]{48}")
    
    if [ -z "$ADDRESS" ]; then
        echo -e "${RED}❌ Deployment failed for $CONTRACT_NAME${NC}"
        cd ../..
        return 1
    fi
    
    echo -e "${GREEN}✅ $CONTRACT_NAME: $ADDRESS${NC}"
    cd ../..
    echo "$ADDRESS"
}

main() {
    check_node
    mkdir -p deployment
    echo "📋 Deploying contracts in order..."
    
    REGISTRY=$(deploy_contract "resource_registry" "resource_registry" "1000000000000000000")
    if [ $? -ne 0 ]; then exit 1; fi
    
    TOKEN=$(deploy_contract "token" "powergrid_token" '"PowerGrid Token" "PGT" 18 1000000000000000000000')
    if [ $? -ne 0 ]; then exit 1; fi
    
    GRID=$(deploy_contract "grid_service" "grid_service" "\"$REGISTRY\" \"$TOKEN\" 750")
    if [ $? -ne 0 ]; then exit 1; fi
    
    GOV=$(deploy_contract "governance" "governance" "\"$TOKEN\" \"$REGISTRY\" \"$GRID\" 100000000000000000000 100 51")
    if [ $? -ne 0 ]; then exit 1; fi
    
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
