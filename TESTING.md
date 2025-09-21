# PowerGrid Network Testing Guide

This document provides step-by-step instructions for testing the PowerGrid Network smart contracts.

## Prerequisites

Before running any tests, ensure you have completed the initial setup:

```bash
./scripts/setup.sh
```

## Testing Levels

### 1. Unit Testing

Unit tests verify individual contract functions in isolation.

**Run all unit tests:**
```bash
./scripts/test-all.sh
```

**Run tests for a specific contract:**
```bash
cd contracts/token
cargo test

cd contracts/resource_registry  
cargo test

cd contracts/grid_service
cargo test

cd contracts/governance
cargo test
```

**Run specific test functions:**
```bash
cd contracts/token
cargo test test_token_creation

cd contracts/grid_service
cargo test test_grid_automation_system
```

### 2. Integration Testing

Integration tests verify contract compilation and basic functionality.

**Build all contracts:**
```bash
./scripts/build-all.sh
```

**Check workspace compilation:**
```bash
cargo check --workspace
```

**Build individual contracts:**
```bash
cd contracts/token
cargo contract build --release

cd contracts/resource_registry
cargo contract build --release

cd contracts/grid_service  
cargo contract build --release

cd contracts/governance
cargo contract build --release
```

### 3. End-to-End Testing

E2E tests verify complete workflows with deployed contracts.

**Step 1: Start the local blockchain node**
```bash
# In Terminal 1 (keep running)
./scripts/run-node.sh
```

**Step 2: Deploy contracts and run E2E tests**
```bash
# In Terminal 2
./scripts/deploy-and-run-e2e.sh
```

## Manual Contract Testing

For detailed contract testing, follow these steps after deployment:

### Step 1: Deploy Contracts

```bash
# Start node (Terminal 1)
./scripts/run-node.sh

# Deploy (Terminal 2) 
./scripts/deploy-and-run-e2e.sh
```

### Step 2: Extract Contract Addresses

From the deployment output, note down the contract addresses that look like:
```
Contract 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL
```

### Step 3: Test Token Contract

```bash
cd contracts/token

# Check total supply
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message total_supply \
  --suri //Alice \
  --url ws://localhost:9944

# Check Alice's balance  
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message balance_of \
  --args 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --suri //Alice \
  --url ws://localhost:9944

# Transfer tokens to Bob
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message transfer \
  --args 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty 1000000000000000000 \
  --suri //Alice \
  --url ws://localhost:9944 \
  --execute \
  --skip-confirm
```

### Step 4: Test Resource Registry

```bash
cd contracts/resource_registry

# Check minimum stake
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message get_min_stake \
  --suri //Alice \
  --url ws://localhost:9944

# Register a device (requires token transfer first)
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message register_device \
  --args '{"device_type":{"SmartPlug":null},"capacity_watts":2000,"location":"Living Room","manufacturer":"PowerGrid Inc","model":"SmartNode-1","firmware_version":"1.0.0","installation_date":1640995200000}' \
  --suri //Bob \
  --url ws://localhost:9944 \
  --execute \
  --skip-confirm \
  --value 1000000000000000000
```

### Step 5: Test Grid Service

```bash
cd contracts/grid_service

# Check grid conditions
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message get_grid_conditions \
  --suri //Alice \
  --url ws://localhost:9944

# Create a grid event
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message create_grid_event \
  --args 'LoadReduction {"target_reduction_kw":1000,"duration_minutes":60,"compensation_per_kwh":100000000000000000}' \
  --suri //Alice \
  --url ws://localhost:9944 \
  --execute \
  --skip-confirm
```

### Step 6: Test Governance

```bash
cd contracts/governance

# Create a proposal
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message create_proposal \
  --args '{"UpdateMinStake":2000000000000000000} "Increase minimum stake for security"' \
  --suri //Alice \
  --url ws://localhost:9944 \
  --execute \
  --skip-confirm

# Vote on proposal
cargo contract call \
  --contract CONTRACT_ADDRESS \
  --message vote \
  --args 1 true "I support this change" \
  --suri //Alice \
  --url ws://localhost:9944 \
  --execute \
  --skip-confirm
```

## Automated Testing Scripts

The repository includes the following core testing scripts:

| Script | Purpose | Prerequisites |
|--------|---------|--------------|
| `./scripts/test-all.sh` | Run all unit tests | None |
| `./scripts/build-all.sh` | Build all contracts | Rust toolchain |
| `./scripts/deploy-and-run-e2e.sh` | Deploy & E2E test | Running node |

## Troubleshooting

### Common Issues

**Issue**: `substrate-contracts-node: command not found`
**Solution**: Run `./scripts/setup.sh` to install dependencies

**Issue**: `Connection refused on port 9944`  
**Solution**: Start the node with `./scripts/run-node.sh`

**Issue**: `Contract build failed`
**Solution**: 
```bash
cargo check --workspace
cargo update
./scripts/build-all.sh
```

**Issue**: `Deployment failed with out of gas`
**Solution**: Increase gas limit in deployment commands

**Issue**: `Unit tests failing`
**Solution**: Check dependencies and run:
```bash
cd contracts/failing_contract
cargo test -- --nocapture
```

### Debug Mode

For verbose output during testing:

```bash
# Verbose unit tests
cargo test -- --nocapture

# Verbose contract calls  
cargo contract call --verbose

# Check contract metadata
cargo contract info --contract CONTRACT_ADDRESS
```

## Test Coverage

The testing suite covers:

- ✅ Token contract: Creation, transfers, balances
- ✅ Resource Registry: Device registration, staking, reputation
- ✅ Grid Service: Event creation, participation, automation
- ✅ Governance: Proposal creation, voting, parameter updates
- ✅ Cross-contract integration: Token transfers for staking
- ✅ Security features: Reentrancy protection, access controls

## Performance Testing

For performance evaluation:

```bash
# Build with optimizations
cargo contract build --release

# Measure gas usage in calls
cargo contract call --dry-run

# Check contract size
ls -la contracts/*/target/ink/*.wasm
```

## Continuous Integration

For CI/CD pipelines, use this test sequence:

```bash
#!/bin/bash
set -e

echo "Running CI tests..."

# Unit tests
./scripts/test-all.sh

# Build verification  
./scripts/build-all.sh

# Workspace check
cargo check --workspace

echo "All tests passed!"
```