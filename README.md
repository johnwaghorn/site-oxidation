# Site Oxidation

A simple uptime monitor that detects when your sites start to rust.

## Requirements

| Tool | Version              | Configured in                                                                               |
|------|----------------------|---------------------------------------------------------------------------------------------|
| Rust | 1.85+ (edition 2024) | [Cargo.toml](Cargo.toml), [Dockerfile](Dockerfile)                                          |
| Node | 24+                  | [.nvmrc](frontend/.nvmrc), [package.json](frontend/package.json), [Dockerfile](Dockerfile) |

## Development

See [docs/development.md](docs/development.md) for how to run locally, generate OpenAPI types, build the frontend, Docker use etc.

## Migrations

Files must be named `{YYYYMMDDHHmmss}_name.sql` (e.g., `20251226195900_initial.sql`).

Single-underscore prefixes like `001_init.sql` will not work.

See [docs/migrations.md](docs/migrations.md).

## TODO

- tech stack watch - cves
- notifications on outage, cves
- import/export sites to csv
- expand FE
- redo "auth" or remove it



