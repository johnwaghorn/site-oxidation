# Development Guide

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.85+ (edition 2024) | [rustup.rs](https://rustup.rs/) |
| Node | 24+ | [nvm](https://github.com/nvm-sh/nvm) (see `frontend/.nvmrc`) |
| cargo-binstall | latest | `cargo install cargo-binstall` |
| prek | latest | `cargo binstall prek` |

## Initial Setup

```bash
# Build backend dependencies
cargo build

# Install frontend dependencies
cd frontend && npm install

# Install git hooks
prek install
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
- The app will be available at `http://localhost:8081`

### Stop the app

- Stop containers: `docker compose down`
- Stop and remove volumes (deletes data): `docker compose down -v`

## Backend tests

- Run all tests: `cargo test`

## Formatting and Linting

A convenience script runs all check, for both frontend and backend (the same
checks that run on commit via prek):

```bash
./scripts/lint_full.sh
```

Useful commands:

- Format Rust code: `cargo fmt`
- Lint Rust code: `cargo clippy --all -- -W clippy::all -W clippy::pedantic`
- Lint frontend: `cd frontend && npm run lint`
