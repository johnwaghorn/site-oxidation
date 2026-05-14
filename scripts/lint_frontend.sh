#!/bin/bash
set -e

echo "Auto-fixing formatting with Prettier..."
(cd frontend && npm run format)

echo "Running frontend lint..."
(cd frontend && npm run lint)
