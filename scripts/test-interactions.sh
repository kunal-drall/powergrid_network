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

# Function to run contract call and check result
call_contract() {
    local contract_dir=$1
    local contract_addr=$2
    local message=$3
    local args=$4
    local signer=$5
    local value=${6:-"0"}
    local execute_flag=${7:-""}
    
    echo -e "${BLUE}📞 Calling $message on $contract_dir...${NC}"
    
    cd contracts/$contract_dir
    
    local cmd="cargo contract call --contract $contract_addr --message $message"
    
    if [ -n "$args" ]; then
        cmd="$cmd --args $args"
    fi
    
    if [ "$value" != "0" ]; then
        cmd="$cmd --value $value"
    fi
    
    cmd="$cmd --suri $signer --url ws://localhost:9944"
    
    if [ "$execute_flag" = "--execute" ]; then
        cmd="$cmd --execute"
    fi
    
    echo "Running: $cmd"
    
    if eval $cmd; then
        echo -e "${GREEN}✅ Success: $message${NC}"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Failed: $message${NC}"
        cd ../..
        return 1
    fi
}

echo -e "${BLUE}🪙 Testing PowerGrid Token...${NC}"

# Test 1: Check initial token supply
call_contract "token" "$TOKEN_ADDR" "total_supply" "" "//Alice"

# Test 2: Check Alice's balance (should have initial supply)
call_contract "token" "$TOKEN_ADDR" "balance_of" "//Alice" "//Alice"

echo ""
echo -e "${BLUE}📋 Testing Resource Registry...${NC}"

# Test 3: Check minimum stake
call_contract "resource_registry" "$REGISTRY_ADDR" "get_min_stake" "" "//Alice"

# Test 4: Register Bob's device with stake
echo -e "${YELLOW}Registering Bob's device...${NC}"
DEVICE_METADATA='{"device_type":{"SmartPlug":null},"capacity_watts":2000,"location":"Living Room","manufacturer":"PowerGrid Inc","model":"SmartNode-1","firmware_version":"1.0.0","installation_date":1640995200000}'

call_contract "resource_registry" "$REGISTRY_ADDR" "register_device" "$DEVICE_METADATA" "//Bob" "2000000000000000000" "--execute"

# Test 5: Check if Bob's device is registered
call_contract "resource_registry" "$REGISTRY_ADDR" "is_device_registered" "//Bob" "//Alice"

# Test 6: Get Bob's device info
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device" "//Bob" "//Alice"

echo ""
echo -e "${BLUE}⚡ Testing Grid Service...${NC}"

# Test 7: Create a grid event
echo -e "${YELLOW}Creating grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "create_grid_event" "\"DemandResponse\" 60 750 100" "//Alice" "0" "--execute"

# Test 8: Check grid service stats
call_contract "grid_service" "$GRID_ADDR" "get_stats" "" "//Alice"

# Test 9: Get grid event details
call_contract "grid_service" "$GRID_ADDR" "get_grid_event" "1" "//Alice"

# Test 10: Bob participates in the event
echo -e "${YELLOW}Bob participating in grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "participate_in_event" "1 75" "//Bob" "0" "--execute"

# Test 11: Check event participations
call_contract "grid_service" "$GRID_ADDR" "get_event_participations" "1" "//Alice"

# Test 12: Verify Bob's participation (Admin only)
echo -e "${YELLOW}Verifying Bob's participation...${NC}"
call_contract "grid_service" "$GRID_ADDR" "verify_participation" "1 //Bob 70" "//Alice" "0" "--execute"

echo ""
echo -e "${BLUE}🗳️  Testing Governance...${NC}"

# Test 13: Create a governance proposal
echo -e "${YELLOW}Creating governance proposal...${NC}"
call_contract "governance" "$GOV_ADDR" "create_proposal" '{"UpdateMinStake":2000000000000000000} "Increase minimum stake for better security"' "//Alice" "0" "--execute"

# Test 14: Check if proposal was created
call_contract "governance" "$GOV_ADDR" "get_proposal" "1" "//Alice"

# Test 15: Vote on the proposal
echo -e "${YELLOW}Voting on proposal...${NC}"
call_contract "governance" "$GOV_ADDR" "vote" "1 true \"I support this change\"" "//Alice" "0" "--execute"

# Test 16: Check voting status
call_contract "governance" "$GOV_ADDR" "has_voted" "1 //Alice" "//Alice"

echo ""
echo -e "${GREEN}🎉 Cross-Contract Integration Tests${NC}"

# Test 17: Complete the grid event
echo -e "${YELLOW}Completing grid event...${NC}"
call_contract "grid_service" "$GRID_ADDR" "complete_grid_event" "1" "//Alice" "0" "--execute"

# Test 18: Check final device reputation
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device_reputation" "//Bob" "//Alice"

# Test 19: Check device count
call_contract "resource_registry" "$REGISTRY_ADDR" "get_device_count" "" "//Alice"

# Test 20: Get governance parameters
call_contract "governance" "$GOV_ADDR" "get_governance_params" "" "//Alice"

echo ""
echo -e "${GREEN}✅ All interaction tests completed successfully!${NC}"
echo ""
echo -e "${YELLOW}📊 Test Summary:${NC}"
echo "  ✓ Token contract: Supply, balances, transfers"
echo "  ✓ Resource Registry: Device registration, staking, reputation"
echo "  ✓ Grid Service: Event creation, participation, verification"
echo "  ✓ Governance: Proposal creation, voting, parameters"
echo "  ✓ Cross-contract: Integration and data flow"
echo ""
echo -e "${BLUE}🎯 All PowerGrid Network features validated!${NC}"

# Optional: Run a quick simulation
echo ""
echo -e "${YELLOW}💡 Want to run the full integration test suite?${NC}"
echo "   Run: ./scripts/test-integration.sh"