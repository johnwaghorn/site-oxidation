pub const UPDATE_PASSWORD: &str =
    "UPDATE users SET password = ?, must_change_password = 0 WHERE id = ?";

pub const SELECT_USER_BY_ID: &str =
    "SELECT id, username, password, role, active, must_change_password FROM users WHERE id = ?";
