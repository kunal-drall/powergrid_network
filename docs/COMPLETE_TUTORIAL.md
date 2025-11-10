# PowerGrid Network - Complete Tutorial

## From Hardware to Blockchain: Complete Flow

This tutorial walks you through the complete PowerGrid Network system, from connecting a Tapo P110 smart plug to earning blockchain rewards.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Setup](#setup)
4. [Hardware Connection](#hardware-connection)
5. [Blockchain Setup](#blockchain-setup)
6. [Oracle Service](#oracle-service)
7. [Grid Events](#grid-events)
8. [Rewards & Verification](#rewards--verification)
9. [Troubleshooting](#troubleshooting)

---

## Overview

The PowerGrid Network connects IoT devices (like Tapo P110 smart plugs) to a blockchain network, allowing them to:

1. **Monitor Energy Consumption** - Real-time power and energy tracking
2. **Participate in Grid Events** - Automatically respond to grid demand signals
3. **Earn Token Rewards** - Receive PWGD tokens for energy contributions
4. **Build Reputation** - Track successful participations on-chain

### System Architecture

```
Tapo P110 → Oracle Service → Blockchain Contracts → Token Rewards
   (IoT)      (Python)         (ink! Smart Contracts)   (PWGD)
```

---

## Prerequisites

### Hardware
- **Tapo P110 Smart Plug** (or compatible Tapo device)
- **WiFi Network** (device must be connected)
- **Computer** (Mac/Linux/Windows with Python 3.10+)

### Software
- **Rust** (1.86.0+) with `wasm32-unknown-unknown` target
- **cargo-contract** (for building contracts)
- **substrate-contracts-node** (local blockchain node)
- **Python 3.10+** with virtual environment
- **Git** (for cloning repository)

### Accounts
- **Tapo Account** (email/password for device access)
- **Blockchain Account** (seed phrase for //Alice or custom account)

---

## Setup

### 1. Clone Repository

```bash
git clone https://github.com/kunal-drall/powergrid_network.git
cd powergrid_network
```

### 2. Install Dependencies

#### Rust & Contracts

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.86.0
rustup target add wasm32-unknown-unknown
rustup component add rust-src

# Install cargo-contract
cargo install --force --locked cargo-contract

# Install substrate-contracts-node
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.42.0
```

#### Python Backend

```bash
cd backend
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
```

### 3. Build Contracts

```bash
cd contracts
./scripts/build-all.sh
```

---

## Hardware Connection

### 1. Set Up Tapo Device

1. **Install Tapo App** on your phone
2. **Add Device** - Follow Tapo app instructions to connect P110 to WiFi
3. **Note Device IP** - Find IP address in Tapo app settings
4. **Test Connection** - Ensure device is online and controllable

### 2. Configure Backend

Edit `backend/.env`:

```bash
# Tapo Device Credentials
TAPO_EMAIL=your-email@example.com
TAPO_PASSWORD=your-password
TAPO_DEVICE_IP=192.168.1.44  # Your device IP

# Blockchain Configuration
SUBSTRATE_RPC_URL=ws://127.0.0.1:9944
DEVICE_OWNER_SEED=//Alice  # Or your custom seed

# Contract Addresses (will be set after deployment)
TOKEN_CONTRACT_ADDRESS=
REGISTRY_CONTRACT_ADDRESS=
GRID_SERVICE_CONTRACT_ADDRESS=
GOVERNANCE_CONTRACT_ADDRESS=

# Service Configuration
MONITORING_INTERVAL_SECONDS=30
STAKE_AMOUNT=2000000000000000000
```

### 3. Test Tapo Connection

```bash
cd backend
source venv/bin/activate
python src/tapo_monitor.py
```

Expected output:
```
✅ Connected to P110 (MAC: XX-XX-XX-XX-XX-XX)
📊 Complete Device Snapshot:
   Current Power: 0.00 W
   Today's Energy: 0.000 kWh
```

---

## Blockchain Setup

### 1. Start Local Node

```bash
substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all
```

Keep this terminal open. The node will run on `ws://localhost:9944`.

### 2. Deploy Contracts

```bash
./scripts/deploy-local.sh
```

This will deploy:
- **PowerGrid Token** - PWGD token contract
- **Resource Registry** - Device registration
- **Grid Service** - Grid event management
- **Governance** - DAO governance

**Important:** Copy the contract addresses from the output and update `backend/.env`.

### 3. Set Up Authorization

```bash
cd backend
source venv/bin/activate
python scripts/setup_authorization.py
```

This authorizes the oracle service to:
- Create grid events
- Mint tokens for rewards
- Update device registry

---

## Oracle Service

### 1. Start Oracle Service

```bash
cd backend
source venv/bin/activate
python src/oracle_service.py
```

The oracle will:
- ✅ Connect to Tapo device
- ✅ Connect to blockchain
- ✅ Register device (first run)
- ✅ Start monitoring every 30 seconds

### 2. Monitor Logs

```bash
tail -f backend/logs/oracle.log
```

Expected output:
```
🚀 PowerGrid Oracle Service Starting...
✅ Configuration validated
✅ Tapo device connected
✅ Blockchain connected
✅ Device already registered
📊 Monitoring Iteration #1
⚡ Current Power: 0.00 W
📈 Today's Energy: 0.000 kWh
💰 PWGD Balance: 1000.0000 tokens
```

### 3. Oracle Workflow

Every 30 seconds, the oracle:
1. **Reads Device Data** - Gets current power and energy from Tapo
2. **Checks Registration** - Verifies device is registered on-chain
3. **Monitors Events** - Scans for active grid events
4. **Participates** - Reports energy contribution if event is active
5. **Tracks Rewards** - Monitors token balance

---

## Grid Events

### 1. Create Test Event

```bash
cd backend
source venv/bin/activate
python scripts/create_test_event.py
```

Or use bash script:
```bash
./scripts/create-grid-event.sh DemandResponse 60 750000000000000000 100
```

This creates:
- **Type:** DemandResponse
- **Duration:** 60 minutes
- **Compensation:** 0.75 tokens per kWh
- **Target:** 100 kW reduction

### 2. Event Types

- **DemandResponse** - Reduce load during peak demand
- **FrequencyRegulation** - Maintain grid frequency
- **PeakShaving** - Reduce peak consumption
- **LoadBalancing** - Balance grid load
- **Emergency** - Emergency grid support

### 3. Oracle Participation

When an event is active and your device consumes energy:
```
📊 Monitoring Iteration #5
⚡ Current Power: 150.50 W
📈 Today's Energy: 0.125 kWh
📢 Found 1 active event(s)
🎯 Event 1: DemandResponse
✅ Participated with 125 Wh
💰 PWGD Balance: 1000.0938 tokens
```

---

## Rewards & Verification

### 1. Check Token Balance

```bash
cd backend
source venv/bin/activate
python scripts/check-rewards.py
```

Output:
```
💰 Token Balance:
   PWGD Balance: 1000.0938 tokens
   Raw Balance: 1000093800000000000000 wei

📋 Device Status:
   ✅ Device is registered
   Reputation: 0

📢 Active Grid Events:
   Found 1 active event(s)
```

### 2. Verify Participation

Check blockchain events:
```bash
# View contract events in logs or use substrate explorer
```

### 3. Track Rewards Over Time

The oracle automatically:
- ✅ Participates in events when energy is consumed
- ✅ Earns tokens based on compensation rate
- ✅ Updates reputation based on participation
- ✅ Tracks all activity on-chain

---

## Troubleshooting

### Tapo Device Not Connecting

**Symptoms:**
```
❌ Failed to connect to Tapo device: Connection refused
```

**Solutions:**
1. Check device IP in `backend/.env`
2. Verify device is powered on and connected to WiFi
3. Test connection: `python src/tapo_monitor.py`
4. Update IP if device changed networks

### Blockchain Connection Failed

**Symptoms:**
```
❌ Failed to connect to blockchain
```

**Solutions:**
1. Verify node is running: `curl http://localhost:9944`
2. Check `SUBSTRATE_RPC_URL` in `backend/.env`
3. Restart node if needed

### Contract Errors

**Symptoms:**
```
❌ Failed to load contracts
```

**Solutions:**
1. Verify contracts are deployed: `./scripts/deploy-local.sh`
2. Check contract addresses in `backend/.env`
3. Ensure node is running and synced

### Oracle Not Participating

**Symptoms:**
```
⚠️  No energy contribution to report yet
```

**Solutions:**
1. Plug something into Tapo device (must consume power)
2. Wait for next monitoring cycle (30 seconds)
3. Check that grid event is active
4. Verify device is registered

### Authorization Errors

**Symptoms:**
```
❌ Unauthorized caller
```

**Solutions:**
1. Run authorization setup: `python scripts/setup_authorization.py`
2. Verify owner account has permissions
3. Check contract owner is correct

---

## Complete Demo Flow

### Quick Start

```bash
# Terminal 1: Start node
substrate-contracts-node --dev --tmp --rpc-external

# Terminal 2: Deploy contracts
./scripts/deploy-local.sh

# Terminal 3: Run end-to-end test
./scripts/run-e2e-test.sh

# Terminal 4: Start oracle
cd backend && source venv/bin/activate && python src/oracle_service.py

# Terminal 5: Create event
cd backend && source venv/bin/activate && python scripts/create_test_event.py

# Terminal 6: Watch logs
tail -f backend/logs/oracle.log
```

### Expected Flow

1. **Device Registration** (first run)
   - Oracle registers device on blockchain
   - Stakes tokens for participation
   - Gets initial reputation

2. **Energy Monitoring** (every 30s)
   - Oracle reads power consumption
   - Tracks daily energy usage
   - Logs all data

3. **Event Participation** (when active)
   - Oracle detects active grid event
   - Reports energy contribution
   - Receives token rewards

4. **Reward Tracking** (continuous)
   - Token balance updates
   - Reputation increases
   - Participation history recorded

---

## Next Steps

1. **Scale Up** - Add more devices
2. **Custom Events** - Create specific grid events
3. **Governance** - Participate in DAO proposals
4. **Analytics** - Build dashboards for energy data
5. **Integration** - Connect to other IoT devices

---

## Support

For issues or questions:
- Check `scripts/README.md` for script documentation
- Review `DEMO_GUIDE.md` for demo instructions
- See `SETUP_COMPLETE.md` for setup details

---

**🎉 Congratulations!** You've successfully set up the PowerGrid Network MVP!

