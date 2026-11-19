#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."

RESET_DB=false
HTTPS=false
for arg in "$@"; do
    case "$arg" in
        --reset-db) RESET_DB=true ;;
        --https) HTTPS=true ;;
        *)
            echo "Unknown option: $arg (expected --reset-db and/or --https)"
            exit 1
            ;;
    esac
done

port_in_use() {
    (echo >"/dev/tcp/localhost/$1") >/dev/null 2>&1
}

exit_if_port_in_use() {
    if port_in_use "$1"; then
        echo "Port $1 is already in use. Stop whatever is listening on it and try again."
        if command -v lsof >/dev/null 2>&1; then
            lsof -i ":$1" -sTCP:LISTEN
        fi
        exit 1
    fi
}

HTTPS_PORT=8443
if $HTTPS; then
    if ! command -v caddy >/dev/null 2>&1; then
        echo "--https needs Caddy: https://caddyserver.com/docs/install"
        exit 1
    fi
    exit_if_port_in_use "$HTTPS_PORT"
fi

if $RESET_DB; then
    rm -rf "${DATA_DIR:-./data}"
fi

PORT="${SERVER_PORT:-8080}"
exit_if_port_in_use "$PORT"

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

HTTPS_URL="https://localhost:$HTTPS_PORT"
VITE_PORT=5173
CADDY_PID=""
if $HTTPS; then
    caddy reverse-proxy --from "$HTTPS_URL" --to "localhost:$VITE_PORT" &
    CADDY_PID=$!
    echo "HTTPS: open $HTTPS_URL"
fi

trap 'kill $BACKEND_PID $FRONTEND_PID $CADDY_PID 2>/dev/null' EXIT

wait
