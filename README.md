# Site Oxidation

A simple uptime monitor that detects when your sites start to rust.

## Requirements

| Tool | Version | Install | Configured in |
|------|---------|---------|---------------|
| Rust | 1.85+ (edition 2024) | [rustup.rs](https://rustup.rs/) | [Cargo.toml](Cargo.toml), [Dockerfile](Dockerfile) |
| Node | 24+ | [nvm](https://github.com/nvm-sh/nvm) | [.nvmrc](frontend/.nvmrc), [package.json](frontend/package.json), [Dockerfile](Dockerfile) |
| cargo-binstall | latest | `cargo install cargo-binstall` | — |
| prek | latest | `cargo binstall prek` | [.pre-commit-config.yaml](.pre-commit-config.yaml) |

## Development

See [docs/development.md](docs/development.md) for how to run locally, generate OpenAPI types, build the frontend, Docker use etc.

## Migrations

Files must be named `{YYYYMMDDHHmmss}_name.sql` (e.g., `20251226195900_initial.sql`).

Single-underscore prefixes like `001_init.sql` will not work.

See [docs/backend/migrations.md](docs/backend/migrations.md).
