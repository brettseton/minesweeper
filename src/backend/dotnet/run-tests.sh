#!/bin/bash

# Exit on error
set -e

# Move to the directory where the script is located to ensure paths are relative
cd "$(dirname "$0")"

# Build the test image
echo "🔨 Building test image..."
docker build -t minesweeper-tests -f Dockerfile.tests .

# Run the tests with Docker socket mounted (for Testcontainers)
echo "🚀 Running tests (including Docker Testcontainers)..."
docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  minesweeper-tests