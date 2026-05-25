#!/bin/sh
set -e

PUID="${PUID:-99}"
PGID="${PGID:-100}"

mkdir -p /app/data
chown -R "${PUID}:${PGID}" /app/data

exec gosu "${PUID}:${PGID}" "$@"
