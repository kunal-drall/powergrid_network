#!/usr/bin/env bash
set -e

echo "🔧 Setting up PowerGrid Network Integration Tests..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Make all scripts executable
echo -e "${BLUE}📋 Making scripts executable...${NC}"
chmod +x scripts/*.sh

echo -e "${GREEN}✅ Scripts are now executable${NC}"

# Update integration tests files
echo -e "${BLUE}📝 Updating integration test files...${NC}"

# Update integration tests Cargo.toml
cat > contracts/integration-tests/Cargo.toml << 'EOF'
[package]
name = "integration-tests"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ink.workspace = true
scale.workspace = true
scale-info.workspace = true
powergrid-shared.workspace = true

# Include all contract dependencies for integration testing
powergrid_token = { path = "../token", default-features = false }
resource_registry = { path = "../resource_registry", default-features = false }
grid_service = { path = "../grid_service", default-features = false }
governance = { path = "../governance", default-features = false }

[features]
default = ["std"]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
    "powergrid-shared/std",
    "powergrid_token/std",
    "resource_registry/std", 
    "grid_service/std",
    "governance/std",
]
EOF

echo -e "${GREEN}✅ Integration tests Cargo.toml updated${NC}"

# Create the integration test script
cat > scripts/test-integration.sh << 'EOF'
#!/usr/bin/env bash
set -e

echo "🧪 Running PowerGrid Network Integration Tests..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
INTEGRATION_TESTS_DIR="$ROOT_DIR/contracts/integration-tests"

echo -e "${BLUE}📍 Testing from: $INTEGRATION_TESTS_DIR${NC}"

# Function to run a specific test with detailed output
run_test() {
    local test_name=$1
    echo -e "${BLUE}🚀 Running: $test_name${NC}"
    
    cd "$INTEGRATION_TESTS_DIR"
    
    if cargo test "$test_name" --verbose 2>&1; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        return 1
    fi
}

# Function to validate milestone completion
validate_milestone() {
    echo -e "${YELLOW}📊 Validating Milestone 1 Completion...${NC}"
    
    local tests=(
        "test_complete_user_journey"
        "test_cross_contract_reputation"
        "test_governance_parameter_updates"
        "test_reward_distribution_flow"
        "test_cross_contract_error_handling"
        "test_multiple_grid_events"
    )
    
    local passed=0
    local total=${#tests[@]}
    
    echo -e "${BLUE}Running ${total} integration tests...${NC}"
    echo ""
    
    for test in "${tests[@]}"; do
        if run_test "$test"; then
            ((passed++))
        fi
        echo ""
    done
    
    echo -e "${BLUE}📈 Test Results Summary:${NC}"
    echo -e "   Total Tests: $total"
    echo -e "   Passed: ${GREEN}$passed${NC}"
    echo -e "   Failed: ${RED}$((total - passed))${NC}"
    echo ""
    
    if [ $passed -eq $total ]; then
        echo -e "${GREEN}🎉 ALL INTEGRATION TESTS PASSED!${NC}"
        echo -e "${GREEN}✅ Milestone 1 Requirements Validated:${NC}"
        echo "   ✓ Device registration and staking"
        echo "   ✓ Grid event creation and participation"
        echo "   ✓ Reward calculation and distribution"
        echo "   ✓ Cross-contract interactions"
        echo "   ✓ Governance proposal and voting"
        echo "   ✓ Error handling and edge cases"
        echo ""
        echo -e "${YELLOW}🏆 MILESTONE 1 COMPLETE!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some integration tests failed. Milestone not complete.${NC}"
        return 1
    fi
}

# Function to run unit tests for all contracts first
run_unit_tests() {
    echo -e "${BLUE}🔧 Running unit tests for all contracts first...${NC}"
    
    local contracts=("token" "resource_registry" "grid_service" "governance")
    
    for contract in "${contracts[@]}"; do
        echo -e "${BLUE}Testing contract: $contract${NC}"
        cd "$ROOT_DIR/contracts/$contract"
        
        if cargo test --verbose; then
            echo -e "${GREEN}✅ Unit tests passed: $contract${NC}"
        else
            echo -e "${RED}❌ Unit tests failed: $contract${NC}"
            echo "Cannot proceed with integration tests until unit tests pass."
            exit 1
        fi
        echo ""
    done
    
    echo -e "${GREEN}✅ All unit tests passed!${NC}"
    echo ""
}

# Function to build all contracts
build_contracts() {
    echo -e "${BLUE}🔨 Building all contracts...${NC}"
    
    cd "$ROOT_DIR"
    
    if ./scripts/build-all.sh; then
        echo -e "${GREEN}✅ All contracts built successfully!${NC}"
    else
        echo -e "${RED}❌ Contract build failed!${NC}"
        exit 1
    fi
    echo ""
}

# Main execution
main() {
    echo -e "${YELLOW}🚀 Starting comprehensive test validation...${NC}"
    echo ""
    
    # Step 1: Build all contracts
    build_contracts
    
    # Step 2: Run unit tests
    run_unit_tests
    
    # Step 3: Run integration tests and validate milestone
    validate_milestone
    
    if [ $? -eq 0 ]; then
        echo ""
        echo -e "${GREEN}🎯 All tests completed successfully!${NC}"
        echo -e "${YELLOW}📝 Next steps:${NC}"
        echo "   1. Deploy locally: ./scripts/deploy-local.sh"
        echo "   2. Test interactions: ./scripts/test-interactions.sh"
        echo "   3. Deploy to testnet for live validation"
        echo ""
        exit 0
    else
        echo ""
        echo -e "${RED}❌ Integration tests failed. Please fix issues before proceeding.${NC}"
        exit 1
    fi
}

# Run specific test if provided
if [ $# -eq 1 ]; then
    echo -e "${BLUE}Running specific test: $1${NC}"
    run_test "$1"
else
    main
fi
EOF

chmod +x scripts/test-integration.sh
echo -e "${GREEN}✅ Integration test script created${NC}"

# Update the deploy script
cat > scripts/deploy-local.sh << 'EOF'
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
    cat > deployment/local-addresses.json << EOF
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
    
    echo ""
    echo -e "${GREEN}🎉 All contracts deployed successfully!${NC}"
    echo "📄 Contract addresses saved to: deployment/local-addresses.json"
    echo ""
    echo "📋 Deployment Summary:"
    echo -e "  ${BLUE}PowerGrid Token:${NC}    $TOKEN_ADDR"
    echo -e "  ${BLUE}Resource Registry:${NC}  $REGISTRY_ADDR"
    echo -e "  ${BLUE}Grid Service:${NC}       $GRID_ADDR"
    echo -e "  ${BLUE}Governance:${NC}         $GOV_ADDR"
    echo ""
    echo -e "${YELLOW}💡 Next steps:${NC}"
    echo "   1. Run: ./scripts/test-interactions.sh"
    echo "   2. Or manually test with the deployed contracts"
}

main "$@"
EOF

chmod +x scripts/deploy-local.sh
echo -e "${GREEN}✅ Deployment script updated${NC}"

echo ""
echo -e "${GREEN}🎉 Integration test setup complete!${NC}"
echo ""
echo -e "${YELLOW}📝 Next steps:${NC}"
echo "1. Run the comprehensive integration tests:"
echo "   ${BLUE}./scripts/test-integration.sh${NC}"
echo ""
echo "2. Or run step by step:"
echo "   ${BLUE}./scripts/build-all.sh${NC}"
echo "   ${BLUE}./scripts/test-all.sh${NC}"
echo "   ${BLUE}substrate-contracts-node --dev --tmp${NC}  (in separate terminal)"
echo "   ${BLUE}./scripts/deploy-local.sh${NC}"
echo "   ${BLUE}./scripts/test-interactions.sh${NC}"
echo ""
echo -e "${GREEN}✅ All scripts are now ready to run!${NC}"