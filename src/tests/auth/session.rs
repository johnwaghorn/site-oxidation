use super::*;
use crate::tests::{TEST_PASSWORD, insert_test_user};
use axum_extra::extract::cookie::Key;
use password_auth::generate_hash;

async fn insert_session_row(pool: &SqlitePool, id: &str, user_id: i64, expiry_date: i64) {
    sqlx::query(
        "INSERT INTO sessions (id, user_id, auth_revision, expiry_date) VALUES (?, ?, 0, ?)",
    )
    .bind(id)
    .bind(user_id)
    .bind(expiry_date)
    .execute(pool)
    .await
    .unwrap();
}

async fn count_sessions(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn bump_auth_revision(pool: &SqlitePool, user_id: i64) {
    sqlx::query("UPDATE users SET auth_revision = auth_revision + 1 WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn auth_session_for(pool: &SqlitePool, session_id: &str) -> (AuthSession, User) {
    let user = find_active_user_for_session(pool, session_id)
        .await
        .unwrap()
        .unwrap();
    let key = Key::generate();
    let auth_session = AuthSession {
        user: Some(user.clone()),
        live_session_id: Some(session_id.to_owned()),
        jar: SignedCookieJar::new(key.clone()),
        authenticator: Authenticator::new(pool.clone(), generate_hash("__dummy__"), key, false),
    };
    (auth_session, user)
}

async fn stored_password_hash(pool: &SqlitePool, user_id: i64) -> String {
    sqlx::query_scalar("SELECT password FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[test]
fn test_session_cookie_is_secure_when_enabled() {
    let cookie = build_session_cookie("abc".to_owned(), true);
    assert_eq!(cookie.secure(), Some(true));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_expired_sessions_are_rejected_and_purged(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_session_row(&pool, "live", user_id, expiry_unix_timestamp_from_now()).await;
    insert_session_row(&pool, "expired", user_id, 0).await;

    assert!(
        find_active_user_for_session(&pool, "live")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        find_active_user_for_session(&pool, "expired")
            .await
            .unwrap()
            .is_none()
    );

    delete_expired_sessions(&pool).await.unwrap();
    assert_eq!(
        count_sessions(&pool).await,
        1,
        "only the live session survives the purge"
    );
    assert!(
        find_active_user_for_session(&pool, "live")
            .await
            .unwrap()
            .is_some()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_all_sessions_for_user_leaves_other_users_alone(pool: SqlitePool) {
    let alice = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let bob = insert_test_user(&pool, "bob", TEST_PASSWORD, "user", false).await;
    insert_session_row(&pool, "alice-1", alice, expiry_unix_timestamp_from_now()).await;
    insert_session_row(&pool, "alice-2", alice, expiry_unix_timestamp_from_now()).await;
    insert_session_row(&pool, "bob-1", bob, expiry_unix_timestamp_from_now()).await;

    let mut conn = pool.acquire().await.unwrap();
    delete_all_sessions_for_user(&mut conn, alice)
        .await
        .unwrap();

    assert!(
        find_active_user_for_session(&pool, "alice-1")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        find_active_user_for_session(&pool, "alice-2")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        find_active_user_for_session(&pool, "bob-1")
            .await
            .unwrap()
            .is_some()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_change_password_is_rejected_when_the_account_changed_mid_request(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_session_row(&pool, "current", user_id, expiry_unix_timestamp_from_now()).await;
    let (auth_session, user_as_authorised) = auth_session_for(&pool, "current").await;
    let hash_before = stored_password_hash(&pool, user_id).await;

    bump_auth_revision(&pool, user_id).await;
    let result = auth_session
        .change_password(&user_as_authorised, &generate_hash("a-new-password-42"))
        .await;

    assert!(
        matches!(result, Err(AuthError::StaleAuthorization)),
        "an admin reset landing after authorisation must win, got {result:?}"
    );
    assert_eq!(
        stored_password_hash(&pool, user_id).await,
        hash_before,
        "password left untouched"
    );
    assert_eq!(
        count_sessions(&pool).await,
        1,
        "no replacement session was created"
    );
}
