use anyhow::{Context, Result};
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use std::path::Path;

pub async fn init_db(database_path: &str) -> Result<SqlitePool> {
    if let Some(parent) = Path::new(database_path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("Failed to connect SQLite DB at path: {database_path}"))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run SQLite migrations")?;
    Ok(pool)
}
