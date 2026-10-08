use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use sqlx::{SqliteConnection, SqlitePool};
use time::{Duration, OffsetDateTime};

use super::{AuthError, Authenticator, Credentials, queries};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::models::user::User;

pub const SESSION_COOKIE: &str = "id";
const SESSION_TTL: Duration = Duration::days(7);

#[derive(Clone)]
pub struct AuthSession {
    pub user: Option<User>,
    live_session_id: Option<String>,
    jar: SignedCookieJar,
    authenticator: Authenticator,
}

impl AuthSession {
    pub async fn authenticate(&self, creds: Credentials) -> Result<Option<User>, AuthError> {
        self.authenticator.authenticate(creds).await
    }

    pub async fn login(self, verified_user: &User) -> Result<SignedCookieJar, AuthError> {
        let mut tx = self.authenticator.pool.begin().await?;
        if let Some(replaced_session_id) = &self.live_session_id {
            delete_session_by_id(&mut tx, replaced_session_id).await?;
        }
        let session_id =
            insert_session(&mut tx, verified_user.id, verified_user.auth_revision).await?;
        tx.commit().await?;
        Ok(self.jar.add(build_session_cookie(
            session_id,
            self.authenticator.cookie_secure,
        )))
    }

    pub async fn change_password(
        self,
        user: &User,
        new_password_hash: &str,
    ) -> Result<SignedCookieJar, AuthError> {
        let mut tx = self.authenticator.pool.begin().await?;
        let updated = sqlx::query(queries::UPDATE_PASSWORD)
            .bind(new_password_hash)
            .bind(user.id)
            .bind(user.auth_revision)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() != 1 {
            return Err(AuthError::StaleAuthorization);
        }
        let jar = self.replace_all_sessions(&mut tx, user).await?;
        tx.commit().await?;
        Ok(jar)
    }

    async fn replace_all_sessions(
        self,
        tx: &mut SqliteConnection,
        user: &User,
    ) -> Result<SignedCookieJar, AuthError> {
        delete_all_sessions_for_user(tx, user.id).await?;
        let current_auth_revision: i64 = sqlx::query_scalar(queries::SELECT_AUTH_REVISION)
            .bind(user.id)
            .fetch_one(&mut *tx)
            .await?;
        let session_id = insert_session(tx, user.id, current_auth_revision).await?;
        Ok(self.jar.add(build_session_cookie(
            session_id,
            self.authenticator.cookie_secure,
        )))
    }

    pub async fn logout(self) -> Result<SignedCookieJar, AuthError> {
        if let Some(session_id) = &self.live_session_id {
            let mut conn = self.authenticator.pool.acquire().await?;
            delete_session_by_id(&mut conn, session_id).await?;
        }
        Ok(self.jar.remove(Cookie::build(SESSION_COOKIE).path("/")))
    }
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    Authenticator: FromRef<S>,
{
    type Rejection = ApiErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(loaded_earlier_in_this_request) = parts.extensions.get::<AuthSession>() {
            return Ok(loaded_earlier_in_this_request.clone());
        }
        let authenticator = Authenticator::from_ref(state);
        let jar = SignedCookieJar::from_headers(&parts.headers, authenticator.key.clone());
        let cookie_session_id = jar
            .get(SESSION_COOKIE)
            .map(|cookie| cookie.value().to_owned());
        let (user, live_session_id) = match cookie_session_id {
            Some(session_id) => {
                match find_active_user_for_session(&authenticator.pool, &session_id)
                    .await
                    .map_err(|e| internal_err("Failed to load session", e))?
                {
                    Some(user) => (Some(user), Some(session_id)),
                    None => (None, None),
                }
            }
            None => (None, None),
        };
        let auth_session = Self {
            user,
            live_session_id,
            jar,
            authenticator,
        };
        parts.extensions.insert(auth_session.clone());
        Ok(auth_session)
    }
}

pub async fn delete_expired_sessions(pool: &SqlitePool) -> sqlx::Result<()> {
    sqlx::query(queries::DELETE_EXPIRED_SESSIONS)
        .bind(now_unix_timestamp())
        .execute(pool)
        .await?;
    Ok(())
}

async fn delete_all_sessions_for_user(
    conn: &mut SqliteConnection,
    user_id: i64,
) -> sqlx::Result<()> {
    sqlx::query(queries::DELETE_USER_SESSIONS)
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(())
}

async fn delete_session_by_id(conn: &mut SqliteConnection, session_id: &str) -> sqlx::Result<()> {
    sqlx::query(queries::DELETE_SESSION)
        .bind(session_id)
        .execute(conn)
        .await?;
    Ok(())
}

async fn insert_session(
    conn: &mut SqliteConnection,
    user_id: i64,
    verified_auth_revision: i64,
) -> Result<String, AuthError> {
    let session_id = random_128_bit_hex_id()?;
    sqlx::query(queries::INSERT_SESSION)
        .bind(&session_id)
        .bind(user_id)
        .bind(verified_auth_revision)
        .bind(expiry_unix_timestamp_from_now())
        .execute(conn)
        .await?;
    Ok(session_id)
}

async fn find_active_user_for_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<User>, AuthError> {
    let user = sqlx::query_as(queries::SELECT_ACTIVE_USER_FOR_SESSION)
        .bind(session_id)
        .bind(now_unix_timestamp())
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

fn build_session_cookie(session_id: String, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, session_id))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(SESSION_TTL)
        .build()
}

fn random_128_bit_hex_id() -> Result<String, AuthError> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(format!("{:032x}", u128::from_le_bytes(bytes)))
}

fn now_unix_timestamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn expiry_unix_timestamp_from_now() -> i64 {
    OffsetDateTime::now_utc()
        .saturating_add(SESSION_TTL)
        .unix_timestamp()
}

#[cfg(test)]
#[path = "../tests/auth/session.rs"]
mod tests;
