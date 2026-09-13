use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use time::OffsetDateTime;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{self, Error as StoreError, SessionStore};

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn delete_expired(&self) -> session_store::Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query("DELETE FROM sessions WHERE expiry_date <= ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|err| StoreError::Backend(err.to_string()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SessionStore for SqliteStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        loop {
            let data = serde_json::to_string(&record.data)
                .map_err(|err| StoreError::Encode(err.to_string()))?;
            let result = sqlx::query(
                "INSERT OR ABORT INTO sessions (id, data, expiry_date) VALUES (?, ?, ?)",
            )
            .bind(record.id.to_string())
            .bind(data)
            .bind(record.expiry_date.unix_timestamp())
            .execute(&self.pool)
            .await;
            match result {
                Ok(_) => return Ok(()),
                Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                    // ID collision so we generate a new ID and try again.
                    record.id = Id::default();
                }
                Err(err) => return Err(StoreError::Backend(err.to_string())),
            }
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let data = serde_json::to_string(&record.data)
            .map_err(|err| StoreError::Encode(err.to_string()))?;
        let expiry_date = record.expiry_date.unix_timestamp();
        sqlx::query(
            "INSERT INTO sessions (id, data, expiry_date) VALUES (?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, expiry_date = excluded.expiry_date",
        )
        .bind(record.id.to_string())
        .bind(data)
        .bind(expiry_date)
        .execute(&self.pool)
        .await
        .map_err(|err| StoreError::Backend(err.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let row: Option<(String, i64)> = sqlx::query_as(
            "SELECT data, expiry_date FROM sessions WHERE id = ? AND expiry_date > ?",
        )
        .bind(session_id.to_string())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| StoreError::Backend(err.to_string()))?;
        let Some((data, expiry_date)) = row else {
            return Ok(None);
        };
        let data: HashMap<String, Value> =
            serde_json::from_str(&data).map_err(|err| StoreError::Decode(err.to_string()))?;
        let expiry_date = OffsetDateTime::from_unix_timestamp(expiry_date)
            .map_err(|err| StoreError::Decode(err.to_string()))?;
        Ok(Some(Record {
            id: *session_id,
            data,
            expiry_date,
        }))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|err| StoreError::Backend(err.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::Duration;

    fn record(expires_in: Duration) -> Record {
        Record {
            id: Id::default(),
            data: HashMap::from([("user_id".to_owned(), json!(42))]),
            expiry_date: OffsetDateTime::now_utc() + expires_in,
        }
    }

    async fn count_sessions(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_load_delete_round_trip(pool: SqlitePool) {
        let store = SqliteStore::new(pool);
        let record = record(Duration::hours(1));
        store.save(&record).await.unwrap();
        let loaded = store.load(&record.id).await.unwrap().unwrap();
        assert_eq!(loaded.id, record.id);
        assert_eq!(loaded.data, record.data);
        assert_eq!(
            loaded.expiry_date.unix_timestamp(),
            record.expiry_date.unix_timestamp(),
            "expiry is stored at whole-second precision"
        );
        store.delete(&record.id).await.unwrap();
        assert!(store.load(&record.id).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_expired_sessions_are_not_loaded_and_are_purged(pool: SqlitePool) {
        let store = SqliteStore::new(pool.clone());
        let live = record(Duration::hours(1));
        let expired = record(Duration::hours(-1));
        store.save(&live).await.unwrap();
        store.save(&expired).await.unwrap();
        assert!(store.load(&expired.id).await.unwrap().is_none());
        assert_eq!(count_sessions(&pool).await, 2);
        store.delete_expired().await.unwrap();
        assert_eq!(
            count_sessions(&pool).await,
            1,
            "only the live session should survive the purge"
        );
        assert!(store.load(&live.id).await.unwrap().is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_regenerates_id_on_collision(pool: SqlitePool) {
        let store = SqliteStore::new(pool.clone());
        let existing = record(Duration::hours(1));
        store.save(&existing).await.unwrap();
        let mut colliding = record(Duration::hours(1));
        colliding.id = existing.id;
        colliding.data.insert("user_id".to_owned(), json!(7));
        store.create(&mut colliding).await.unwrap();
        assert_ne!(
            colliding.id, existing.id,
            "create must pick a fresh id on collision"
        );
        assert_eq!(count_sessions(&pool).await, 2);
        let untouched = store.load(&existing.id).await.unwrap().unwrap();
        assert_eq!(
            untouched.data, existing.data,
            "the original session must not be overwritten"
        );
    }
}
