#!/bin/bash

API_KEY=123456 cargo run &
PID=$!
until curl -sf http://localhost:8080/api/docs/openapi.json > /dev/null 2>&1; do
    sleep 0.5
done
cd frontend && npm run generate-api-schema
kill $PID