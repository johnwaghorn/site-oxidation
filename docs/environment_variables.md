# Environment Variables

All configuration is done via environment variables. Set them in your `.env`
file (for Docker) or export them in your shell.

## Server

### `SERVER_PORT`

Port the server listens on.

- **Default:** `8080`
- **Example:** `SERVER_PORT=3000`

### `DATA_DIR`

Base directory for persistent data (database, session key).

- **Default:** `./data`
- **Example:** `DATA_DIR=/app/data`

### `DATABASE_PATH`

Path to the SQLite database file. If not set, derived from `DATA_DIR`.

- **Default:** `{DATA_DIR}/site-oxidation.db`
- **Example:** `DATABASE_PATH=/var/lib/site-oxidation/app.db`

## Authentication

### `COOKIE_SECURE`

Whether session cookies require HTTPS. Set to `false` for local development
over plain HTTP.

- **Default:** `true`
- **Example:** `COOKIE_SECURE=false`

### `BOOTSTRAP_REQUIRE_PRIVATE_IP`

Restrict the bootstrap endpoint (`/api/setup/bootstrap`) to private/loopback
IP addresses. Set to `false` if deploying behind a reverse proxy or on Docker
Desktop where the source IP is not preserved.

- **Default:** `true`
- **Example:** `BOOTSTRAP_REQUIRE_PRIVATE_IP=false`

### `BOOTSTRAP_TRUSTED_IPS`

Comma-separated list of additional IP addresses allowed to access the
bootstrap endpoint. Useful for Docker Desktop where the source IP is a
gVisor virtual address (e.g. `172.64.154.55`).

- **Default:** *(empty)*
- **Example:** `BOOTSTRAP_TRUSTED_IPS=172.64.154.55,10.0.0.1`

## CORS

### `CORS_ALLOWED_ORIGIN`

If the frontend is served from a different origin to the API, set this to
the frontend's origin. When unset, only same-origin requests are allowed
(the default for a self-hosted app where the API serves the frontend).

- **Default:** *(unset - same-origin only)*
- **Example:** `CORS_ALLOWED_ORIGIN=https://waghorn.tech`

## Swagger / OpenAPI

### `ENABLE_SWAGGER_UI`

Expose the Swagger UI at `/api/docs`. Disabled by default in production.

- **Default:** `false`
- **Example:** `ENABLE_SWAGGER_UI=true`

## Site Probing

### `PROBE_TIMEOUT_SECS`

Timeout in seconds for each site probe request.

- **Default:** `30`
- **Example:** `PROBE_TIMEOUT_SECS=10`

### `PROBE_RETRY_COUNT`

Number of retries after a failed probe before marking the site as down.

- **Default:** `2`
- **Example:** `PROBE_RETRY_COUNT=3`

### `PROBE_RETRY_DELAY_MS`

Delay in milliseconds between probe retries.

- **Default:** `3000`
- **Example:** `PROBE_RETRY_DELAY_MS=5000`

### `PROBE_MAX_CONCURRENT_CHECKS`

Maximum number of sites probed concurrently.

- **Default:** `20`
- **Example:** `PROBE_MAX_CONCURRENT_CHECKS=50`

### `PROBE_ALLOW_PRIVATE_IPS`

Allow probing sites with private/internal IP addresses (e.g. `192.168.x.x`,
`10.x.x.x`). Enable this if you monitor internal services.

- **Default:** `false`
- **Example:** `PROBE_ALLOW_PRIVATE_IPS=true`

### `PROBE_USER_AGENT`

User-Agent header sent with probe requests.

- **Default:** `SiteOxidation/1.0 (+https://github.com/johnwaghorn/site-oxidation)`
- **Example:** `PROBE_USER_AGENT=InternalOrg/1.0`

## Canary

### `CANARY_URL`

URL used for connectivity canary checks. If this URL is unreachable, probes
are skipped to avoid false positives from network outages.

- **Default:** `https://www.google.com`
- **Example:** `CANARY_URL=https://cloudflare.com`

### `CANARY_TIMEOUT_SECS`

Timeout in seconds for the canary check.

- **Default:** `3`
- **Example:** `CANARY_TIMEOUT_SECS=5`
