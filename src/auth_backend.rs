//! Authentication backend for session login.
//!
//! This module uses a precomputed dummy Argon2 hash for "user not found" and
//! inactive-user login attempts so auth paths have similar verification cost,
//! reducing username-enumeration timing signals.

use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::verify_password;
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::task;
use utoipa::ToSchema;

use crate::models::user::User;

impl AuthUser for User {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Clone, Deserialize, ToSchema)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &crate::security::redaction::REDACTED)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Backend {
    db: SqlitePool,
    dummy_password_hash: String,
}

impl Backend {
    pub fn new(db: SqlitePool, dummy_password_hash: String) -> Self {
        Self {
            db,
            dummy_password_hash,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    Join(#[from] task::JoinError),
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, username, password, role, active, must_change_password FROM users WHERE username = ?"
        )
            .bind(&creds.username)
            .fetch_optional(&self.db)
            .await?;
        let password = creds.password;
        let dummy_hash = self.dummy_password_hash.clone();
        task::spawn_blocking(move || {
            Ok(if let Some(user) = user {
                if !user.active {
                    let _ = verify_password(&password, &dummy_hash);
                    return Ok(None);
                }
                if verify_password(&password, &user.password).is_ok() {
                    Some(user)
                } else {
                    None
                }
            } else {
                // Reduces username enumeration timing signal
                let _ = verify_password(&password, &dummy_hash);
                None
            })
        })
        .await?
    }
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // Deactivated users lose access on next request, even if they still hold a valid
        // session cookie.
        let user = sqlx::query_as(
            "SELECT id, username, password, role, active, must_change_password FROM users \
            WHERE id = ? AND active = 1",
        )
        .bind(*user_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(user)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
