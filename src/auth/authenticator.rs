use axum_extra::extract::cookie::Key;
use password_auth::verify_password;
use sqlx::SqlitePool;
use tokio::task;

use super::{AuthError, Credentials, queries};
use crate::models::user::User;

#[derive(Clone)]
pub struct Authenticator {
    pub(super) pool: SqlitePool,
    dummy_password_hash: String,
    pub(super) key: Key,
    pub(super) cookie_secure: bool,
}

impl Authenticator {
    pub fn new(
        pool: SqlitePool,
        dummy_password_hash: String,
        key: Key,
        cookie_secure: bool,
    ) -> Self {
        Self {
            pool,
            dummy_password_hash,
            key,
            cookie_secure,
        }
    }

    pub async fn authenticate(&self, creds: Credentials) -> Result<Option<User>, AuthError> {
        let user: Option<User> = sqlx::query_as(queries::SELECT_USER_BY_USERNAME)
            .bind(&creds.username)
            .fetch_optional(&self.pool)
            .await?;
        let password = creds.password;
        let dummy_hash = self.dummy_password_hash.clone();
        task::spawn_blocking(move || {
            Ok(if let Some(user) = user {
                if !user.active {
                    mask_username_enumeration_timing(&password, &dummy_hash);
                    return Ok(None);
                }
                if verify_password(&password, &user.password).is_ok() {
                    Some(user)
                } else {
                    None
                }
            } else {
                mask_username_enumeration_timing(&password, &dummy_hash);
                None
            })
        })
        .await?
    }
}

fn mask_username_enumeration_timing(password: &str, dummy_hash: &str) {
    let _ = verify_password(password, dummy_hash);
}
