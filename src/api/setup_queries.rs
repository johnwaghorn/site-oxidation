pub const COUNT_USERS: &str = "SELECT COUNT(*) FROM users";

pub const INSERT_ADMIN: &str = concat!(
    "INSERT INTO users (username, password, role, must_change_password) ",
    "VALUES ('admin', ?, 'admin', 1)"
);
