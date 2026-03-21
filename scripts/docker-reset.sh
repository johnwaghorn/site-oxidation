#!/bin/bash

set -euo pipefail
docker compose down
rm -f ./data/site-oxidation.db ./data/site-oxidation.db-shm ./data/site-oxidation.db-wal
docker compose up --build -d
