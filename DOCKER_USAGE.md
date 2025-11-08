# Docker Usage Guide

## Quick Start

### Build the Docker Image
```bash
docker build -t powergrid-network .
```
**Note**: First build takes 15-30 minutes (compiles substrate-contracts-node from source)

### Option 1: Start Node with Docker Compose
```bash
# Start the substrate node
docker-compose up node

# In another terminal, run tests
docker-compose run tester bash
# Then inside container:
./scripts/test-integration.sh
```

### Option 2: Interactive Container
```bash
# Start interactive container with node running in background
docker run -it --rm \
  -v $(pwd):/app \
  -w /app \
  -p 9944:9944 \
  -p 9933:9933 \
  -p 30333:30333 \
  powergrid-network bash
```

### Option 3: Run Node Only
```bash
# Start substrate node
docker run -d --name powergrid-node \
  -p 9944:9944 \
  -p 9933:9933 \
  -p 30333:30333 \
  powergrid-network \
  substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all

# Check logs
docker logs -f powergrid-node

# Stop node
docker stop powergrid-node && docker rm powergrid-node
```

## Inside the Container

Once inside the container, you have access to:

- ✅ Rust 1.86.0
- ✅ cargo-contract 5.0.1
- ✅ substrate-contracts-node v0.42.0
- ✅ All build tools (clang, binaryen, etc.)

### Available Commands

```bash
# Build all contracts
./scripts/build-all.sh

# Run unit tests
./scripts/test-all.sh

# Run integration tests (requires node running)
./scripts/test-integration.sh

# Deploy contracts
./scripts/deploy-local.sh

# Check node version
substrate-contracts-node --version

# Check cargo-contract version
cargo-contract --version
```

## Running Integration Tests

### Method 1: Using Docker Compose
```bash
# Terminal 1: Start node
docker-compose up node

# Terminal 2: Run tests
docker-compose run tester bash
cd /app
./scripts/test-integration.sh
```

### Method 2: Separate Containers
```bash
# Terminal 1: Start node
docker run -d --name powergrid-node \
  -p 9944:9944 \
  powergrid-network \
  substrate-contracts-node --dev --tmp --rpc-external --rpc-cors all

# Terminal 2: Run tests
docker run -it --rm \
  --link powergrid-node:node \
  -v $(pwd):/app \
  -w /app \
  -e CONTRACTS_NODE=ws://node:9944 \
  powergrid-network \
  ./scripts/test-integration.sh
```

## Checking Build Progress

```bash
# Check Docker build logs
tail -f /tmp/docker-build.log

# Or if using docker build directly
docker build -t powergrid-network . 2>&1 | tee docker-build.log
```

## Troubleshooting

### Build fails with wasm32-unknown-unknown error
The Dockerfile should handle this automatically, but if issues persist:
- Ensure Docker has enough resources (8GB+ RAM recommended)
- Check Docker logs: `docker logs <container-id>`

### Container can't find contracts-node
Ensure the Docker image was built successfully:
```bash
docker images | grep powergrid-network
```

### Port conflicts
If ports 9944, 9933, or 30333 are already in use:
```bash
# Change ports in docker-compose.yml or use:
docker run -p 9945:9944 ...  # Use different host port
```

## Next Steps

Once the Docker image is built:

1. **Start the node**:
   ```bash
   docker-compose up node
   ```

2. **Run integration tests**:
   ```bash
   docker-compose run tester bash
   cd /app && ./scripts/test-integration.sh
   ```

3. **Build and deploy contracts**:
   ```bash
   docker-compose run tester bash
   cd /app && ./scripts/build-all.sh && ./scripts/deploy-local.sh
   ```

