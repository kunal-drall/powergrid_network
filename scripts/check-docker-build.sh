#!/usr/bin/env bash
# Check Docker build status and start services when ready

set -e

IMAGE_NAME="powergrid-network"
LOG_FILE="/tmp/docker-build-arm64-v2.log"

echo "🔍 Checking Docker build status..."

# Check if build process is running
if ps aux | grep -q "[d]ocker build.*powergrid-network"; then
    echo "⏳ Build is still running..."
    echo ""
    echo "📊 Recent build activity:"
    tail -5 "$LOG_FILE" 2>/dev/null | grep -E "(Compiling|Installing|Finished)" | tail -3 || echo "   (checking...)"
    echo ""
    echo "💡 Monitor live: tail -f $LOG_FILE"
    exit 0
fi

# Check if image exists
if docker images | grep -q "$IMAGE_NAME"; then
    echo "✅ Docker image '$IMAGE_NAME' built successfully!"
    echo ""
    docker images | grep "$IMAGE_NAME"
    echo ""
    echo "🚀 Ready to start services!"
    echo ""
    echo "To start the node:"
    echo "   docker-compose up -d node"
    echo ""
    echo "To check node logs:"
    echo "   docker-compose logs -f node"
    exit 0
else
    echo "❌ Docker image not found."
    echo ""
    echo "Checking build log for errors..."
    if [ -f "$LOG_FILE" ]; then
        echo ""
        tail -20 "$LOG_FILE" | grep -E "(ERROR|error|Failed)" | tail -5 || echo "No errors found in recent log"
    fi
    echo ""
    echo "💡 To rebuild:"
    echo "   docker build --platform linux/arm64 -t powergrid-network ."
    exit 1
fi
