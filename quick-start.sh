#!/bin/bash

# Exit on error
set -e

echo "🔍 Checking dependencies..."

# 1. Check for required tools
for tool in docker minikube skaffold kubectl; do
    if ! command -v $tool &> /dev/null; then
        echo "❌ $tool is not installed. Please install it to continue."
        exit 1
    fi
done

# 2. Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo "❌ Docker is not running. Please start Docker Desktop or your Docker daemon."
    exit 1
fi

# 3. Check Minikube status and start if necessary
if minikube status | grep -q "Stopped" || ! minikube status &> /dev/null; then
    echo "🚀 Starting Minikube..."
    minikube start
else
    echo "✅ Minikube is running."
fi

# 3. Ensure kubectl is using minikube context
echo "🎯 Setting kubectl context to minikube..."
kubectl config use-context minikube

# 4. Verify .auth.env location (handled natively by Kustomize)
AUTH_ENV="src/backend/dotnet/src/kubernetes-manifests/.auth.env"

if [ ! -f "$AUTH_ENV" ]; then
    echo "⚠️  $AUTH_ENV not found!"
    echo "Creating template at $AUTH_ENV..."
    cat <<EOF > "$AUTH_ENV"
GOOGLE_CLIENT_ID=PLACEHOLDER_ID
GOOGLE_CLIENT_SECRET=PLACEHOLDER_SECRET
EOF
    echo "❌ Please edit $AUTH_ENV with your Google OAuth credentials and run this script again."
    exit 1
fi

# Check if placeholders are still there
if grep -q "PLACEHOLDER" "$AUTH_ENV"; then
    echo "❌ Please replace placeholders in $AUTH_ENV with actual credentials."
    exit 1
fi

echo "✅ Environment is ready."
echo "🚀 Starting 'skaffold dev'..."

# 5. Start Skaffold
# This will build the containers, generate secrets via Kustomize, and deploy to Minikube.
skaffold dev