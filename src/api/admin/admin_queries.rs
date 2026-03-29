// Users

pub const LIST_USERS: &str = concat!(
    "SELECT u.id, u.username, u.role, u.active, u.must_change_password, ",
    "COALESCE(GROUP_CONCAT(t.name, ', '), '') AS team_names ",
    "FROM users u ",
    "LEFT JOIN team_members tm ON u.id = tm.user_id ",
    "LEFT JOIN teams t ON tm.team_id = t.id ",
    "GROUP BY u.id ",
    "ORDER BY u.id"
);

pub const INSERT_USER: &str = concat!(
    "INSERT INTO users (username, password, role, must_change_password) ",
    "VALUES (?, ?, ?, 1) RETURNING id"
);

pub const UPDATE_USER: &str = "UPDATE users SET role = ?, active = ? WHERE id = ? RETURNING id";

pub const RESET_PASSWORD: &str =
    "UPDATE users SET password = ?, must_change_password = 1 WHERE id = ? RETURNING id";

pub const COUNT_ACTIVE_ADMINS: &str =
    "SELECT COUNT(*) FROM users WHERE role = 'admin' AND active = 1";

pub const IS_ACTIVE_ADMIN: &str = "SELECT role = 'admin' AND active = 1 FROM users WHERE id = ?";

// Teams

pub const LIST_TEAMS: &str = concat!(
    "SELECT t.id, t.name, ",
    "COUNT(DISTINCT tm.user_id) AS member_count, ",
    "COUNT(DISTINCT s.id) AS site_count ",
    "FROM teams t ",
    "LEFT JOIN team_members tm ON t.id = tm.team_id ",
    "LEFT JOIN sites s ON t.id = s.team_id ",
    "GROUP BY t.id ",
    "ORDER BY t.id"
);

pub const INSERT_TEAM: &str = "INSERT INTO teams (name) VALUES (?) RETURNING id";

pub const UPDATE_TEAM: &str = "UPDATE teams SET name = ? WHERE id = ? RETURNING id";

pub const DELETE_TEAM: &str = "DELETE FROM teams WHERE id = ? RETURNING id";

pub const COUNT_TEAM_SITES: &str = "SELECT COUNT(*) FROM sites WHERE team_id = ?";

pub const ADD_TEAM_MEMBER: &str = "INSERT INTO team_members (team_id, user_id) VALUES (?, ?)";

pub const REMOVE_TEAM_MEMBER: &str =
    "DELETE FROM team_members WHERE team_id = ? AND user_id = ? RETURNING team_id";

pub const TEAM_EXISTS: &str = "SELECT COUNT(*) FROM teams WHERE id = ?";

pub const USER_EXISTS: &str = "SELECT COUNT(*) FROM users WHERE id = ?";
