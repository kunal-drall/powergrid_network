#!/bin/bash

echo "🚀 Deploying PowerGrid Network Locally..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

check_node() {
    echo "🔍 Checking if substrate-contracts-node is running..."
    if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9944 2>/dev/null | grep -q '"result"'; then
        echo -e "${GREEN}✅ substrate-contracts-node is running on port 9944${NC}"
        return 0
    else
        echo -e "${RED}❌ substrate-contracts-node not responding on port 9944${NC}"
        echo -e "${YELLOW}💡 Please start it with: substrate-contracts-node --dev --tmp${NC}"
        exit 1
    fi
}

deploy_contract() {
    local CONTRACT_DIR=$1
    local CONTRACT_NAME=$2
    local CONSTRUCTOR_ARGS=$3
    
    echo -e "${BLUE}🚀 Deploying $CONTRACT_NAME...${NC}"
    
    cd contracts/$CONTRACT_DIR || exit 1
    
    # Build first to ensure we have the latest version
    echo "📦 Building $CONTRACT_NAME..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying $CONTRACT_NAME..."
    OUTPUT=$(cargo contract instantiate \
        --constructor new \
        --args "$CONSTRUCTOR_ARGS" \
        --suri //Alice \
        --url ws://localhost:9944 \
        --execute \
        --gas 1000000000000 \
        --proof-size 1000000 \
        --value 0 2>&1)
    
    echo "$OUTPUT"
    
    # Extract contract address - updated regex for better matching
    ADDRESS=$(echo "$OUTPUT" | grep -oE "Contract [A-Za-z0-9]{48}" | grep -oE "[A-Za-z0-9]{48}" | head -1)
    
    if [ -z "$ADDRESS" ]; then
        echo -e "${RED}❌ Failed to extract contract address for $CONTRACT_NAME${NC}"
        echo "Full output:"
        echo "$OUTPUT"
        cd ../..
        return 1
    fi
    
    echo -e "${GREEN}✅ $CONTRACT_NAME deployed: $ADDRESS${NC}"
    cd ../..
    echo "$ADDRESS"
}

main() {
    # Check if node is running
    check_node
    
    # Create deployment directory
    mkdir -p deployment
    
    echo "📋 Deploying contracts in dependency order..."
    
    # 1. Deploy Token Contract first
    echo -e "${BLUE}Step 1: Deploying PowerGrid Token...${NC}"
    TOKEN_ADDR=$(deploy_contract "token" "PowerGrid Token" '"PowerGrid Token" "PGT" 18 1000000000000000000000')
    if [ $? -ne 0 ] || [ -z "$TOKEN_ADDR" ]; then 
        echo -e "${RED}❌ Token deployment failed${NC}"
        exit 1
    fi
    
    # 2. Deploy Resource Registry
    echo -e "${BLUE}Step 2: Deploying Resource Registry...${NC}"
    REGISTRY_ADDR=$(deploy_contract "resource_registry" "Resource Registry" "1000000000000000000")
    if [ $? -ne 0 ] || [ -z "$REGISTRY_ADDR" ]; then 
        echo -e "${RED}❌ Registry deployment failed${NC}"
        exit 1
    fi
    
    # 3. Deploy Grid Service
    echo -e "${BLUE}Step 3: Deploying Grid Service...${NC}"
    GRID_ADDR=$(deploy_contract "grid_service" "Grid Service" "$TOKEN_ADDR $REGISTRY_ADDR")
    if [ $? -ne 0 ] || [ -z "$GRID_ADDR" ]; then 
        echo -e "${RED}❌ Grid Service deployment failed${NC}"
        exit 1
    fi
    
    # 4. Deploy Governance
    echo -e "${BLUE}Step 4: Deploying Governance...${NC}"
    GOV_ADDR=$(deploy_contract "governance" "Governance" "$TOKEN_ADDR $REGISTRY_ADDR $GRID_ADDR 100000000000000000000 100 51")
    if [ $? -ne 0 ] || [ -z "$GOV_ADDR" ]; then 
        echo -e "${RED}❌ Governance deployment failed${NC}"
        exit 1
    fi
    
    # 5. Create deployment addresses file
cat << EOF > deployment/local-addresses.json
{
  "contracts": {
    "powergrid_token": "$TOKEN_ADDR",
    "resource_registry": "$REGISTRY_ADDR",
    "grid_service": "$GRID_ADDR",
    "governance": "$GOV_ADDR"
  },
  "network": "local",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "//Alice"
}
EOF
