#!/bin/bash
set -e
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Cleanup old process
kill -9 $(lsof -ti :8080) 2>/dev/null || true
sleep 1

cd "$PROJECT_DIR/backend"

# Build if needed
if [ ! -f target/release/backend ]; then
    cargo build --release
fi

export FRONTEND_DIR="$PROJECT_DIR/frontend"
export RUST_BACKTRACE=1
# export JWT_SECRET="your-production-secret-here"

echo "Starting Trello Local on http://localhost:8080 ..."
nohup ./target/release/backend > /tmp/trello.log 2>&1 &
PID=$!
echo "PID: $PID"
sleep 2
echo "Log: /tmp/trello.log"
echo "Done. Open http://localhost:8080"
