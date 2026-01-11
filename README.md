# Site Oxidation

## Tools

```bash
cargo clippy --all -- -W clippy::all -W clippy::pedantic
```

## Migrations

Files must be named `{YYYYMMDDHHmmss}_name.sql` (e.g., `20251226195900_initial.sql`).

Single-underscore prefixes like `001_init.sql` will not work.

See [docs/migrations.md](docs/migrations.md).

## TODO

- tech stack watch - cves
- notifications on outage, cves
- FE
- serve static





