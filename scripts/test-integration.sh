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
        "test_cross_contract_reward_distribution"
        "test_cross_contract_device_verification"
        "test_governance_execution_affects_contracts"
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
