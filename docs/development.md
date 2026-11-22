# Development Guide

## Prerequisites

| Tool | Version | Install | Configured in |
|------|---------|---------|---------------|
| Rust | 1.94+ (edition 2024) | [rustup.rs](https://rustup.rs/) | [Cargo.toml](../Cargo.toml), [Dockerfile](../Dockerfile) |
| Node | 26+ | [nvm](https://github.com/nvm-sh/nvm) | [.nvmrc](../frontend/.nvmrc), [package.json](../frontend/package.json), [Dockerfile](../Dockerfile) |
| cargo-binstall | latest | `cargo install cargo-binstall` | N/A |
| prek | latest | `cargo binstall prek` | [.pre-commit-config.yaml](../.pre-commit-config.yaml) |

## Initial Setup

```bash
# Build backend dependencies
cargo build

# Install frontend dependencies
cd frontend && npm install

# Install git hooks (both pre-commit and pre-push)
prek install --hook-type pre-commit --hook-type pre-push
```

## Running Locally

- Start the backend: `cargo run`
- The API will be available at `http://localhost:8080`
- The OpenAPI spec is served at `http://localhost:8080/api/docs/openapi.json`

## Frontend Development

- Navigate to the frontend directory: `cd frontend`
- Install dependencies: `npm install`
- Start the dev server: `npm run dev`

## Generating OpenAPI Types

The frontend uses generated TypeScript types from the backend's OpenAPI spec.

There is a convenience script that starts the backend, generates the types,
and stops the backend:

```bash
./scripts/generate-schema.sh
```

Or manually:

1. Start the backend: `cargo run`
2. From the frontend directory: `npm run generate-api-schema`

This fetches the schema from `http://localhost:8080/api/docs/openapi.json` and
writes types to `frontend/src/generated/schema.d.ts`.

## Building the Frontend

- From the frontend directory: `npm run build`
- Built assets are output to `../static/` and served by the Rust backend

## Docker

### Start the app

- Build and run: `docker compose up --build -d`
- The app will be available at `http://localhost:8026`

### Stop the app

- Stop containers: `docker compose down`
- Stop and remove volumes (deletes data): `docker compose down -v`

## Backend tests

- Run all tests: `cargo test`

## Formatting and Linting

Run every check across the whole repo (same as CI):

```bash
prek run --all-files --hook-stage pre-push
```

Useful commands:

- Format Rust code: `cargo fmt`
- Lint Rust code: `cargo clippy` (lints come from `[lints.clippy]` in `Cargo.toml`)
- Lint frontend: `cd frontend && npm run lint`

## Migrations

Migration files must be named `{YYYYMMDDHHmmss}_name.sql` (e.g.
`20251226195900_initial.sql`). Single-underscore prefixes like `001_init.sql`
will not work. See [backend/migrations.md](backend/migrations.md).
