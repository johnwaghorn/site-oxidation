pub const SELECT_USER_BY_USERNAME: &str = concat!(
    "SELECT id, username, password, role, active, must_change_password, theme_preference, auth_revision ",
    "FROM users WHERE username = ?"
);

pub const UPDATE_PASSWORD: &str = concat!(
    "UPDATE users SET password = ?, must_change_password = 0, auth_revision = auth_revision + 1 ",
    "WHERE id = ? AND auth_revision = ? AND active = 1"
);

pub const SELECT_AUTH_REVISION: &str = "SELECT auth_revision FROM users WHERE id = ?";

pub const DELETE_EXPIRED_SESSIONS: &str = "DELETE FROM sessions WHERE expiry_date <= ?";

pub const DELETE_USER_SESSIONS: &str = "DELETE FROM sessions WHERE user_id = ?";

pub const DELETE_SESSION: &str = "DELETE FROM sessions WHERE id = ?";

pub const INSERT_SESSION: &str =
    "INSERT INTO sessions (id, user_id, auth_revision, expiry_date) VALUES (?, ?, ?, ?)";

pub const SELECT_ACTIVE_USER_FOR_SESSION: &str = concat!(
    "SELECT u.id, u.username, u.password, u.role, u.active, u.must_change_password, u.theme_preference, u.auth_revision ",
    "FROM sessions s JOIN users u ON u.id = s.user_id ",
    "WHERE s.id = ? AND s.expiry_date > ? AND u.active = 1 AND s.auth_revision = u.auth_revision"
);
