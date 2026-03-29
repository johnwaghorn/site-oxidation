#!/bin/bash
set -e

echo "Running frontend type check..."
(cd frontend && npx tsc -b --noEmit)

echo "Running frontend eslint..."
(cd frontend && npx eslint src/)
