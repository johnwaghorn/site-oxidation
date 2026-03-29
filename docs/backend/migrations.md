# Migrations

SQLx embeds migrations at **compile time** via the `sqlx::migrate!()` macro. This means:

1. Migration files must use a valid naming format
2. After changing migration files, you must **rebuild** (`cargo build`) for changes to take effect
3. Migrations run automatically on application startup

## Naming format

SQLx accepts two formats:

| Format | Example |
|--------|---------|
| Timestamp | `20251226195900_initial.sql` |
| Versioned (double underscore) | `V001__initial.sql` |

Use the timestamp format for consistency.

## Add a new migration

```bash
cargo install sqlx-cli --no-default-features --features sqlite
sqlx migrate add description_of_change
```

```bash
touch migrations/$(date +%Y%m%d%H%M%S)_description.sql
```

## Modify migration

```bash
rm data/site-oxidation.db
cargo build
cargo run
```
