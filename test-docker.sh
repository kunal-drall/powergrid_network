#!/bin/bash
set -e

echo "🧪 Testing PowerGrid Network Docker Setup"
echo "=========================================="

# Function to check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        echo "❌ Docker daemon is not running"
        exit 1
    fi
    
    echo "✅ Docker is available"
}

# Function to build the Docker image
build_image() {
    echo "🔨 Building PowerGrid Network Docker image..."
    if docker build -t powergrid-network .; then
        echo "✅ Docker image built successfully"
    else
        echo "❌ Failed to build Docker image"
        exit 1
    fi
}

# Function to test substrate-contracts-node
test_node() {
    echo "🔍 Testing substrate-contracts-node..."
    if docker run --rm powergrid-network substrate-contracts-node --version; then
        echo "✅ substrate-contracts-node is working"
    else
        echo "❌ substrate-contracts-node failed"
        exit 1
    fi
}

# Function to test container setup
test_setup() {
    echo "⚙️ Testing container setup..."
    if docker run --rm powergrid-network /workspace/setup-in-container.sh; then
        echo "✅ Container setup completed"
    else
        echo "❌ Container setup failed"
        exit 1
    fi
}

# Function to test contract building
test_build() {
    echo "🔨 Testing contract building..."
    if timeout 300 docker run --rm powergrid-network ./scripts/build-all.sh; then
        echo "✅ Contracts built successfully"
    else
        echo "❌ Contract building failed or timed out"
        exit 1
    fi
}

# Main execution
main() {
    check_docker
    build_image
    test_node
    test_setup
    test_build
    
    echo ""
    echo "🎉 All Docker tests passed!"
    echo "📋 Next steps:"
    echo "   - Run: docker-compose up"
    echo "   - Or run manually: docker run -p 9944:9944 powergrid-network substrate-contracts-node --dev --rpc-external --ws-external"
}

main "$@"