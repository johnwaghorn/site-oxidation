#!/bin/bash
set -e
cd "$(dirname "$0")/.."

npm --prefix frontend run build
cargo build

E2E_DIR="$(mktemp -d)"
DATA_DIR="$E2E_DIR" \
DATABASE_PATH="$E2E_DIR/e2e.sqlite" \
SERVER_PORT="${E2E_PORT:-8123}" \
COOKIE_SECURE=false \
CANARY_URL="http://127.0.0.1:9/canary" \
exec ./target/debug/site-oxidation
