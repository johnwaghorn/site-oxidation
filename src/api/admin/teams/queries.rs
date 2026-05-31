pub const LIST_TEAMS: &str = concat!(
    "SELECT t.id, t.name, ",
    "COUNT(DISTINCT CASE WHEN u.active = 1 THEN tm.user_id END) AS member_count, ",
    "COUNT(DISTINCT s.id) AS site_count ",
    "FROM teams t ",
    "LEFT JOIN team_members tm ON t.id = tm.team_id ",
    "LEFT JOIN users u ON u.id = tm.user_id ",
    "LEFT JOIN sites s ON t.id = s.team_id ",
    "GROUP BY t.id ",
    "ORDER BY t.id ",
    "LIMIT ? OFFSET ?"
);

pub const COUNT_TEAMS: &str = "SELECT COUNT(*) FROM teams";

pub const SEARCH_TEAM_OPTIONS: &str = concat!(
    "SELECT id, name FROM teams ",
    "WHERE (?1 IS NULL OR name LIKE '%' || ?1 || '%') ",
    "ORDER BY name LIMIT ?2"
);

pub const INSERT_TEAM: &str = "INSERT INTO teams (name) VALUES (?) RETURNING id";

pub const UPDATE_TEAM: &str = "UPDATE teams SET name = ? WHERE id = ? RETURNING id";

pub const DELETE_TEAM: &str = concat!(
    "DELETE FROM teams WHERE id = ?1 AND NOT EXISTS (",
    "  SELECT 1 FROM team_members tm ",
    "  JOIN users u ON u.id = tm.user_id ",
    "  WHERE tm.team_id = ?1 AND u.role = 'user' AND NOT EXISTS (",
    "    SELECT 1 FROM team_members other ",
    "    WHERE other.user_id = tm.user_id AND other.team_id != ?1",
    "  )",
    ") RETURNING id"
);

pub const COUNT_TEAM_SITES: &str = "SELECT COUNT(*) FROM sites WHERE team_id = ?";

pub const ADD_TEAM_MEMBER: &str = "INSERT INTO team_members (team_id, user_id) VALUES (?, ?)";

pub const REMOVE_TEAM_MEMBER: &str = concat!(
    "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2 AND (",
    "  EXISTS(SELECT 1 FROM users WHERE id = ?2 AND role = 'admin') ",
    "  OR EXISTS(",
    "    SELECT 1 FROM team_members ",
    "    WHERE user_id = ?2 AND team_id != ?1",
    "  )",
    ") RETURNING team_id"
);

pub const MEMBERSHIP_EXISTS: &str =
    "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ?)";

pub const TEAM_EXISTS: &str = "SELECT COUNT(*) FROM teams WHERE id = ?";

pub const USER_EXISTS: &str = "SELECT COUNT(*) FROM users WHERE id = ?";
