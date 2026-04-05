#!/bin/bash
set -e

SCHEMA_PORT=19821
ENABLE_SWAGGER_UI=true COOKIE_SECURE=false SERVER_PORT=$SCHEMA_PORT cargo run &
PID=$!
until curl -sf http://localhost:$SCHEMA_PORT/api/docs/openapi.json > /dev/null 2>&1; do
    sleep 0.5
done
cd frontend && npx openapi-typescript http://localhost:$SCHEMA_PORT/api/docs/openapi.json -o ./src/generated/schema.d.ts
kill $PID
