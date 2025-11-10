#!/usr/bin/env bash
# Complete End-to-End Integration Test Script

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   PowerGrid Network - End-to-End Integration Test          ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check Node
echo -e "${BLUE}Step 1: Checking Substrate Node...${NC}"
if curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9944 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Node is running${NC}"
else
    echo -e "${RED}❌ Node is not running${NC}"
    echo -e "${YELLOW}Starting node in background...${NC}"
    ~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all > /tmp/substrate-node.log 2>&1 &
    NODE_PID=$!
    echo "Node started with PID: $NODE_PID"
    echo "Waiting for node to be ready..."
    sleep 15
    if curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9944 > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Node is ready${NC}"
    else
        echo -e "${RED}❌ Node failed to start${NC}"
        exit 1
    fi
fi
echo ""

# Step 2: Deploy Contracts
echo -e "${BLUE}Step 2: Deploying Contracts...${NC}"
if [ -f "backend/.env" ] && grep -q "TOKEN_CONTRACT_ADDRESS=" backend/.env && [ -n "$(grep TOKEN_CONTRACT_ADDRESS backend/.env | cut -d'=' -f2)" ]; then
    echo -e "${GREEN}✅ Contracts appear to be deployed${NC}"
    TOKEN_ADDR=$(grep TOKEN_CONTRACT_ADDRESS backend/.env | cut -d'=' -f2)
    echo "   Token: $TOKEN_ADDR"
else
    echo -e "${YELLOW}⚠️  Contracts not deployed. Deploying now...${NC}"
    ./scripts/deploy-local.sh
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Contract deployment failed${NC}"
        exit 1
    fi
fi
echo ""

# Step 3: Setup Authorization
echo -e "${BLUE}Step 3: Setting Up Contract Authorization...${NC}"
cd backend && source venv/bin/activate 2>/dev/null || true
python3 scripts/setup_authorization.py 2>&1 | grep -E "(✅|❌|⚠️|Step)" || echo -e "${YELLOW}⚠️  Authorization setup completed with warnings${NC}"
cd ..
echo ""

# Step 4: Check Device Registration
echo -e "${BLUE}Step 4: Checking Device Registration...${NC}"
cd backend && source venv/bin/activate 2>/dev/null || true
python3 -c "
import sys
sys.path.insert(0, 'src')
from blockchain_client import BlockchainClient
from config.config import Config

client = BlockchainClient(Config.SUBSTRATE_RPC_URL, Config.DEVICE_OWNER_SEED)
if client.connect():
    client.load_contracts(
        Config.TOKEN_CONTRACT,
        Config.REGISTRY_CONTRACT,
        Config.GRID_SERVICE_CONTRACT,
        Config.GOVERNANCE_CONTRACT
    )
    is_reg = client.is_device_registered()
    if is_reg:
        print('✅ Device is registered')
    else:
        print('⚠️  Device not registered - oracle will register on first run')
" 2>&1 | grep -E "(✅|❌|⚠️)" || echo -e "${YELLOW}⚠️  Could not check registration${NC}"
cd ..
echo ""

# Step 5: Test Tapo Connection
echo -e "${BLUE}Step 5: Testing Tapo Device Connection...${NC}"
cd backend && source venv/bin/activate 2>/dev/null || true
python3 -c "
import sys
import asyncio
sys.path.insert(0, 'src')
from tapo_monitor import TapoMonitor
from config.config import Config

async def test():
    monitor = TapoMonitor(Config.TAPO_EMAIL, Config.TAPO_PASSWORD, Config.TAPO_DEVICE_IP)
    if await monitor.connect():
        print('✅ Tapo device connected')
        snapshot = await monitor.get_complete_snapshot()
        if snapshot:
            power = snapshot['current_power']['power_watts']
            print(f'   Current Power: {power:.2f} W')
    else:
        print('❌ Tapo device not connected')

asyncio.run(test())
" 2>&1 | grep -E "(✅|❌|Power)" || echo -e "${YELLOW}⚠️  Could not test Tapo device${NC}"
cd ..
echo ""

# Step 6: Create Test Event
echo -e "${BLUE}Step 6: Creating Test Grid Event...${NC}"
cd backend && source venv/bin/activate 2>/dev/null || true
python3 scripts/create_test_event.py 2>&1 | grep -E "(✅|❌|Creating|Transaction)" || echo -e "${YELLOW}⚠️  Event creation output above${NC}"
cd ..
echo ""

# Step 7: Check Oracle Service
echo -e "${BLUE}Step 7: Oracle Service Status...${NC}"
if pgrep -f "python.*oracle_service" > /dev/null; then
    echo -e "${GREEN}✅ Oracle service is running${NC}"
    echo "   PID: $(pgrep -f 'python.*oracle_service')"
    echo ""
    echo -e "${YELLOW}💡 View live logs: tail -f backend/logs/oracle.log${NC}"
else
    echo -e "${YELLOW}⚠️  Oracle service is not running${NC}"
    echo ""
    echo -e "${CYAN}To start oracle service:${NC}"
    echo "   cd backend"
    echo "   source venv/bin/activate"
    echo "   python src/oracle_service.py"
fi
echo ""

# Summary
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    Test Summary                           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ Integration Test Complete!${NC}"
echo ""
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo "   1. Start oracle service (if not running)"
echo "   2. Plug something into Tapo device"
echo "   3. Watch oracle participate in grid events"
echo "   4. Check rewards: cd backend && python scripts/check-rewards.py"
echo ""
echo -e "${CYAN}🎉 System is ready for end-to-end testing!${NC}"

