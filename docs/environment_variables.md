# Environment Variables

Most of the configuration is done with environment variables. With Docker Compose, put
them in the `.env` file next to the compose file. On Unraid, set them in the
container template. When running with `cargo run`, export them in your shell
(the `.env` file is not read).

Invalid values stop the app at startup with an error naming the variable.

## Server

### `SERVER_PORT`

Port the server listens on inside the container. To change the port you browse
to with Docker, change the host side of the port mapping instead.

- Default: `8080`
- Example: `SERVER_PORT=3000`

### `DATA_DIR`

Directory for persistent data (the database and session key).

- Default: `./data` (`/app/data` in the Docker image)
- Example: `DATA_DIR=/var/lib/site-oxidation`

### `DATABASE_PATH`

Path to the SQLite database file. Only needed to put the database somewhere
other than `DATA_DIR`.

- Default: `{DATA_DIR}/site-oxidation.db`
- Example: `DATABASE_PATH=/var/lib/site-oxidation/app.db`

## Authentication

### `COOKIE_SECURE`

Whether the session cookie is only sent over HTTPS.

Set it to `true` when the app is reached through HTTPS, for example behind a
reverse proxy. Set it to `false` when browsing to the app directly over plain
HTTP, such as `http://unraid-ip:8026` on your LAN. If it is `true` over plain
HTTP, the browser never sends the cookie back and login silently fails.

- Default: `true`
- Example: `COOKIE_SECURE=false`

### `BOOTSTRAP_REQUIRE_PRIVATE_IP`

Only allow the first-run setup (`/api/setup/bootstrap`) from private or
loopback IP addresses. Set it to `false` if the source IP is not preserved and
setup is refused, for example on some Docker Desktop setups.

- Default: `true`
- Example: `BOOTSTRAP_REQUIRE_PRIVATE_IP=false`

### `BOOTSTRAP_TRUSTED_IPS`

Comma-separated IP addresses that are also allowed to run first-run setup.
Useful on Docker Desktop, where requests arrive from a virtual address such as
`172.64.154.55`.

- Default: none
- Example: `BOOTSTRAP_TRUSTED_IPS=172.64.154.55,10.0.0.1`

## CORS

### `CORS_ALLOWED_ORIGIN`

The frontend's origin, if it is served from a different origin to the API.
When unset, only same-origin requests are allowed, which is right for the
normal setup where the app serves its own frontend.

- Default: unset (same-origin only)
- Example: `CORS_ALLOWED_ORIGIN=https://waghorn.tech`

## Logging

### `RUST_LOG`

How much the app logs. Use `debug` when reporting a problem. Accepts any
[`tracing` filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html),
such as `site_oxidation=debug` to limit debug output to the app itself.

- Default: `info`
- Example: `RUST_LOG=debug`

## Swagger / OpenAPI

### `ENABLE_SWAGGER_UI`

Serve the Swagger UI at `/api/docs`.

- Default: `false`
- Example: `ENABLE_SWAGGER_UI=true`

## Site probing

### `PROBE_TIMEOUT_SECS`

Timeout in seconds for each probe request. Must be greater than 0.

- Default: `30`
- Example: `PROBE_TIMEOUT_SECS=10`

### `PROBE_BODY_SIZE_LIMIT_BYTES`

Maximum number of response body bytes read when a site has an expected text
check. Reading stops as soon as the text is found. Must be greater than 0.

- Default: `1048576` (1 MiB)
- Example: `PROBE_BODY_SIZE_LIMIT_BYTES=2097152`

### `PROBE_RETRY_COUNT`

Number of retries after a failed probe before the site is marked as down.

- Default: `2`
- Example: `PROBE_RETRY_COUNT=3`

### `PROBE_RETRY_DELAY_MS`

Delay in milliseconds between probe retries.

- Default: `3000`
- Example: `PROBE_RETRY_DELAY_MS=5000`

### `PROBE_MAX_CONCURRENT_CHECKS`

Maximum number of sites probed at the same time. Must be greater than 0.

- Default: `20`
- Example: `PROBE_MAX_CONCURRENT_CHECKS=50`

### `PROBE_ALLOW_PRIVATE_IPS`

Allow monitoring sites on private or internal addresses (such as `192.168.x.x`
or `10.x.x.x`). Turn this on to monitor services on your own network.

- Default: `false`
- Example: `PROBE_ALLOW_PRIVATE_IPS=true`

### `PROBE_USER_AGENT`

User-Agent header sent with probe requests.

- Default: `SiteOxidation/1.0 (+https://github.com/johnwaghorn/site-oxidation)`
- Example: `PROBE_USER_AGENT=InternalOrg/1.0`

## Certificates

### `CERT_WARN_DAYS`

Days before a TLS certificate expires at which it is shown as expiring soon.

- Default: `30`
- Example: `CERT_WARN_DAYS=21`

### `CERT_CRITICAL_DAYS`

Days before a TLS certificate expires at which it is shown as critical. Must
not be greater than `CERT_WARN_DAYS`, and neither can be negative.

- Default: `7`
- Example: `CERT_CRITICAL_DAYS=3`

## Notifications

### `SMTP_ALLOW_PRIVATE_HOSTS`

Allow email alerts to be sent through an SMTP server on a private or internal
address, such as a mail relay on your LAN. When off, SMTP hosts that resolve to
private addresses are refused before sending.

- Default: `false`
- Example: `SMTP_ALLOW_PRIVATE_HOSTS=true`

## Docker image

These are read by the container's startup script, not by the app itself.

### `PUID` and `PGID`

User and group IDs the app runs as. On startup the container gives ownership of
`/app/data` to this user, then drops to it from root. The defaults match
Unraid's `nobody:users`.

- Default: `99` and `100`
- Example: `PUID=1000` and `PGID=1000`
