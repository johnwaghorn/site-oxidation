pub const LIST_USERS: &str = concat!(
    "SELECT u.id, u.username, u.role, u.active, u.must_change_password, ",
    "COALESCE(GROUP_CONCAT(t.name, ', '), '') AS team_names ",
    "FROM users u ",
    "LEFT JOIN team_members tm ON u.id = tm.user_id ",
    "LEFT JOIN teams t ON tm.team_id = t.id ",
    "WHERE (?1 IS NULL OR u.username LIKE '%' || ?1 || '%') ",
    "  AND (?2 IS NULL OR EXISTS ( ",
    "       SELECT 1 FROM team_members tmf WHERE tmf.user_id = u.id AND tmf.team_id = ?2 ",
    "  )) ",
    "  AND (?3 IS NULL OR NOT EXISTS ( ",
    "       SELECT 1 FROM team_members tme WHERE tme.user_id = u.id AND tme.team_id = ?3 ",
    "  )) ",
    "  AND (?4 IS NULL OR u.active = ?4) ",
    "GROUP BY u.id ",
    "ORDER BY u.id ",
    "LIMIT ?5 OFFSET ?6"
);

pub const COUNT_USERS: &str = concat!(
    "SELECT COUNT(*) FROM users u ",
    "WHERE (?1 IS NULL OR u.username LIKE '%' || ?1 || '%') ",
    "  AND (?2 IS NULL OR EXISTS ( ",
    "       SELECT 1 FROM team_members tmf WHERE tmf.user_id = u.id AND tmf.team_id = ?2 ",
    "  )) ",
    "  AND (?3 IS NULL OR NOT EXISTS ( ",
    "       SELECT 1 FROM team_members tme WHERE tme.user_id = u.id AND tme.team_id = ?3 ",
    "  )) ",
    "  AND (?4 IS NULL OR u.active = ?4)"
);

pub const INSERT_USER: &str = concat!(
    "INSERT INTO users (username, password, role, must_change_password) ",
    "VALUES (?, ?, ?, 1) RETURNING id"
);

pub const TEAM_EXISTS: &str = "SELECT COUNT(*) FROM teams WHERE id = ?";

pub const ADD_TEAM_MEMBER: &str = "INSERT INTO team_members (team_id, user_id) VALUES (?, ?)";

pub const UPDATE_USER: &str = concat!(
    "UPDATE users SET role = ?1, active = ?2 ",
    "WHERE id = ?3 AND (",
    "  ?1 != 'user' ",
    "  OR EXISTS(SELECT 1 FROM team_members WHERE user_id = ?3)",
    ") RETURNING id"
);

pub const USER_EXISTS: &str = "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)";

pub const DELETE_USER: &str = concat!(
    "DELETE FROM users WHERE id = ?1 AND (",
    "  role != 'admin' OR active = 0 ",
    "  OR (SELECT COUNT(*) FROM users WHERE role = 'admin' AND active = 1) > 1",
    ") RETURNING id"
);

pub const RESET_PASSWORD: &str =
    "UPDATE users SET password = ?, must_change_password = 1 WHERE id = ? RETURNING id";

pub const COUNT_ACTIVE_ADMINS: &str =
    "SELECT COUNT(*) FROM users WHERE role = 'admin' AND active = 1";

pub const IS_ACTIVE_ADMIN: &str = "SELECT role = 'admin' AND active = 1 FROM users WHERE id = ?";
