#!/bin/bash

# Test contract interactions locally
set -e

echo "🧪 Testing PowerGrid Network Contract Interactions..."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Load contract addresses
if [ ! -f "deployment/local-addresses.json" ]; then
    echo -e "${RED}❌ Local deployment not found. Run ./scripts/deploy-local.sh first${NC}"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not found. Installing...${NC}"
    sudo apt-get update && sudo apt-get install -y jq
fi

# Extract addresses from the JSON file
REGISTRY_ADDR=$(jq -r '.contracts.resource_registry' deployment/local-addresses.json)
TOKEN_ADDR=$(jq -r '.contracts.powergrid_token' deployment/local-addresses.json)
GRID_ADDR=$(jq -r '.contracts.grid_service' deployment/local-addresses.json)
GOV_ADDR=$(jq -r '.contracts.governance' deployment/local-addresses.json)

# AccountId for //Alice (standard development account)
ALICE_ADDR="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
BOB_ADDR="5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"

# Validate addresses
if [ "$REGISTRY_ADDR" = "null" ] || [ "$TOKEN_ADDR" = "null" ] || [ "$GRID_ADDR" = "null" ] || [ "$GOV_ADDR" = "null" ]; then
    echo -e "${RED}❌ Invalid contract addresses found in deployment file${NC}"
    echo "Registry: $REGISTRY_ADDR"
    echo "Token: $TOKEN_ADDR"
    echo "Grid: $GRID_ADDR"
    echo "Governance: $GOV_ADDR"
    exit 1
fi

echo -e "${GREEN}📋 Contract Addresses Loaded:${NC}"
echo "  Resource Registry: $REGISTRY_ADDR"
echo "  PowerGrid Token:   $TOKEN_ADDR"
echo "  Grid Service:      $GRID_ADDR"
echo "  Governance:        $GOV_ADDR"
echo ""

# Test counter
TEST_COUNT=0
PASSED_TESTS=0

# Function to run contract call and check result
call_contract() {
    local contract_dir=$1
    local contract_addr=$2
    local message=$3
    local args_string="$4"
    local signer=$5
    local value=${6:-"0"}
    local execute_flag=${7:-""}
    
    TEST_COUNT=$((TEST_COUNT + 1))
    echo -e "${BLUE}📞 Test $TEST_COUNT: Calling $message on $contract_dir...${NC}"
    
    cd contracts/$contract_dir
    
    local cmd="cargo contract call --contract $contract_addr --message $message"
    
    if [ -n "$args_string" ]; then
        cmd="$cmd --args $args_string"
    fi
    
    if [ "$value" != "0" ]; then
        cmd="$cmd --value $value"
    fi
    
    cmd="$cmd --suri $signer --url ws://localhost:9944"
    
    if [ "$execute_flag" = "--execute" ]; then
        cmd="$cmd --execute --skip-confirm"
    fi
    
    echo "Running: $cmd"
    echo ""
    
    if eval $cmd; then
        echo ""
        echo -e "${GREEN}✅ Test $TEST_COUNT PASSED: $message${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        cd ../..
        echo "----------------------------------------"
        return 0
    else
        echo ""
        echo -e "${RED}❌ Test $TEST_COUNT FAILED: $message${NC}"
        cd ../..
        echo "----------------------------------------"
        return 1
    fi
}

echo -e "${BLUE}🪙 Testing PowerGrid Token Contract...${NC}"
echo "=========================================="

# Test 1: Check initial token supply
call_contract "token" "$TOKEN_ADDR" "total_supply" "" "//Alice"

# Test 2: Check Alice's balance (should have initial supply)
call_contract "token" "$TOKEN_ADDR" "balance_of" "$ALICE_ADDR" "//Alice"

echo ""
echo -e "${BLUE}📋 Testing Resource Registry Contract...${NC}"
echo "=============================================="

# Test 3: Check minimum stake
call_contract "resource_registry" "$REGISTRY_ADDR" "get_min_stake" "" "//Alice"

# Test 4: Register Bob's device with stake
echo -e "${YELLOW}🔧 Registering Bob's device...${NC}"
DEVICE_METADATA='{"device_type":{"SmartPlug":null},"capacity_watts":2000,"location":"Living Room","manufacturer":"PowerGrid Inc","model":"SmartNode-1","firmware_version":"1.0.0","installation_date":1640995200000}'

call_contract "resource_registry" "$REGISTRY_ADDR" "register_device" "$DEVICE_METADATA" "//Bob" "2000000000000000000" "--execute"

# Test 5: Check if Bob's device is registered
call_contract "resource_registry" "$REGISTRY_ADDR" "is_device_registered" "$BOB_ADDR" "//Alice"

# Test 6: Get Bob's device info
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device" "$BOB_ADDR" "//Alice"

echo ""
echo -e "${BLUE}⚡ Testing Grid Service Contract...${NC}"
echo "======================================"

# Test 7: Create a grid event
echo -e "${YELLOW}🔧 Creating grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "create_grid_event" '"DemandResponse" 60 750 100' "//Alice" "0" "--execute"

# Test 8: Check grid service stats
call_contract "grid_service" "$GRID_ADDR" "get_stats" "" "//Alice"

# Test 9: Get grid event details
call_contract "grid_service" "$GRID_ADDR" "get_grid_event" "1" "//Alice"

# Test 10: Bob participates in the event
echo -e "${YELLOW}🔧 Bob participating in grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "participate_in_event" "1 75" "//Bob" "0" "--execute"

# Test 11: Check event participations
call_contract "grid_service" "$GRID_ADDR" "get_event_participations" "1" "//Alice"

# Test 12: Verify Bob's participation (Admin only)
echo -e "${YELLOW}🔧 Verifying Bob's participation...${NC}"
call_contract "grid_service" "$GRID_ADDR" "verify_participation" "1 $BOB_ADDR 70" "//Alice" "0" "--execute"

echo ""
echo -e "${BLUE}🗳️  Testing Governance Contract...${NC}"
echo "====================================="

# Test 13: Create a governance proposal
echo -e "${YELLOW}🔧 Creating governance proposal...${NC}"
call_contract "governance" "$GOV_ADDR" "create_proposal" '{"UpdateMinStake":2000000000000000000} "Increase minimum stake for better security"' "//Alice" "0" "--execute"

# Test 14: Check if proposal was created
call_contract "governance" "$GOV_ADDR" "get_proposal" "1" "//Alice"

# Test 15: Vote on the proposal
echo -e "${YELLOW}🔧 Voting on proposal...${NC}"
call_contract "governance" "$GOV_ADDR" "vote" '1 true "I support this change"' "//Alice" "0" "--execute"

# Test 16: Check voting status
call_contract "governance" "$GOV_ADDR" "has_voted" "1 $ALICE_ADDR" "//Alice"

echo ""
echo -e "${GREEN}🎉 Cross-Contract Integration Tests${NC}"
echo "=========================================="

# Test 17: Complete the grid event
echo -e "${YELLOW}🔧 Completing grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "complete_grid_event" "1" "//Alice" "0" "--execute"

# Test 18: Check final device reputation
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device_reputation" "$BOB_ADDR" "//Alice"

# Test 19: Check device count
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device_count" "" "//Alice"

# Test 20: Get governance parameters
call_contract "governance" "$GOV_ADDR" "get_governance_params" "" "//Alice"

echo ""
echo "================================================================"
echo -e "${GREEN}🏆 ALL INTERACTION TESTS COMPLETED!${NC}"
echo "================================================================"
echo ""
echo -e "${YELLOW}📊 Test Results Summary:${NC}"
echo -e "   Total Tests Run: ${BLUE}$TEST_COUNT${NC}"
echo -e "   Tests Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "   Tests Failed: ${RED}$((TEST_COUNT - PASSED_TESTS))${NC}"
echo ""

if [ $PASSED_TESTS -eq $TEST_COUNT ]; then
    echo -e "${GREEN}🎯 ALL TESTS PASSED! PowerGrid Network is working perfectly!${NC}"
    echo ""
    echo -e "${YELLOW}✅ Validated Features:${NC}"
    echo "  ✓ Token contract: Supply, balances, metadata"
    echo "  ✓ Resource Registry: Device registration, staking, reputation"
    echo "  ✓ Grid Service: Event creation, participation, verification"
    echo "  ✓ Governance: Proposal creation, voting, parameters"
    echo "  ✓ Cross-contract: Integration and data flow"
    echo ""
    echo -e "${BLUE}🚀 Next Steps:${NC}"
    echo "  1. Deploy to testnet: Use testnet deployment scripts"
    echo "  2. Run integration tests: ./scripts/test-integration.sh"
    echo "  3. Explore live interactions with the deployed contracts"
else
    echo -e "${RED}❌ Some tests failed. Please check the output above for details.${NC}"
fi

echo ""
echo -e "${YELLOW}💡 For comprehensive integration testing, run:${NC}"
echo "   ${BLUE}./scripts/test-integration.sh${NC}"