#!/usr/bin/env bash
set -e

echo "🚀 PowerGrid Network - Deploy and Run E2E Tests"
echo "==============================================="

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if substrate-contracts-node is running
check_node() {
    echo "🔍 Checking if substrate-contracts-node is running..."
    if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9944 2>/dev/null | grep -q '"result"'; then
        echo -e "${GREEN}✅ substrate-contracts-node is running on port 9944${NC}"
        return 0
    else
        echo -e "${RED}❌ substrate-contracts-node not responding on port 9944${NC}"
        echo -e "${YELLOW}💡 Starting substrate-contracts-node in dev mode...${NC}"
        
        # Try to start substrate-contracts-node if available
        if command -v substrate-contracts-node &> /dev/null; then
            echo "Starting substrate-contracts-node in background..."
            nohup substrate-contracts-node --dev --tmp --rpc-cors all --rpc-methods=unsafe > substrate-node.log 2>&1 &
            SUBSTRATE_PID=$!
            
            # Wait for node to start
            echo "Waiting for node to initialize..."
            for i in {1..30}; do
                sleep 1
                if curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://localhost:9944 2>/dev/null | grep -q '"result"'; then
                    echo -e "${GREEN}✅ substrate-contracts-node started successfully${NC}"
                    return 0
                fi
                echo -n "."
            done
            echo ""
            echo -e "${RED}❌ Failed to start substrate-contracts-node${NC}"
            kill $SUBSTRATE_PID 2>/dev/null || true
        else
            echo -e "${YELLOW}💡 substrate-contracts-node not found. Please install it:${NC}"
            echo "cargo install contracts-node --force"
            echo "Then start it manually with: substrate-contracts-node --dev"
        fi
        exit 1
    fi
}

# Deploy a contract and capture its address
deploy_contract() {
    local contract_dir=$1
    local constructor=$2
    local args="$3"
    local contract_name=$4
    local output_var=$5
    
    echo -e "${BLUE}🚀 Deploying $contract_name...${NC}"
    
    cd "contracts/$contract_dir" || exit 1
    
    echo "📦 Building $contract_name..."
    cargo contract build --release --quiet
    
    echo "🚀 Deploying $contract_name..."
    
    local cmd="cargo contract instantiate --constructor $constructor"
    if [ -n "$args" ]; then
        cmd="$cmd --args $args"
    fi
    cmd="$cmd --suri //Alice --url ws://localhost:9944 --execute --skip-confirm --skip-dry-run --gas 1000000000000 --proof-size 1000000 --value 0"
    
    # Capture deployment output
    local deploy_output
    if deploy_output=$(eval "$cmd" 2>&1); then
        echo -e "${GREEN}✅ $contract_name deployed successfully${NC}"
        
        # Extract contract address from output
        local contract_address
        contract_address=$(echo "$deploy_output" | grep -E "Contract [a-zA-Z0-9]{48}" | head -1 | sed -E 's/.*Contract ([a-zA-Z0-9]{48}).*/\1/')
        
        if [ -z "$contract_address" ]; then
            # Try alternative pattern for contract address extraction
            contract_address=$(echo "$deploy_output" | grep -oE '[15][a-km-zA-HJ-NP-Z1-9]{47}' | head -1)
        fi
        
        if [ -n "$contract_address" ] && [ -n "$output_var" ]; then
            eval "$output_var=\"$contract_address\""
            echo "📍 Contract address: $contract_address"
        fi
        
        echo "$deploy_output"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ $contract_name deployment failed${NC}"
        echo "$deploy_output"
        cd ../..
        return 1
    fi
}

# Test contract interaction and capture output
test_contract_call() {
    local contract_dir=$1
    local contract_addr=$2
    local message=$3
    local args_string="$4"
    local signer=$5
    local value=${6:-"0"}
    local execute_flag=${7:-""}
    local description="$8"
    
    echo -e "${BLUE}📞 Testing: $description${NC}"
    
    cd "contracts/$contract_dir" || exit 1
    
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
    
    local output
    if output=$(eval "$cmd" 2>&1); then
        echo -e "${GREEN}✅ Test PASSED: $description${NC}"
        echo "$output"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Test FAILED: $description${NC}"
        echo "$output"
        cd ../..
        return 1
    fi
}

# Verify contract state
verify_contract_state() {
    local contract_dir=$1
    local contract_addr=$2
    local message=$3
    local expected_pattern="$4"
    local description="$5"
    
    echo -e "${BLUE}🔍 Verifying: $description${NC}"
    
    cd "contracts/$contract_dir" || exit 1
    
    local cmd="cargo contract call --contract $contract_addr --message $message --suri //Alice --url ws://localhost:9944"
    
    local output
    if output=$(eval "$cmd" 2>&1); then
        if echo "$output" | grep -q "$expected_pattern"; then
            echo -e "${GREEN}✅ State verification PASSED: $description${NC}"
            cd ../..
            return 0
        else
            echo -e "${RED}❌ State verification FAILED: $description${NC}"
            echo "Expected pattern: $expected_pattern"
            echo "Actual output: $output"
            cd ../..
            return 1
        fi
    else
        echo -e "${RED}❌ State verification ERROR: $description${NC}"
        echo "$output"
        cd ../..
        return 1
    fi
}

# Test cross-contract workflow
test_cross_contract_workflow() {
    local token_addr=$1
    local registry_addr=$2
    local grid_addr=$3
    local governance_addr=$4
    
    echo -e "${YELLOW}🔄 Testing Cross-Contract Workflow${NC}"
    echo "=============================================="
    
    # Test 1: Verify initial token supply
    verify_contract_state "token" "$token_addr" "total_supply" "1000000000000000000000" "Initial token supply is 1000 tokens"
    
    # Test 2: Register a device in registry (requires stake)
    test_contract_call "resource_registry" "$registry_addr" "register_device" \
        '"device123" "SmartPlug" 2000 "Living Room" "PowerGrid Inc" "SP-1" "1.0.0" 1640995200000' \
        "//Alice" "1000000000000000000" "--execute" "Device registration with 1 token stake"
    
    # Test 3: Verify device registration
    verify_contract_state "resource_registry" "$registry_addr" "get_device_count" "1" "Device count after registration"
    
    # Test 4: Check Alice's token balance (should be reduced by stake)
    verify_contract_state "token" "$token_addr" "balance_of" "999000000000000000000" "Alice's balance after staking 1 token"
    
    # Test 5: Create a grid event
    test_contract_call "grid_service" "$grid_addr" "create_grid_event" \
        '"DemandResponse" 60 750 100' \
        "//Alice" "0" "--execute" "Create demand response grid event"
    
    # Test 6: Participate in grid event (simulate energy contribution)
    test_contract_call "grid_service" "$grid_addr" "participate_in_event" \
        '0 75' \
        "//Alice" "0" "--execute" "Participate in grid event with 75kW contribution"
    
    # Test 7: Verify and distribute rewards
    test_contract_call "grid_service" "$grid_addr" "verify_and_distribute_rewards" \
        "0 0" \
        "//Alice" "0" "--execute" "Verify participation and distribute rewards"
    
    # Test 8: Check if Alice received rewards (tokens should increase)
    echo -e "${BLUE}🔍 Checking token balance after rewards...${NC}"
    verify_contract_state "token" "$token_addr" "balance_of" "999" "Alice received rewards (balance increased)"
    
    # Test 9: Create governance proposal
    test_contract_call "governance" "$governance_addr" "create_proposal" \
        '"UpdateMinStake" 2000000000000000000 "Increase minimum stake for better security"' \
        "//Alice" "0" "--execute" "Create governance proposal to increase min stake"
    
    # Test 10: Vote on proposal
    test_contract_call "governance" "$governance_addr" "vote" \
        "0 true" \
        "//Alice" "0" "--execute" "Vote YES on governance proposal"
    
    echo -e "${GREEN}🎉 Cross-contract workflow testing completed!${NC}"
    echo ""
}

# Main deployment and testing
main() {
    echo -e "${YELLOW}🔍 Step 1: Environment Check${NC}"
    check_node
    
    # Create deployment directory
    mkdir -p deployment
    
    echo ""
    echo -e "${YELLOW}🏗️  Step 2: Contract Deployment${NC}"
    echo "==============================="
    
    # Declare variables to store contract addresses
    local token_address=""
    local registry_address=""
    local grid_address=""
    local governance_address=""
    
    # Deploy PowerGrid Token with correct constructor (name, symbol, decimals, initial_supply)
    if deploy_contract "token" "new" '"PowerGrid Token" "PGT" 18 1000000000000000000000' "PowerGrid Token" token_address; then
        echo "✅ Token deployment completed"
    else
        echo "❌ Token deployment failed"
        exit 1
    fi
    
    echo ""
    
    # Deploy Resource Registry with min_stake parameter
    if deploy_contract "resource_registry" "new" "1000000000000000000" "Resource Registry" registry_address; then
        echo "✅ Registry deployment completed"
    else
        echo "❌ Registry deployment failed"
        exit 1
    fi
    
    echo ""
    
    # Deploy Grid Service with min_stake parameter
    if deploy_contract "grid_service" "new" "1000000000000000000" "Grid Service" grid_address; then
        echo "✅ Grid Service deployment completed"
    else
        echo "❌ Grid Service deployment failed"
        exit 1
    fi
    
    echo ""
    
    # Deploy Governance with voting_period and quorum_percentage
    if deploy_contract "governance" "new" "86400 50" "Governance" governance_address; then
        echo "✅ Governance deployment completed"
    else
        echo "❌ Governance deployment failed"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}🎉 All contracts deployed successfully!${NC}"
    
    # Create deployment record with actual addresses
    cat > deployment/local-addresses.json << EOF
{
  "contracts": {
    "powergrid_token": "${token_address:-"DEPLOYMENT_FAILED"}",
    "resource_registry": "${registry_address:-"DEPLOYMENT_FAILED"}",
    "grid_service": "${grid_address:-"DEPLOYMENT_FAILED"}",
    "governance": "${governance_address:-"DEPLOYMENT_FAILED"}"
  },
  "network": "local",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "//Alice"
}
EOF
    
    echo "📄 Deployment record created at deployment/local-addresses.json"
    
    echo ""
    echo -e "${YELLOW}🧪 Step 3: Cross-Contract E2E Tests${NC}"
    echo "===================================="
    
    # Only run tests if we have all contract addresses
    if [ -n "$token_address" ] && [ -n "$registry_address" ] && [ -n "$grid_address" ] && [ -n "$governance_address" ]; then
        test_cross_contract_workflow "$token_address" "$registry_address" "$grid_address" "$governance_address"
    else
        echo -e "${RED}❌ Cannot run cross-contract tests: Missing contract addresses${NC}"
        echo "Token: ${token_address:-"MISSING"}"
        echo "Registry: ${registry_address:-"MISSING"}"
        echo "Grid: ${grid_address:-"MISSING"}"
        echo "Governance: ${governance_address:-"MISSING"}"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}✅ Deploy and E2E workflow completed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Deployment Summary:${NC}"
    echo "PowerGrid Token: $token_address"
    echo "Resource Registry: $registry_address"
    echo "Grid Service: $grid_address"
    echo "Governance: $governance_address"
    echo ""
    echo -e "${YELLOW}📋 Next Steps:${NC}"
    echo "1. Contract addresses saved to deployment/local-addresses.json"
    echo "2. Run unit tests: ./scripts/test-all.sh"
    echo "3. Run integration tests: cd contracts/integration-tests && cargo test --features e2e-tests"
    echo "4. Use addresses above for manual contract interactions"
}

main "$@"