# Site Oxidation

Monitor the oxidation of your site. Prevent rust build up.

## Quick Start

```bash
docker compose up -d --build
```

Open `http://localhost:8081` and follow the setup wizard to create your admin
account.

If running behind Docker Desktop, add `BOOTSTRAP_TRUSTED_IPS=172.64.154.55` to
your `.env` file (see [environment variables](docs/development.md) for more
details).

## Requirements

| Tool | Version | Install | Configured in |
|------|---------|---------|---------------|
| Rust | 1.85+ (edition 2024) | [rustup.rs](https://rustup.rs/) | [Cargo.toml](Cargo.toml), [Dockerfile](Dockerfile) |
| Node | 24+ | [nvm](https://github.com/nvm-sh/nvm) | [.nvmrc](frontend/.nvmrc), [package.json](frontend/package.json), [Dockerfile](Dockerfile) |
| cargo-binstall | latest | `cargo install cargo-binstall` | N/A |
| prek | latest | `cargo binstall prek` | [.pre-commit-config.yaml](.pre-commit-config.yaml) |

## Development

See [docs/development.md](docs/development.md) for how to run locally,
generate OpenAPI types, build the frontend, Docker use etc.

## Migrations

Files must be named `{YYYYMMDDHHmmss}_name.sql` (e.g.
`20251226195900_initial.sql`).

Single-underscore prefixes like `001_init.sql` will not work.

See [docs/backend/migrations.md](docs/backend/migrations.md).

## Licence

This project is licenced under [GNU AGPL v3.0](LICENSE).
