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

# 4. Verify local .auth.env files for Kustomize secretGenerator (do not commit)
AUTH_ENV_FILES=(
    "src/backend/dotnet/src/kubernetes-manifests/.auth.env"
    "src/backend/rust/kubernetes-manifests/.auth.env"
)

SESSION_ENV="src/backend/rust/kubernetes-manifests/.session.env"

auth_env_needs_values() {
    local auth_env="$1"

    if [ ! -f "$auth_env" ]; then
        return 0
    fi

    # Missing or placeholder values should trigger a prompt.
    if ! grep -Eq '^GOOGLE_CLIENT_ID=.+$' "$auth_env"; then
        return 0
    fi
    if ! grep -Eq '^GOOGLE_CLIENT_SECRET=.+$' "$auth_env"; then
        return 0
    fi
    if grep -Eq '^GOOGLE_CLIENT_ID=(PLACEHOLDER|your-client-|your-client-secret|your-client-id|your-client-id\.apps\.googleusercontent\.com)' "$auth_env"; then
        return 0
    fi
    if grep -Eq '^GOOGLE_CLIENT_SECRET=(PLACEHOLDER|your-client-|your-client-secret)' "$auth_env"; then
        return 0
    fi

    return 1
}

generate_session_secret() {
    if command -v openssl &> /dev/null; then
        openssl rand -hex 32
        return 0
    fi

    if command -v od &> /dev/null; then
        # 32 bytes => 64 hex chars
        od -An -N32 -tx1 </dev/urandom | tr -d ' \n'
        echo ""
        return 0
    fi

    return 1
}

ensure_session_env() {
    local session_env="$1"
    local session_env_example="${session_env}.example"

    if [ -f "$session_env" ] && grep -Eq '^SESSION_SECRET_KEY=.{64,}$' "$session_env"; then
        return 0
    fi

    if [ ! -f "$session_env" ] && [ -f "$session_env_example" ]; then
        echo "⚠️  $session_env not found; copying from $session_env_example"
        cp "$session_env_example" "$session_env"
    fi

    local secret_key="${SESSION_SECRET_KEY:-}"
    if [ -z "$secret_key" ]; then
        if ! secret_key="$(generate_session_secret)"; then
            echo "⚠️  Unable to auto-generate SESSION_SECRET_KEY (missing 'openssl' and 'od')."
            if [ -f "$session_env_example" ]; then
                echo "✅ Using $session_env_example as a template: $session_env"
                cp "$session_env_example" "$session_env"
            fi
            echo "❌ Set SESSION_SECRET_KEY in your environment (64+ chars) or edit $session_env, then rerun."
            exit 1
        fi
    fi

    if [ ${#secret_key} -lt 64 ]; then
        echo "❌ SESSION_SECRET_KEY must be at least 64 characters long."
        exit 1
    fi

    local session_env_dir
    session_env_dir="$(dirname "$session_env")"
    mkdir -p "$session_env_dir"

    echo "✅ Writing $session_env"
    (
        umask 077
        cat <<EOF > "$session_env"
SESSION_SECRET_KEY=$secret_key
EOF
    )
}

write_auth_env() {
    local auth_env="$1"
    local client_id="$2"
    local client_secret="$3"

    local auth_env_dir
    auth_env_dir="$(dirname "$auth_env")"
    mkdir -p "$auth_env_dir"

    # Keep secrets private on disk.
    (
        umask 077
        cat <<EOF > "$auth_env"
GOOGLE_CLIENT_ID=$client_id
GOOGLE_CLIENT_SECRET=$client_secret
EOF
    )
}

ensure_session_env "$SESSION_ENV"

needs_auth=false
for auth_env in "${AUTH_ENV_FILES[@]}"; do
    if auth_env_needs_values "$auth_env"; then
        needs_auth=true
        break
    fi
done

if [ "$needs_auth" = true ]; then
    client_id="${GOOGLE_CLIENT_ID:-}"
    client_secret="${GOOGLE_CLIENT_SECRET:-}"

    if [ -z "$client_id" ] || [ -z "$client_secret" ]; then
        if [ ! -t 0 ]; then
            echo "❌ Missing Google OAuth credentials and cannot prompt (non-interactive shell)."
            echo "Set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET env vars or create:"
            for auth_env in "${AUTH_ENV_FILES[@]}"; do
                echo "  - $auth_env"
            done
            exit 1
        fi

        echo "🔐 Google OAuth credentials are required to run auth in Kubernetes."
        echo "Create/find them in Google Cloud Console:"
        echo "  https://console.cloud.google.com/apis/credentials"
        read -r -p "Enter GOOGLE_CLIENT_ID: " client_id
        while [ -z "$client_id" ]; do
            read -r -p "Enter GOOGLE_CLIENT_ID (cannot be empty): " client_id
        done

        read -r -s -p "Enter GOOGLE_CLIENT_SECRET: " client_secret
        echo ""
        while [ -z "$client_secret" ]; do
            read -r -s -p "Enter GOOGLE_CLIENT_SECRET (cannot be empty): " client_secret
            echo ""
        done
    fi

    for auth_env in "${AUTH_ENV_FILES[@]}"; do
        if auth_env_needs_values "$auth_env"; then
            echo "✅ Writing $auth_env"
            write_auth_env "$auth_env" "$client_id" "$client_secret"
        fi
    done
fi

echo "✅ Environment is ready."
echo "🚀 Starting 'skaffold dev'..."

# 5. Start Skaffold
# This will build the containers, generate secrets via Kustomize, and deploy to Minikube.
skaffold dev
