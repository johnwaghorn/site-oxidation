pub const UPDATE_PASSWORD: &str =
    "UPDATE users SET password = ?, must_change_password = 0 WHERE id = ?";

pub const SELECT_USER_BY_ID: &str =
    "SELECT id, username, password, role, active, must_change_password FROM users WHERE id = ?";

pub const SELECT_USER_TEAMS: &str = concat!(
    "SELECT t.id, t.name ",
    "FROM teams t ",
    "JOIN team_members tm ON t.id = tm.team_id ",
    "WHERE tm.user_id = ? ",
    "ORDER BY t.name"
);

pub const SELECT_ALL_TEAMS: &str = "SELECT id, name FROM teams ORDER BY name";
