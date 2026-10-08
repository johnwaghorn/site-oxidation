mod authenticator;
mod queries;
mod session;

pub use authenticator::Authenticator;
pub use session::{AuthSession, SESSION_COOKIE, delete_expired_sessions};

use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error("failed to generate session id: {0}")]
    Random(#[from] getrandom::Error),

    #[error("the account changed after this request was authorised")]
    StaleAuthorization,
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
