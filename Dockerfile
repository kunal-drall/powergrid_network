# Use the official Swanky image as our base.
# It is published on the GitHub Container Registry (ghcr.io).
FROM ghcr.io/astarnetwork/swanky-node:latest

# The swanky image already contains all the necessary tools and dependencies.
# We just need to set up our working directory and copy our project files.

# Set the working directory inside the container
WORKDIR /app

# Copy your local project files into the container's working directory
# Ensure you have a .dockerignore file to avoid copying unnecessary files
COPY . .

# The swanky-node is already installed at /usr/local/bin/swanky-node.
# The default swanky-node runs on port 9944.
CMD ["swanky-node", "--dev", "--unsafe-rpc-external", "--rpc-cors=all"]