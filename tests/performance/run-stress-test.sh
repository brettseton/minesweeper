#!/bin/bash

# Default values
URL=${1:-"http://localhost:4533"}
THREADS=${2:-2}
CONNECTIONS=${3:-100}
DURATION=${4:-"60s"}
SCENARIO=${5:-"complex"}

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "Error: 'wrk' is not installed."
    echo "Please install it using your package manager."
    echo "  MacOS: brew install wrk"
    echo "  Linux: sudo apt-get install wrk"
    exit 1
fi

LUA_SCRIPT="tests/performance/game-scenarios.lua"
if [ "$SCENARIO" == "simple" ]; then
    LUA_SCRIPT="tests/performance/create-game.lua"
fi

echo "Running $SCENARIO stress test against $URL"
echo "Threads: $THREADS, Connections: $CONNECTIONS, Duration: $DURATION"
echo "Using script: $LUA_SCRIPT"
echo "----------------------------------------------------------------"

wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" -s "$LUA_SCRIPT" "$URL"
