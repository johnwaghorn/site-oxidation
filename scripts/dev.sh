#!/bin/bash
set -e
cd "$(dirname "$0")/.."

if [[ "$1" == "--reset-db" ]]; then
    rm -rf "${DATA_DIR:-./data}"
fi

PORT="${SERVER_PORT:-8080}"
if lsof -i ":$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port $PORT is already in use:"
    lsof -i ":$PORT" -sTCP:LISTEN
    echo "Kill it with: kill -9 $(lsof -ti ":$PORT" -sTCP:LISTEN)"
    exit 1
fi

cargo run &
BACKEND_PID=$!

until curl -sf "http://localhost:$PORT/health" >/dev/null 2>&1; do
    if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
        echo "Backend exited before becoming healthy"
        exit 1
    fi
    sleep 0.5
done

npm --prefix frontend run dev &
FRONTEND_PID=$!

trap 'kill $BACKEND_PID $FRONTEND_PID 2>/dev/null' EXIT

wait
