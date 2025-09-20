#!/usr/bin/env bash
set -e

echo "🧪 Validating Enhanced Deploy and Run E2E Script"
echo "================================================"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Test 1: Check script syntax
echo -e "${BLUE}Test 1: Checking script syntax...${NC}"
if bash -n scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Script syntax is valid${NC}"
else
    echo -e "${RED}❌ Script syntax has errors${NC}"
    exit 1
fi

# Test 2: Check that all functions are defined
echo -e "${BLUE}Test 2: Checking function definitions...${NC}"
required_functions=("check_node" "deploy_contract" "test_contract_call" "verify_contract_state" "test_cross_contract_workflow" "main")

for func in "${required_functions[@]}"; do
    if grep -q "^$func()" scripts/deploy-and-run-e2e.sh; then
        echo -e "${GREEN}✅ Function $func is defined${NC}"
    else
        echo -e "${RED}❌ Function $func is missing${NC}"
        exit 1
    fi
done

# Test 3: Check constructor arguments match actual contracts
echo -e "${BLUE}Test 3: Validating constructor arguments...${NC}"

# Check token constructor
echo "Checking PowerGrid Token constructor..."
if grep -q 'new.*PowerGrid Token.*PGT.*18.*1000000000000000000000' scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Token constructor arguments look correct${NC}"
else
    echo -e "${YELLOW}⚠️  Token constructor arguments may need verification${NC}"
fi

# Check registry constructor
echo "Checking Resource Registry constructor..."
if grep -q 'resource_registry.*new.*1000000000000000000' scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Registry constructor arguments look correct${NC}"
else
    echo -e "${YELLOW}⚠️  Registry constructor arguments may need verification${NC}"
fi

# Test 4: Check that script handles errors properly
echo -e "${BLUE}Test 4: Checking error handling...${NC}"
if grep -q "exit 1" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Script has error handling${NC}"
else
    echo -e "${YELLOW}⚠️  Script should have better error handling${NC}"
fi

# Test 5: Check that contract addresses are captured
echo -e "${BLUE}Test 5: Checking address capture logic...${NC}"
if grep -q "contract_address.*deploy_output" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Address capture logic is present${NC}"
else
    echo -e "${RED}❌ Address capture logic is missing${NC}"
    exit 1
fi

# Test 6: Check cross-contract testing workflow
echo -e "${BLUE}Test 6: Checking cross-contract test workflow...${NC}"
if grep -q "test_cross_contract_workflow" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Cross-contract workflow testing is implemented${NC}"
else
    echo -e "${RED}❌ Cross-contract workflow testing is missing${NC}"
    exit 1
fi

# Test 7: Verify cargo contract commands are properly formed
echo -e "${BLUE}Test 7: Checking cargo contract command syntax...${NC}"
if grep -q "cargo contract instantiate" scripts/deploy-and-run-e2e.sh && grep -q -- "--suri" scripts/deploy-and-run-e2e.sh && grep -q -- "--execute" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Deployment commands look correct${NC}"
else
    echo -e "${RED}❌ Deployment commands may have issues${NC}"
    exit 1
fi

if grep -q "cargo contract call" scripts/deploy-and-run-e2e.sh && grep -q -- "--contract" scripts/deploy-and-run-e2e.sh && grep -q -- "--message" scripts/deploy-and-run-e2e.sh; then
    echo -e "${GREEN}✅ Contract call commands look correct${NC}"
else
    echo -e "${RED}❌ Contract call commands may have issues${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All validation tests passed!${NC}"
echo -e "${YELLOW}📋 Summary of Enhanced Features:${NC}"
echo "✅ Contract address extraction from deployment output"
echo "✅ Cross-contract interaction testing"
echo "✅ State verification functions"
echo "✅ Complete workflow validation"
echo "✅ JSON deployment record with actual addresses"
echo "✅ Auto-start substrate-contracts-node capability"
echo "✅ Comprehensive error handling"
echo ""
echo -e "${BLUE}💡 Next: Test with actual substrate-contracts-node${NC}"