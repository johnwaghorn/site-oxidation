# Development Guide

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

- Make sure the backend is running first: `cargo run`
- From the frontend directory, run: `npm run generate-api-schema`
- This fetches the schema from `http://localhost:8080/api/docs/openapi.json` and writes types to `src/generated/schema.d.ts`

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

## Running Tests

- Run all tests: `cargo test`
- Run with output: `cargo test -- --nocapture`

## Formatting and Linting

- Format Rust code: `cargo fmt`
- Check formatting without modifying: `cargo fmt --check`
- Lint Rust code: `cargo clippy --all -- -W clippy::all -W clippy::pedantic`
- Lint frontend: `cd frontend && npm run lint`
