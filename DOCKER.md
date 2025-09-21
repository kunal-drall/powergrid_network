# Docker Setup for PowerGrid Network

This directory contains Docker configurations to run the PowerGrid Network smart contracts in a containerized environment.

## Quick Start

### Prerequisites
- Docker
- Docker Compose

### Build and Run

1. **Build the Docker image:**
   ```bash
   docker build -t powergrid-network .
   ```

2. **Run with Docker Compose:**
   ```bash
   docker-compose up
   ```

   This will:
   - Start a substrate-contracts-node in development mode
   - Build all smart contracts
   - Deploy all contracts and run E2E tests
   - Save deployment addresses to `deployment/local-addresses.json`

3. **Run specific services:**
   ```bash
   # Only start the node
   docker-compose up substrate-node
   
   # Run E2E tests against running node
   docker-compose up e2e-runner
   ```

## Components

### Dockerfile
Multi-stage Docker build that:
- **Stage 1 (Builder):** Installs Rust toolchain, cargo-contract v5.0.1, builds all contracts
- **Stage 2 (Runtime):** Lightweight runtime with substrate-contracts-node and built artifacts

### docker-compose.yml
Orchestrates two services:
- **substrate-node:** Runs substrate-contracts-node in dev mode with RPC enabled
- **e2e-runner:** Deploys contracts and runs end-to-end tests

### Scripts
- `scripts/deploy-and-run-e2e-docker.sh`: Docker-compatible version of E2E script
  - Supports configurable node URL via `NODE_URL` environment variable
  - Handles Docker networking properly

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NODE_URL` | `ws://localhost:9944` | WebSocket URL for substrate-contracts-node |
| `DOCKER_CONTAINER` | `1` | Indicates running in Docker environment |

## File Structure

```
.
├── Dockerfile                              # Multi-stage Docker build
├── docker-compose.yml                      # Service orchestration
├── .dockerignore                          # Excludes unnecessary files
├── scripts/
│   ├── deploy-and-run-e2e-docker.sh      # Docker-compatible E2E script
│   └── ...                               # Other project scripts
└── deployment/
    └── local-addresses.json              # Contract deployment addresses
```

## Troubleshooting

### Build Issues
- Ensure Docker has enough memory allocated (4GB+ recommended)
- Check network connectivity for downloading dependencies

### Node Connection Issues
- Verify substrate-node service is healthy: `docker-compose ps`
- Check logs: `docker-compose logs substrate-node`
- Ensure port 9944 is not in use by another process

### Contract Deployment Issues
- Check e2e-runner logs: `docker-compose logs e2e-runner`
- Verify contracts built successfully in builder stage
- Ensure substrate-node is responding to health checks

## Development Workflow

1. **Make changes to contracts**
2. **Rebuild and test:**
   ```bash
   docker-compose down
   docker-compose build --no-cache
   docker-compose up
   ```

3. **Debug specific issues:**
   ```bash
   # Run interactive shell in container
   docker run -it powergrid-network bash
   
   # Check contract builds
   ./scripts/build-all.sh
   
   # Test node connectivity
   curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' http://substrate-node:9944
   ```

## Production Considerations

For production deployments:
- Use specific image tags instead of `latest`
- Configure proper networking and security
- Use persistent volumes for blockchain data
- Monitor container health and resources
- Consider using Docker Swarm or Kubernetes for orchestration