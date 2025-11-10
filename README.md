# PowerGrid Network

A decentralized energy grid management system that connects IoT devices (Tapo P110 smart plugs) to blockchain smart contracts, enabling automatic participation in grid events and token-based rewards.

## 🎯 Overview

PowerGrid Network enables:
- **Real-time Energy Monitoring** - Tapo P110 smart plugs track power consumption
- **Automatic Grid Participation** - Oracle service automatically participates in grid events
- **Token Rewards** - Earn PWGD tokens for energy contributions
- **On-Chain Verification** - All device data and participation recorded on blockchain
- **Decentralized Governance** - Community-driven network management

### System Architecture

```
Tapo P110 → Oracle Service → Blockchain Contracts → Token Rewards
   (IoT)      (Python)         (ink! Smart Contracts)   (PWGD)
```

## ✅ System Status

**Milestone 2 MVP: 100% Complete**

- ✅ Real smart plug sending actual data (Tapo P110)
- ✅ Backend service processing and submitting on-chain
- ✅ Smart contracts receiving and storing data
- ✅ Complete flow from device → oracle → contracts → rewards
- ✅ Documented and reproducible

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Local Setup](#local-setup)
4. [Docker Setup (Multi-Architecture)](#docker-setup-multi-architecture)
5. [Testing](#testing)
6. [Complete Demo Flow](#complete-demo-flow)
7. [API Documentation](#api-documentation)
8. [Troubleshooting](#troubleshooting)
9. [Project Structure](#project-structure)

---

## Prerequisites

### Hardware
- **Tapo P110 Smart Plug** (or compatible Tapo device)
- **WiFi Network** (device must be connected)
- **Computer** (Mac/Linux/Windows with Python 3.10+)

### Software Requirements

#### For Local Development
- **Rust** 1.86.0+ with `wasm32-unknown-unknown` target
- **cargo-contract** v5.0.1+
- **substrate-contracts-node** v0.42.0+
- **Python** 3.10+ with virtual environment
- **Git**

#### For Docker (Multi-Architecture)
- **Docker** 20.10+
- **Docker Compose** 2.0+
- Supports: `linux/amd64` (x86_64) and `linux/arm64` (Apple Silicon)

---

## Quick Start

### Complete Setup (5 Steps)

**1. Start Local Node**
```bash
~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all
```

**2. Deploy Contracts → Get Addresses**
```bash
./scripts/deploy-local.sh
```

**Output:**
```
✅ PowerGrid Token deployed
   Contract: 5FY8e8RtXKDWdeAhnYcBv7TjojDt6NxJNmX7T1TRrZZSZMyk

✅ Resource Registry deployed
   Contract: 5D12ZE2pVZTb3v7RnSMMnf4LPHCAEbcwQWE1TAF9qQiYbFDh

✅ Grid Service deployed
   Contract: 5DW1GhTM696DH4vS5n2zj7L6kFG6t4MVipaEnPKygj48TUtX

✅ Governance deployed
   Contract: 5HdqtBYTtX8KppCxe4ofkFRFR5XTD8uaVCQkXxYhpxGkrTi5
```

**3. Configure Oracle with Addresses**

Edit `backend/.env` and add the contract addresses:
```bash
TOKEN_CONTRACT_ADDRESS=5FY8e8RtXKDWdeAhnYcBv7TjojDt6NxJNmX7T1TRrZZSZMyk
REGISTRY_CONTRACT_ADDRESS=5D12ZE2pVZTb3v7RnSMMnf4LPHCAEbcwQWE1TAF9qQiYbFDh
GRID_SERVICE_CONTRACT_ADDRESS=5DW1GhTM696DH4vS5n2zj7L6kFG6t4MVipaEnPKygj48TUtX
GOVERNANCE_CONTRACT_ADDRESS=5HdqtBYTtX8KppCxe4ofkFRFR5XTD8uaVCQkXxYhpxGkrTi5

# Tapo Device
TAPO_EMAIL=your-email@example.com
TAPO_PASSWORD=your-password
TAPO_DEVICE_IP=192.168.1.33  # Your device IP
```

**4. Connect Real Tapo P110 Device**

Test connection:
```bash
cd backend
source venv/bin/activate
python src/tapo_monitor.py
```

**Expected Output:**
```
✅ Connected to P110 (MAC: 8C-86-DD-C7-6D-7C)
⚡ Current Power: 0.00 W
📈 Today's Energy: 0.000 kWh
```

**5. Run Oracle Service**

```bash
cd backend
source venv/bin/activate
python src/oracle_service.py
```

**Expected Output:**
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

### Alternative: Docker Setup

```bash
# 1. Clone repository
git clone https://github.com/kunal-drall/powergrid_network.git
cd powergrid_network

# 2. Build Docker image (15-30 minutes first time)
docker-compose build

# 3. Start substrate node
docker-compose up -d node

# 4. Run complete E2E test
docker-compose run --rm tester ./scripts/run-e2e-test.sh
```

---

## Local Setup

### 1. Install Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.86.0

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Add rust-src component
rustup component add rust-src
```

### 2. Install cargo-contract

```bash
cargo install --force --locked cargo-contract
```

### 3. Install substrate-contracts-node

**macOS (Apple Silicon):**
```bash
# Download pre-built binary
cd /tmp
curl -L https://github.com/paritytech/substrate-contracts-node/releases/download/v0.42.0/substrate-contracts-node-mac-universal.tar.gz -o substrate-contracts-node.tar.gz
tar -xzf substrate-contracts-node.tar.gz
cp substrate-contracts-node-mac/substrate-contracts-node ~/.local/bin/
chmod +x ~/.local/bin/substrate-contracts-node
```

**Linux:**
```bash
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.42.0
```

**Verify installation:**
```bash
substrate-contracts-node --version
```

### 4. Build Contracts

```bash
./scripts/build-all.sh
```

This builds all 4 contracts:
- `powergrid_token` - PWGD token contract
- `resource_registry` - Device registration
- `grid_service` - Grid event management
- `governance` - DAO governance

### 5. Setup Python Backend

```bash
cd backend
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
```

### 6. Configure Environment

Edit `backend/.env`:

```bash
# Tapo Device Credentials
TAPO_EMAIL=your-email@example.com
TAPO_PASSWORD=your-password
TAPO_DEVICE_IP=192.168.1.44  # Your device IP

# Blockchain Configuration
SUBSTRATE_RPC_URL=ws://127.0.0.1:9944
DEVICE_OWNER_SEED=//Alice  # Or your custom seed

# Contract Addresses (set after deployment)
TOKEN_CONTRACT_ADDRESS=
REGISTRY_CONTRACT_ADDRESS=
GRID_SERVICE_CONTRACT_ADDRESS=
GOVERNANCE_CONTRACT_ADDRESS=

# Service Configuration
MONITORING_INTERVAL_SECONDS=30
STAKE_AMOUNT=2000000000000000000
```

---

## Docker Setup (Multi-Architecture)

### Supported Architectures

- **linux/amd64** (x86_64) - Intel/AMD processors
- **linux/arm64** (ARM64) - Apple Silicon (M1/M2/M3), ARM servers

### Build for Specific Architecture

```bash
# Build for ARM64 (Apple Silicon)
docker build --platform linux/arm64 -t powergrid-network:arm64 .

# Build for AMD64 (Intel/AMD)
docker build --platform linux/amd64 -t powergrid-network:amd64 .

# Build for current platform (auto-detect)
docker build -t powergrid-network .
```

### Docker Compose Usage

The `docker-compose.yml` is configured for multi-architecture support:

```bash
# Build (first time: 15-30 minutes)
docker-compose build

# Start substrate node
docker-compose up -d node

# Run tests in container
docker-compose run --rm tester ./scripts/test-all.sh

# Deploy contracts
docker-compose run --rm tester ./scripts/deploy-local.sh

# Interactive shell
docker-compose run --rm tester bash
```

### Docker Services

- **node**: Substrate contracts node (ws://localhost:9944)
- **tester**: Interactive container for running tests and scripts

### Platform-Specific Notes

**Apple Silicon (M1/M2/M3):**
- Docker automatically uses ARM64
- First build compiles substrate from source (~30 minutes)
- Subsequent builds are faster with cache

**Intel/AMD:**
- Uses x86_64 architecture
- Faster builds with pre-compiled binaries

---

## Testing

### Unit Tests

```bash
# Local
./scripts/test-all.sh

# Docker
docker-compose run --rm tester cargo test --workspace
```

### Integration Tests

```bash
# Local (requires running node)
./scripts/test-integration.sh

# Docker
docker-compose run --rm tester ./scripts/test-integration.sh
```

### End-to-End Test

```bash
# Complete system test
./scripts/run-e2e-test.sh

# This tests:
# - Node connectivity
# - Contract deployment
# - Device registration
# - Tapo connection
# - Event creation
# - Oracle service
```

### Test Tapo Connection

```bash
cd backend
source venv/bin/activate
python src/tapo_monitor.py
```

### Test Blockchain Client

```bash
cd backend
source venv/bin/activate
python src/blockchain_client.py
```

---

## Complete Demo Flow

### Step 1: Start Substrate Node

**Local:**
```bash
~/.local/bin/substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all
```

**Docker:**
```bash
docker-compose up -d node
```

### Step 2: Deploy Contracts

```bash
./scripts/deploy-local.sh
```

**Important:** Copy contract addresses from output and update `backend/.env`.

### Step 3: Setup Authorization

```bash
cd backend
source venv/bin/activate
python scripts/setup_authorization.py
```

This authorizes:
- Oracle service on Grid Service contract
- Grid Service as minter on Token contract

### Step 4: Start Oracle Service

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

### Step 5: Create Test Grid Event

```bash
cd backend
source venv/bin/activate
python scripts/create_test_event.py
```

Or use bash script:
```bash
./scripts/create-grid-event.sh DemandResponse 60 750000000000000000 100
```

### Step 6: Watch Oracle Participate

```bash
# View live logs
tail -f backend/logs/oracle.log
```

When device consumes energy:
```
📊 Monitoring Iteration #5
⚡ Current Power: 150.50 W
📈 Today's Energy: 0.125 kWh
📢 Found 1 active event(s)
🎯 Event 1: DemandResponse
✅ Participated with 125 Wh
💰 PWGD Balance: 1000.0938 tokens
```

### Step 7: Check Rewards

```bash
cd backend
source venv/bin/activate
python scripts/check-rewards.py
```

### Quick Demo Script

Run everything at once:
```bash
./scripts/demo-full-flow.sh
```

---

## API Documentation

### Backend Services

#### TapoMonitor

**Location:** `backend/src/tapo_monitor.py`

**Methods:**
- `async connect()` - Connect to Tapo device
- `async get_current_power()` - Get current power consumption (W)
- `async get_energy_usage()` - Get energy usage (kWh)
- `async get_device_info()` - Get device information
- `async get_complete_snapshot()` - Get all device data

**Example:**
```python
from tapo_monitor import TapoMonitor

monitor = TapoMonitor(email, password, device_ip)
await monitor.connect()
snapshot = await monitor.get_complete_snapshot()
print(f"Current Power: {snapshot['current_power']['power_watts']} W")
```

#### BlockchainClient

**Location:** `backend/src/blockchain_client.py`

**Methods:**
- `connect()` - Connect to Substrate node
- `load_contracts()` - Load all contract instances
- `is_device_registered()` - Check device registration
- `register_device()` - Register device on blockchain
- `get_active_events()` - Get active grid events
- `participate_in_event()` - Participate in grid event
- `get_token_balance()` - Get PWGD token balance
- `get_device_reputation()` - Get device reputation

**Example:**
```python
from blockchain_client import BlockchainClient

client = BlockchainClient(rpc_url, seed_phrase)
client.connect()
client.load_contracts(token_addr, registry_addr, grid_addr, gov_addr)
is_reg = client.is_device_registered()
events = client.get_active_events()
```

#### PowerGridOracle

**Location:** `backend/src/oracle_service.py`

**Main Service:**
- Automatic device registration
- Real-time energy monitoring
- Automatic event participation
- Token reward tracking

**Run:**
```bash
python src/oracle_service.py
```

### Smart Contracts

#### PowerGrid Token

**Contract:** `contracts/token/`

**Key Methods:**
- `balance_of(owner)` - Get token balance
- `transfer(to, value)` - Transfer tokens
- `mint(account, amount)` - Mint tokens (minter only)
- `add_minter(account)` - Grant minter role (admin only)

#### Resource Registry

**Contract:** `contracts/resource_registry/`

**Key Methods:**
- `register_device(metadata, stake)` - Register device
- `is_device_registered(account)` - Check registration
- `get_device_reputation(account)` - Get reputation score

#### Grid Service

**Contract:** `contracts/grid_service/`

**Key Methods:**
- `create_grid_event(type, duration, rate, target)` - Create event
- `participate_in_event(event_id, energy)` - Participate
- `get_active_events()` - Get active events
- `add_authorized_caller(caller)` - Authorize caller (owner only)

#### Governance

**Contract:** `contracts/governance/`

**Key Methods:**
- `create_proposal(type, description)` - Create proposal
- `vote(proposal_id, support)` - Vote on proposal
- `execute_proposal(proposal_id)` - Execute proposal

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
5. Check Tapo app to confirm device is online

### Blockchain Connection Failed

**Symptoms:**
```
❌ Failed to connect to blockchain
```

**Solutions:**
1. Verify node is running: `curl http://localhost:9944`
2. Check `SUBSTRATE_RPC_URL` in `backend/.env`
3. Restart node if needed
4. For Docker: Ensure `docker-compose up node` is running

### Contract Errors

**Symptoms:**
```
❌ Failed to load contracts
```

**Solutions:**
1. Verify contracts are deployed: `./scripts/deploy-local.sh`
2. Check contract addresses in `backend/.env`
3. Ensure node is running and synced
4. Rebuild contracts: `./scripts/build-all.sh`

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
5. Check logs: `tail -f backend/logs/oracle.log`

### Authorization Errors

**Symptoms:**
```
❌ Unauthorized caller
```

**Solutions:**
1. Run authorization setup: `python scripts/setup_authorization.py`
2. Verify owner account has permissions
3. Check contract owner is correct
4. Note: Owner account can always create events

### Docker Build Issues

**Symptoms:**
```
error: could not compile rococo-runtime
```

**Solutions:**
1. Ensure Docker has enough resources (4GB+ RAM)
2. Build for correct platform: `--platform linux/arm64` or `--platform linux/amd64`
3. Clear Docker cache: `docker system prune -a`
4. Check Dockerfile for correct Rust version (1.86.0)

### Multi-Architecture Issues

**Apple Silicon:**
- Use `--platform linux/arm64` explicitly
- First build takes 30+ minutes (compiles from source)
- Ensure Docker Desktop is updated

**Intel/AMD:**
- Use `--platform linux/amd64` explicitly
- Faster builds with pre-compiled binaries

---

## Project Structure

```
powergrid_network/
├── backend/                 # Python oracle service
│   ├── src/
│   │   ├── oracle_service.py      # Main oracle service
│   │   ├── blockchain_client.py   # Blockchain integration
│   │   └── tapo_monitor.py        # Tapo device integration
│   ├── config/
│   │   ├── config.py              # Configuration management
│   │   └── abis/                  # Contract ABIs
│   ├── scripts/
│   │   ├── create_test_event.py   # Create grid events
│   │   ├── check-rewards.py       # Check token balance
│   │   └── setup_authorization.py # Setup permissions
│   └── requirements.txt           # Python dependencies
├── contracts/               # ink! smart contracts
│   ├── token/              # PWGD token contract
│   ├── resource_registry/  # Device registration
│   ├── grid_service/      # Grid event management
│   └── governance/        # DAO governance
├── scripts/                # Build and deployment scripts
│   ├── setup.sh           # Initial setup
│   ├── build-all.sh       # Build all contracts
│   ├── deploy-local.sh    # Deploy contracts
│   ├── test-all.sh        # Run unit tests
│   ├── run-e2e-test.sh    # End-to-end test
│   └── demo-full-flow.sh  # Complete demo
├── Dockerfile              # Multi-architecture Docker image
├── docker-compose.yml      # Docker Compose configuration
└── README.md              # This file
```

---

## Scripts Reference

### Build Scripts
- `./scripts/setup.sh` - Install all dependencies
- `./scripts/build-all.sh` - Build all contracts
- `./scripts/test-all.sh` - Run unit tests
- `./scripts/test-integration.sh` - Run integration tests

### Deployment Scripts
- `./scripts/deploy-local.sh` - Deploy all contracts locally
- `./scripts/start-node-and-deploy.sh` - Start node and deploy

### Demo Scripts
- `./scripts/demo-full-flow.sh` - Complete system check
- `./scripts/create-grid-event.sh` - Create grid event (bash)
- `./scripts/run-e2e-test.sh` - End-to-end integration test

### Backend Scripts
- `backend/scripts/create_test_event.py` - Create grid event (Python)
- `backend/scripts/check-rewards.py` - Check token balance
- `backend/scripts/setup_authorization.py` - Setup contract permissions

---

## Configuration

### Environment Variables

**Backend (`backend/.env`):**
- `TAPO_EMAIL` - Tapo account email
- `TAPO_PASSWORD` - Tapo account password
- `TAPO_DEVICE_IP` - Device IP address
- `SUBSTRATE_RPC_URL` - Blockchain RPC endpoint
- `DEVICE_OWNER_SEED` - Account seed phrase
- `TOKEN_CONTRACT_ADDRESS` - Token contract address
- `REGISTRY_CONTRACT_ADDRESS` - Registry contract address
- `GRID_SERVICE_CONTRACT_ADDRESS` - Grid Service contract address
- `GOVERNANCE_CONTRACT_ADDRESS` - Governance contract address
- `MONITORING_INTERVAL_SECONDS` - Oracle monitoring interval (default: 30)
- `STAKE_AMOUNT` - Device registration stake (default: 2 tokens)

### Contract Configuration

Contracts are configured via constructor arguments during deployment. See `scripts/deploy-local.sh` for details.

---

## Development

### Adding New Contracts

1. Create contract in `contracts/your_contract/`
2. Add to workspace `Cargo.toml`
3. Build: `cd contracts/your_contract && cargo contract build --release`
4. Deploy: Add to `scripts/deploy-local.sh`
5. Add ABI to `backend/config/abis/`
6. Load in `blockchain_client.py`

### Adding New Features

1. Update contract code in `contracts/`
2. Rebuild contracts: `./scripts/build-all.sh`
3. Update backend if needed: `backend/src/`
4. Test: `./scripts/test-all.sh`
5. Deploy: `./scripts/deploy-local.sh`

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `./scripts/test-all.sh`
5. Submit a pull request

---

## License

See [LICENSE](LICENSE) file for details.

---

## Support

For issues or questions:
- Check [Troubleshooting](#troubleshooting) section
- Review logs: `backend/logs/oracle.log`
- Check contract events on blockchain explorer

---

## Evidence of Working System

### Terminal Logs - Complete Flow ✅

**Device Connection:**
```
INFO:__main__:✅ Connected to P110 (MAC: 8C-86-DD-C7-6D-7C)
INFO:__main__:⚡ Current Power: 45.20 W
INFO:__main__:📈 Today's Energy: 0.125 kWh
```
✅ **Device connecting** - Real Tapo P110 sending actual data

**Blockchain Transactions:**
```
✅ Test event created successfully!
   Transaction: 0x735dd73e97f71384d75c457cea18f3de67797d5ed7a9ae3f40e0fbec52cfd8db
   Block: 0xfe119dd155d9f12bc29508c96ae4b5ec74ac3bb054dfc25aebb43b4c24584a6e
```
✅ **Blockchain transactions** - All operations recorded on-chain

**Participation Recorded:**
```
INFO:__main__:📢 Found 1 active event(s)
INFO:__main__:🎯 Event 1: DemandResponse
INFO:__main__:✅ Participated with 125 Wh
```
✅ **Participation recorded** - Oracle automatically participates

**Rewards Distributed:**
```
INFO:__main__:💰 PWGD Balance: 1000.0938 tokens
```
✅ **Rewards distributed** - Token balance increases after participation

### Multiple Monitoring Cycles ✅

**Cycle 1:**
```
📊 Monitoring Iteration #1
⚡ Current Power: 0.00 W
📈 Today's Energy: 0.000 kWh
💰 PWGD Balance: 1000.0000 tokens
```

**Cycle 2 (With Energy):**
```
📊 Monitoring Iteration #3
⚡ Current Power: 45.20 W
📈 Today's Energy: 0.125 kWh
✅ Participated with 125 Wh
💰 PWGD Balance: 1000.0938 tokens
```

### On-Chain State Changes - Proof ✅

**Device Registration:**
- Before: `Device registered: False`
- After: `Device registered: True` ✅

**Token Balance:**
- Before: `1000.0000 tokens`
- After: `1000.0938 tokens` (+0.0938 tokens) ✅

**Active Events:**
- Created: `Event ID: 1, Type: DemandResponse` ✅
- Detected: `Found 1 active event(s)` ✅

**See [docs/TEST_RESULTS.md](docs/TEST_RESULTS.md) for complete evidence and test results.**

## Status

**✅ Milestone 2 MVP: 100% Complete and Verified**

- ✅ Real hardware integration (Tapo P110) - **Verified with actual device**
- ✅ Complete data pipeline - **Tested with real power readings**
- ✅ Blockchain integration - **All transactions verified on-chain**
- ✅ Automatic event participation - **Confirmed with multiple cycles**
- ✅ Token reward system - **Rewards distributed and tracked**
- ✅ Multi-architecture support - **ARM64 and x86_64 tested**
- ✅ Comprehensive documentation - **Complete setup and testing guides**

**Ready for production deployment and scaling!**
