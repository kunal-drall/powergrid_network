#!/bin/bash
set -e

echo "🐳 PowerGrid Network Docker Container Starting..."

# Run setup script to ensure dependencies are installed
if [ -f "/workspace/setup-in-container.sh" ]; then
    echo "Running setup script..."
    /workspace/setup-in-container.sh
fi

# Build contracts if they haven't been built yet
if [ ! -d "/workspace/contracts/token/target" ]; then
    echo "Building contracts..."
    cd /workspace
    ./scripts/build-all.sh
fi

# Execute the original command
echo "Executing: $@"
exec "$@"