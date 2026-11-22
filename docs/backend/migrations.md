# Migrations

SQLx embeds migrations into the binary at compile time with the
`sqlx::migrate!()` macro, and runs any new ones when the app starts. This
means:

1. Migration files must use a name format SQLx accepts.
2. After changing a migration file, rebuild (`cargo build`) for the change to
   take effect.

## Naming format

Use the timestamp format, for example `20251226195900_initial.sql`
(`{YYYYMMDDHHmmss}_name.sql`).

SQLx also accepts a versioned format with a double underscore (`V001__initial.sql`),
but single-underscore prefixes like `001_init.sql` do not work.

## Add a new migration

With `sqlx-cli`:

```bash
cargo install sqlx-cli --no-default-features --features sqlite
sqlx migrate add description_of_change
```

Or create the file by hand:

```bash
touch migrations/$(date +%Y%m%d%H%M%S)_description.sql
```

## Change an existing migration

SQLx refuses to start if a migration that has already run has changed. In
development, delete the local database and rebuild:

```bash
rm data/site-oxidation.db
cargo build
cargo run
```
