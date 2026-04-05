#!/bin/bash
set -e

echo "Running frontend lint..."
(cd frontend && npm run lint)
