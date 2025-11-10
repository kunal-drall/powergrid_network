# PowerGrid Network - Demo and Testing Scripts

This directory contains scripts for testing and demonstrating the PowerGrid Network MVP.

## Quick Start

### 1. Full Demo Flow
Run the complete demo to check all components:
```bash
./scripts/demo-full-flow.sh
```

### 2. Create a Grid Event
Create a test grid event for the oracle to participate in:
```bash
./scripts/create-grid-event.sh [EVENT_TYPE] [DURATION_MIN] [COMPENSATION_RATE] [TARGET_REDUCTION_KW]
```

**Example:**
```bash
# Create a 30-minute DemandResponse event with 100 kW target
./scripts/create-grid-event.sh DemandResponse 30 1000000000000000000 100
```

**Parameters:**
- `EVENT_TYPE`: `DemandResponse`, `FrequencyRegulation`, or `VoltageSupport` (default: `DemandResponse`)
- `DURATION_MIN`: Duration in minutes (default: `30`)
- `COMPENSATION_RATE`: Compensation rate in wei (default: `1000000000000000000` = 1 token)
- `TARGET_REDUCTION_KW`: Target energy reduction in kW (default: `100`)

### 3. Check Rewards
Check your token balance and participation status:
```bash
cd backend
source venv/bin/activate
python scripts/check-rewards.py
```

## Available Scripts

### `create-grid-event.sh`
Creates a new grid event on the blockchain. The oracle service will automatically detect and participate in active events.

**Usage:**
```bash
./scripts/create-grid-event.sh [options]
```

### `demo-full-flow.sh`
Runs a complete system check:
- Verifies node is running
- Checks contract deployment
- Verifies device registration
- Tests Tapo device connection
- Creates a test grid event
- Checks oracle service status

**Usage:**
```bash
./scripts/demo-full-flow.sh
```

### `check-rewards.py`
Python script to check:
- Token balance (PWGD tokens)
- Device registration status
- Device reputation
- Active grid events
- Account balance (for gas)

**Usage:**
```bash
cd backend
source venv/bin/activate
python scripts/check-rewards.py
```

## Testing Workflow

1. **Start the node** (if not running):
   ```bash
   ~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external
   ```

2. **Start the oracle service**:
   ```bash
   cd backend
   source venv/bin/activate
   python src/oracle_service.py
   ```

3. **Create a test event**:
   ```bash
   ./scripts/create-grid-event.sh DemandResponse 30 1000000000000000000 100
   ```

4. **Plug something into the Tapo device** (to generate energy consumption)

5. **Watch the oracle participate**:
   ```bash
   tail -f backend/logs/oracle.log
   ```

6. **Check rewards**:
   ```bash
   cd backend && source venv/bin/activate && python scripts/check-rewards.py
   ```

## Environment Variables

The scripts use environment variables from `backend/.env`:
- `SUBSTRATE_RPC_URL`: Blockchain RPC endpoint (default: `ws://localhost:9944`)
- `GRID_SERVICE_CONTRACT_ADDRESS`: Grid Service contract address
- `TOKEN_CONTRACT_ADDRESS`: Token contract address
- `REGISTRY_CONTRACT_ADDRESS`: Registry contract address

## Troubleshooting

### Node not running
```bash
~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external
```

### Contracts not deployed
```bash
./scripts/deploy-local.sh
```

### Oracle service not running
```bash
cd backend
source venv/bin/activate
python src/oracle_service.py
```

### Tapo device not connected
- Check IP address in `backend/.env` (`TAPO_DEVICE_IP`)
- Verify device is powered on and connected to WiFi
- Test connection: `cd backend && source venv/bin/activate && python src/tapo_monitor.py`

