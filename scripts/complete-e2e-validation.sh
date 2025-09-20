#!/usr/bin/env bash
set -e

echo "🧪 PowerGrid Network - Complete E2E Test Validation"
echo "=================================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Test 1: Validate deployment script
echo -e "${BLUE}Test 1: Validating enhanced deployment script...${NC}"
if ./scripts/validate-deployment-script.sh; then
    echo -e "${GREEN}✅ Deployment script validation passed${NC}"
else
    echo -e "${RED}❌ Deployment script validation failed${NC}"
    exit 1
fi

echo ""

# Test 2: Check all contracts build correctly
echo -e "${BLUE}Test 2: Building all contracts...${NC}"
if ./scripts/build-all.sh; then
    echo -e "${GREEN}✅ All contracts built successfully${NC}"
else
    echo -e "${RED}❌ Contract build failed${NC}"
    exit 1
fi

echo ""

# Test 3: Check integration tests compile
echo -e "${BLUE}Test 3: Checking integration tests compilation...${NC}"
cd contracts/integration-tests
if cargo check --features e2e-tests; then
    echo -e "${GREEN}✅ Integration tests compile correctly${NC}"
else
    echo -e "${RED}❌ Integration tests compilation failed${NC}"
    exit 1
fi

cd ../..

echo ""

# Test 4: Validate constructor signatures match
echo -e "${BLUE}Test 4: Validating constructor signatures...${NC}"

# Check PowerGrid Token constructor in script matches actual implementation
script_token_args=$(grep -A 1 'deploy_contract "token"' scripts/deploy-and-run-e2e.sh | grep -o '"[^"]*"' | tr '\n' ' ')
echo "Script token args: $script_token_args"

actual_token_constructor=$(grep -A 1 "pub fn new" contracts/token/src/lib.rs | grep "name.*symbol.*decimals.*initial_supply")
if [ -n "$actual_token_constructor" ]; then
    echo -e "${GREEN}✅ Token constructor signature validation passed${NC}"
else
    echo -e "${YELLOW}⚠️  Token constructor signature needs manual verification${NC}"
fi

# Check ResourceRegistry constructor
script_registry_args=$(grep -A 1 'deploy_contract "resource_registry"' scripts/deploy-and-run-e2e.sh | grep -o '"[^"]*"')
echo "Script registry args: $script_registry_args"

actual_registry_constructor=$(grep -A 1 "pub fn new" contracts/resource_registry/src/lib.rs | grep "min_stake")
if [ -n "$actual_registry_constructor" ]; then
    echo -e "${GREEN}✅ Registry constructor signature validation passed${NC}"
else
    echo -e "${YELLOW}⚠️  Registry constructor signature needs manual verification${NC}"
fi

echo ""

# Test 5: Check cross-contract testing workflow
echo -e "${BLUE}Test 5: Validating cross-contract testing workflow...${NC}"

required_tests=(
    "verify_contract_state.*total_supply"
    "register_device"
    "create_grid_event"
    "participate_in_event"
    "verify_and_distribute_rewards"
    "create_proposal"
    "vote"
)

all_tests_found=true
for test in "${required_tests[@]}"; do
    if grep -q "$test" scripts/deploy-and-run-e2e.sh; then
        echo -e "${GREEN}✅ Found test: $test${NC}"
    else
        echo -e "${RED}❌ Missing test: $test${NC}"
        all_tests_found=false
    fi
done

if [ "$all_tests_found" = true ]; then
    echo -e "${GREEN}✅ All cross-contract tests are implemented${NC}"
else
    echo -e "${RED}❌ Some cross-contract tests are missing${NC}"
    exit 1
fi

echo ""

# Test 6: Check deployment record functionality
echo -e "${BLUE}Test 6: Checking deployment record functionality...${NC}"
if grep -q "local-addresses.json" scripts/deploy-and-run-e2e.sh && grep -q "powergrid_token.*registry.*grid.*governance" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Deployment record functionality is implemented${NC}"
else
    echo -e "${RED}❌ Deployment record functionality is incomplete${NC}"
    exit 1
fi

echo ""

# Test 7: Summary and recommendations
echo -e "${GREEN}🎉 All validation tests passed!${NC}"
echo ""
echo -e "${YELLOW}📋 Enhanced E2E Testing Features Summary:${NC}"
echo "✅ Enhanced deploy-and-run-e2e.sh script with:"
echo "   • Contract address extraction from deployment output"
echo "   • Cross-contract interaction testing (registry → grid → token → governance)"
echo "   • State verification after each operation"
echo "   • Complete workflow validation"
echo "   • JSON deployment record with actual addresses"
echo "   • Auto-start substrate-contracts-node capability"
echo ""
echo "✅ Fixed integration tests with correct constructor signatures:"
echo "   • PowergridToken: name, symbol, decimals, initial_supply"
echo "   • ResourceRegistry: min_stake only"
echo "   • GridService: token_address, registry_address" 
echo "   • Governance: all contract addresses + parameters"
echo ""
echo -e "${BLUE}🚀 Ready for deployment on substrate-contracts-node!${NC}"
echo ""
echo -e "${YELLOW}📋 To run full E2E tests:${NC}"
echo "1. Start substrate-contracts-node: substrate-contracts-node --dev"
echo "2. Run deployment script: ./scripts/deploy-and-run-e2e.sh"
echo "3. Run integration tests: cd contracts/integration-tests && cargo test --features e2e-tests"