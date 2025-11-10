# PowerGrid Network - Complete Demo Guide

This guide walks you through testing the complete PowerGrid Network MVP.

## Prerequisites

1. **Substrate Node Running**
   ```bash
   ~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all
   ```

2. **Contracts Deployed**
   - All 4 contracts should be deployed
   - Addresses configured in `backend/.env`

3. **Tapo Device**
   - Device powered on and connected to WiFi
   - IP address configured in `backend/.env`

## Demo Flow

### Step 1: Check System Status

Run the full system check:
```bash
./scripts/demo-full-flow.sh
```

This will verify:
- ✅ Node is running
- ✅ Contracts are deployed
- ✅ Device is registered
- ✅ Tapo device is connected

### Step 2: Start Oracle Service

In a terminal, start the oracle service:
```bash
cd backend
source venv/bin/activate
python src/oracle_service.py
```

The oracle will:
- Connect to Tapo device
- Connect to blockchain
- Check device registration
- Start monitoring every 30 seconds

### Step 3: Create a Test Grid Event

In another terminal, create a test event:
```bash
cd backend
source venv/bin/activate
python scripts/create_test_event.py
```

Or use the bash script:
```bash
./scripts/create-grid-event.sh DemandResponse 60 750000000000000000 100
```

This creates:
- Event Type: DemandResponse
- Duration: 60 minutes
- Compensation: 0.75 tokens per kWh
- Target: 100 kW reduction

### Step 4: Watch Oracle Participate

The oracle will automatically:
1. Detect the new grid event (within 30 seconds)
2. Check current energy consumption
3. Participate if energy > 0
4. Report energy contribution
5. Earn token rewards

Watch the logs:
```bash
tail -f backend/logs/oracle.log
```

### Step 5: Check Rewards

Check your token balance and participation:
```bash
cd backend
source venv/bin/activate
python scripts/check-rewards.py
```

## Complete Demo Script

For a quick demo, run everything in sequence:

```bash
# Terminal 1: Node (if not running)
~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external

# Terminal 2: Full system check
./scripts/demo-full-flow.sh

# Terminal 3: Start oracle
cd backend && source venv/bin/activate && python src/oracle_service.py

# Terminal 4: Create event
cd backend && source venv/bin/activate && python scripts/create_test_event.py

# Terminal 5: Watch logs
tail -f backend/logs/oracle.log

# Terminal 6: Check rewards
cd backend && source venv/bin/activate && python scripts/check-rewards.py
```

## Expected Output

### Oracle Service Output
```
🚀 PowerGrid Oracle Service Starting...
✅ Configuration validated
✅ Tapo device connected
✅ Blockchain connected
✅ Device already registered
📊 Monitoring Iteration #1
⚡ Current Power: 0.00 W
📈 Today's Energy: 0.000 kWh
📢 Found 1 active event(s)
🎯 Event 1: DemandResponse
⚠️  No energy contribution to report yet
💰 PWGD Balance: 1000.0000 tokens
```

### When Device Consumes Power
```
📊 Monitoring Iteration #5
⚡ Current Power: 150.50 W
📈 Today's Energy: 0.125 kWh
📢 Found 1 active event(s)
🎯 Event 1: DemandResponse
✅ Participated with 125 Wh
💰 PWGD Balance: 1000.0938 tokens
```

## Troubleshooting

### Oracle not detecting events
- Check that events are active: `python scripts/check-rewards.py`
- Verify oracle is connected: Check logs for connection errors
- Ensure node is running: `curl http://localhost:9944`

### Tapo device not connecting
- Check IP address in `backend/.env`
- Verify device is on and connected to WiFi
- Test connection: `python src/tapo_monitor.py`

### Contract errors
- Verify contracts are deployed: `./scripts/demo-full-flow.sh`
- Check contract addresses in `backend/.env`
- Ensure node is running

## Next Steps

1. **Plug in a device** to the Tapo smart plug
2. **Watch energy consumption** in real-time
3. **See automatic participation** in grid events
4. **Track token rewards** as they accumulate

## Scripts Reference

- `scripts/demo-full-flow.sh` - Complete system check
- `scripts/create-grid-event.sh` - Create events (bash)
- `backend/scripts/create_test_event.py` - Create events (Python)
- `backend/scripts/check-rewards.py` - Check token balance

For more details, see `scripts/README.md`.

