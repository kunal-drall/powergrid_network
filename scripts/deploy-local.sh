#!/bin/bash

# PowerGrid Network Local Deployment Script
set -e

echo "🚀 Deploying PowerGrid Network Locally..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if ink-node is running
check_node() {
    if ! pgrep -f "ink-node" > /dev/null; then
        echo -e "${YELLOW}⚠️  ink-node not running. Starting local node...${NC}"
        echo "Please run 'ink-node' in another terminal and press Enter to continue..."
        read -r
    else
        echo -e "${GREEN}✅ ink-node is running${NC}"
    fi
}

# Deploy a single contract
deploy_contract() {
    local CONTRACT_NAME=$1
    local CONSTRUCTOR_ARGS=$2
    
    echo -e "${BLUE}🚀 Deploying $CONTRACT_NAME...${NC}"
    
    cd contracts/$CONTRACT_NAME
    
    # Build the contract first
    cargo contract build --release
    
    # Deploy the contract
    DEPLOY_OUTPUT=$(cargo contract instantiate \
        --constructor new \
        --args "$CONSTRUCTOR_ARGS" \
        --suri //Alice \
        --execute \
        2>&1)
    
    echo "$DEPLOY_OUTPUT"
    
    # Extract contract address from output
    CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep -o "Contract [A-Za-z0-9]*" | cut -d' ' -f2 | head -1)
    
    if [ -z "$CONTRACT_ADDRESS" ]; then
        echo -e "${RED}❌ Failed to deploy $CONTRACT_NAME${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ $CONTRACT_NAME deployed at: $CONTRACT_ADDRESS${NC}"
    
    # Save address to deployment file
    cd ../..
    mkdir -p deployment
    echo "$CONTRACT_ADDRESS" > "deployment/local-${CONTRACT_NAME}.addr"
    
    echo "$CONTRACT_ADDRESS"
}

# Main deployment function
main() {
    echo -e "${BLUE}🏗️  PowerGrid Network Local Deployment${NC}"
    echo ""
    
    # Check if node is running
    check_node
    
    # Create deployment directory
    mkdir -p deployment
    
    # Deploy contracts in order
    echo -e "${YELLOW}📋 Step 1: Deploying Resource Registry...${NC}"
    REGISTRY_ADDR=$(deploy_contract "resource_registry" "1000000000000000000")
    
    echo -e "${YELLOW}🪙 Step 2: Deploying PowerGrid Token...${NC}"
    TOKEN_ADDR=$(deploy_contract "powergrid_token" '"PowerGrid Token" "PGT" 18 1000000000000000000000')
    
    echo -e "${YELLOW}⚡ Step 3: Deploying Grid Service...${NC}"
    GRID_ADDR=$(deploy_contract "grid_service" "\"$REGISTRY_ADDR\" \"$TOKEN_ADDR\" 750")
    
    echo -e "${YELLOW}🗳️  Step 4: Deploying Governance...${NC}"
    GOV_ADDR=$(deploy_contract "governance" "\"$TOKEN_ADDR\" \"$REGISTRY_ADDR\" \"$GRID_ADDR\" 100000000000000000000 100 51")
    
    # Create summary file
    cat > deployment/local-addresses.json << JSON
{
  "network": "local",
  "deployer": "//Alice",
  "contracts": {
    "resource_registry": "$REGISTRY_ADDR",
    "powergrid_token": "$TOKEN_ADDR",
    "grid_service": "$GRID_ADDR",
    "governance": "$GOV_ADDR"
  }
}
JSON
    
    echo ""
    echo -e "${GREEN}🎉 PowerGrid Network Local Deployment Complete!${NC}"
    echo ""
    echo -e "${BLUE}📋 Contract Addresses:${NC}"
    echo "  Resource Registry: $REGISTRY_ADDR"
    echo "  PowerGrid Token:   $TOKEN_ADDR"
    echo "  Grid Service:      $GRID_ADDR"
    echo "  Governance:        $GOV_ADDR"
    echo ""
    echo -e "${BLUE}📄 Addresses saved to: deployment/local-addresses.json${NC}"
    echo ""
    echo -e "${YELLOW}🔗 Next Steps:${NC}"
    echo "  1. Test contract interactions: ./scripts/test-interactions.sh"
    echo "  2. View in Contracts UI: https://ui.use.ink/"
    echo "  3. Run full test suite: ./scripts/test-all.sh"
}

main "$@"
