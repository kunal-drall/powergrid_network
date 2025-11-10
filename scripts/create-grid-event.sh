#!/usr/bin/env bash
# Script to create a test grid event

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Default values
EVENT_TYPE="${1:-DemandResponse}"
DURATION_MINUTES="${2:-30}"
COMPENSATION_RATE="${3:-1000000000000000000}"  # 1 token per kWh in wei
TARGET_REDUCTION_KW="${4:-100}"  # 100 kW reduction target

WS_URL="${WS_URL:-ws://localhost:9944}"
SURI="${SURI:-//Alice}"

echo -e "${BLUE}🚀 Creating Grid Event...${NC}"
echo ""
echo "Event Type: $EVENT_TYPE"
echo "Duration: $DURATION_MINUTES minutes"
echo "Compensation Rate: $COMPENSATION_RATE wei"
echo "Target Reduction: $TARGET_REDUCTION_KW kW"
echo ""

# Load contract address from .env or use default
if [ -f "../backend/.env" ]; then
    GRID_ADDR=$(grep GRID_SERVICE_CONTRACT_ADDRESS ../backend/.env | cut -d'=' -f2)
else
    echo -e "${YELLOW}⚠️  backend/.env not found, using default address${NC}"
    GRID_ADDR="5DW1GhTM696DH4vS5n2zj7L6kFG6t4MVipaEnPKygj48TUtX"
fi

if [ -z "$GRID_ADDR" ]; then
    echo -e "${RED}❌ Grid Service contract address not found${NC}"
    exit 1
fi

echo "Grid Service Contract: $GRID_ADDR"
echo ""

# Navigate to grid_service contract directory
cd contracts/grid_service || exit 1

# Build contract if needed
if [ ! -f "target/ink/grid_service.contract" ]; then
    echo "📦 Building contract..."
    cargo contract build --release --quiet
fi

# Create the event
echo "📝 Creating grid event..."
echo ""

# Map event type to contract enum format
case "$EVENT_TYPE" in
    DemandResponse|demand|DR|dr)
        EVENT_ENUM="DemandResponse"
        ;;
    FrequencyRegulation|frequency|FR|fr)
        EVENT_ENUM="FrequencyRegulation"
        ;;
    PeakShaving|peak|PS|ps)
        EVENT_ENUM="PeakShaving"
        ;;
    LoadBalancing|load|LB|lb)
        EVENT_ENUM="LoadBalancing"
        ;;
    Emergency|emergency|EM|em)
        EVENT_ENUM="Emergency"
        ;;
    *)
        EVENT_ENUM="DemandResponse"
        echo -e "${YELLOW}⚠️  Unknown event type, using DemandResponse${NC}"
        ;;
esac

cargo contract call \
    --contract "$GRID_ADDR" \
    --message create_grid_event \
    --args "$EVENT_ENUM" "$DURATION_MINUTES" "$COMPENSATION_RATE" "$TARGET_REDUCTION_KW" \
    --suri "$SURI" \
    --url "$WS_URL" \
    --execute \
    --skip-confirm \
    --skip-dry-run \
    --gas 1000000000000 \
    --proof-size 1000000 \
    --value 0

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Grid event created successfully!${NC}"
    echo ""
    echo -e "${YELLOW}💡 The oracle service will automatically detect and participate in this event${NC}"
else
    echo ""
    echo -e "${RED}❌ Failed to create grid event${NC}"
    exit 1
fi

