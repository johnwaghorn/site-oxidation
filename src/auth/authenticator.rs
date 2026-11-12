use axum_extra::extract::cookie::Key;
use password_auth::verify_password;
use sqlx::SqlitePool;

use super::{AuthError, Credentials, queries};
use crate::models::user::User;
use crate::security::password_hashing::PasswordHashingLimiter;

#[derive(Clone)]
pub struct Authenticator {
    pub(super) pool: SqlitePool,
    dummy_password_hash: String,
    hashing_limiter: PasswordHashingLimiter,
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
            hashing_limiter: PasswordHashingLimiter::sized_for_this_machine(),
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
        let verified_user = self
            .hashing_limiter
            .run(move || {
                let Some(user) = user.filter(|user| user.active) else {
                    mask_username_enumeration_timing(&password, &dummy_hash);
                    return None;
                };
                verify_password(&password, &user.password)
                    .is_ok()
                    .then_some(user)
            })
            .await?;
        Ok(verified_user)
    }
}

fn mask_username_enumeration_timing(password: &str, dummy_hash: &str) {
    let _ = verify_password(password, dummy_hash);
}
