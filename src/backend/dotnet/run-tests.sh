#!/bin/bash

# Exit on error
set -e

# Move to the directory where the script is located to ensure paths are relative
cd "$(dirname "$0")"

# Build the test image
echo "🔨 Building test image..."
docker build -t minesweeper-tests -f Dockerfile.tests .

# Run the tests with Docker socket mounted (for Testcontainers)
echo "🚀 Running tests..."
docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -e "TESTCONTAINERS_HOST_OVERRIDE=host.docker.internal" \
  --add-host=host.docker.internal:host-gateway \
  minesweeper-tests
