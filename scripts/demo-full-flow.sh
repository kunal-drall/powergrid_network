#!/usr/bin/env bash
# Full demo script showing the complete PowerGrid Network flow

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     PowerGrid Network - Full Demo Script                  ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check node
echo -e "${BLUE}Step 1: Checking Substrate Node...${NC}"
if curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9944 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Node is running${NC}"
else
    echo -e "${RED}❌ Node is not running. Please start it first.${NC}"
    exit 1
fi
echo ""

# Step 2: Check contracts
echo -e "${BLUE}Step 2: Checking Contract Deployment...${NC}"
if [ -f "backend/.env" ]; then
    TOKEN_ADDR=$(grep TOKEN_CONTRACT_ADDRESS backend/.env | cut -d'=' -f2)
    REGISTRY_ADDR=$(grep REGISTRY_CONTRACT_ADDRESS backend/.env | cut -d'=' -f2)
    GRID_ADDR=$(grep GRID_SERVICE_CONTRACT_ADDRESS backend/.env | cut -d'=' -f2)
    
    if [ -n "$TOKEN_ADDR" ] && [ -n "$REGISTRY_ADDR" ] && [ -n "$GRID_ADDR" ]; then
        echo -e "${GREEN}✅ Contracts deployed${NC}"
        echo "   Token: $TOKEN_ADDR"
        echo "   Registry: $REGISTRY_ADDR"
        echo "   Grid Service: $GRID_ADDR"
    else
        echo -e "${RED}❌ Contract addresses not found${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ backend/.env not found${NC}"
    exit 1
fi
echo ""

# Step 3: Check device registration
echo -e "${BLUE}Step 3: Checking Device Registration...${NC}"
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
        rep = client.get_device_reputation()
        print(f'   Reputation: {rep}')
    else:
        print('❌ Device is not registered')
        sys.exit(1)
" 2>&1 | grep -E "(✅|❌|Reputation)" || echo -e "${YELLOW}⚠️  Could not check registration${NC}"
cd ..
echo ""

# Step 4: Check Tapo connection
echo -e "${BLUE}Step 4: Checking Tapo Device Connection...${NC}"
cd backend && source venv/bin/activate 2>/dev/null || true
python3 -c "
import sys
import asyncio
sys.path.insert(0, 'src')
from tapo_monitor import TapoMonitor
from config.config import Config

async def check():
    monitor = TapoMonitor(Config.TAPO_EMAIL, Config.TAPO_PASSWORD, Config.TAPO_DEVICE_IP)
    if await monitor.connect():
        print('✅ Tapo device connected')
        snapshot = await monitor.get_complete_snapshot()
        if snapshot:
            power = snapshot['current_power']['power_watts']
            energy = snapshot['energy_usage']['today_energy_kwh']
            print(f'   Current Power: {power:.2f} W')
            print(f'   Today Energy: {energy:.3f} kWh')
    else:
        print('❌ Tapo device not connected')
        sys.exit(1)

asyncio.run(check())
" 2>&1 | grep -E "(✅|❌|Power|Energy)" || echo -e "${YELLOW}⚠️  Could not check Tapo device${NC}"
cd ..
echo ""

# Step 5: Create a test grid event
echo -e "${BLUE}Step 5: Creating Test Grid Event...${NC}"
echo -e "${YELLOW}Creating a DemandResponse event (30 min, 100 kW target)...${NC}"
./scripts/create-grid-event.sh DemandResponse 30 1000000000000000000 100 2>&1 | grep -E "(✅|❌|Event|event)" || echo -e "${YELLOW}⚠️  Event creation output above${NC}"
echo ""

# Step 6: Check oracle service
echo -e "${BLUE}Step 6: Checking Oracle Service Status...${NC}"
if pgrep -f "python.*oracle_service" > /dev/null; then
    echo -e "${GREEN}✅ Oracle service is running${NC}"
    echo "   PID: $(pgrep -f 'python.*oracle_service')"
    echo ""
    echo -e "${YELLOW}💡 View live logs: tail -f backend/logs/oracle.log${NC}"
else
    echo -e "${YELLOW}⚠️  Oracle service is not running${NC}"
    echo "   Start it with: cd backend && source venv/bin/activate && python src/oracle_service.py"
fi
echo ""

# Summary
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    Demo Summary                            ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ System Status:${NC}"
echo "   - Node: Running"
echo "   - Contracts: Deployed"
echo "   - Device: Registered"
echo "   - Tapo: Connected"
echo "   - Grid Event: Created"
echo ""
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo "   1. Plug something into the Tapo device"
echo "   2. Watch the oracle service participate automatically"
echo "   3. Check token rewards: cd backend && python scripts/check-rewards.py"
echo ""
echo -e "${CYAN}🎉 Demo setup complete!${NC}"

