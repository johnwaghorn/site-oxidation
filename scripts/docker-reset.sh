#!/bin/bash
set -euo pipefail

docker compose down -v
docker compose up --build -d
